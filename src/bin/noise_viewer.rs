//! An interactive program that allows the user to view the output of a noise
//! function in realtime. It is meant as a prototype for displaying a dynamically created image in bevy

use bevy::{
    prelude::{
        default, App, Assets, Camera2dBundle, Commands, Handle, Image, Input, KeyCode, Query, Res,
        ResMut, Resource,
    },
    sprite::SpriteBundle,
    window::Window,
    DefaultPlugins,
};
use image::{DynamicImage, RgbaImage};
use indicatif::ProgressIterator;
use noise::{NoiseFn, Perlin, ScalePoint};
use palette::{encoding::Linear, rgb::Rgb, Gradient, LinSrgb, Srgb};
use rand::{distributions::Uniform, thread_rng, Rng};
use tiny_skia::{
    Color as SkiaColor, FillRule, Paint, PathBuilder, Pixmap, PremultipliedColorU8,
    Transform as SkiaTransform,
};

#[derive(Resource, Default, Debug)]
struct DisplayImage(Handle<Image>);

#[derive(Resource, Debug)]
struct NoiseScale(f64);

impl Default for NoiseScale {
    fn default() -> Self {
        NoiseScale(1.)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<DisplayImage>()
        .init_resource::<NoiseScale>()
        .add_startup_system(setup)
        .add_system(update_display)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut display_img: ResMut<DisplayImage>,
    window: Query<&Window>,
) {
    // create camera
    commands.spawn(Camera2dBundle::default());

    // paint a starter pixmap
    let mut rng = thread_rng();
    let window_w = window.single().resolution.width() as u32;
    let window_h = window.single().resolution.height() as u32;
    let pixmap = paint_circle_flag(window_w, window_h, &mut rng);

    // export pixmap to texture
    let rgba = RgbaImage::from_raw(window_w, window_h, pixmap.data().into()).unwrap();
    let dyn_img = DynamicImage::ImageRgba8(rgba);
    let bvy_img = Image::from_dynamic(dyn_img, false);
    let img_handle = images.add(bvy_img);
    *display_img = DisplayImage(img_handle.clone());

    // create a sprite that takes up the whole screen
    commands.spawn(SpriteBundle {
        texture: img_handle,
        ..default()
    });
}

/// update the display image when spacebar is pressed
fn update_display(
    keys: Res<Input<KeyCode>>,
    mut images: ResMut<Assets<Image>>,
    display_img: Res<DisplayImage>,
    window: Query<&Window>,
) {
    if keys.just_pressed(KeyCode::Space) {
        println!("updating display...");
        // set up random generation of colors
        let mut rng = thread_rng();
        let window_w = window.single().resolution.width() as u32;
        let window_h = window.single().resolution.height() as u32;
        let pixmap = paint_noise(window_w, window_h, &mut rng);

        let bvy_img = images.get_mut(&display_img.0).unwrap();
        let rgba = RgbaImage::from_raw(window_w, window_h, pixmap.data().into()).unwrap();
        let dyn_img = DynamicImage::ImageRgba8(rgba);
        *bvy_img = Image::from_dynamic(dyn_img, false);
    }
}

fn paint_circle_flag<R: Rng>(width: u32, height: u32, rng: &mut R) -> Pixmap {
    let color_range = Uniform::new_inclusive(0, 255);
    let mut pixmap = Pixmap::new(width, height).unwrap();
    pixmap.fill(SkiaColor::from_rgba8(
        rng.sample(color_range),
        rng.sample(color_range),
        rng.sample(color_range),
        255,
    ));
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(
        rng.sample(color_range),
        rng.sample(color_range),
        rng.sample(color_range),
        255,
    );
    let mut pb = PathBuilder::new();
    pb.push_circle(
        width as f32 / 2.,
        height as f32 / 2.,
        width.min(height) as f32 / 2. * 0.66,
    );
    let path = pb.finish().unwrap();
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        SkiaTransform::identity(),
        None,
    );
    pixmap
}

fn paint_noise<R: Rng>(width: u32, height: u32, rng: &mut R) -> Pixmap {
    let scale = 4. / width.max(height) as f64;
    let noise = ScalePoint::new(Perlin::new(rng.gen())).set_scale(scale);
    let mut pixmap = Pixmap::new(width, height).unwrap();
    let pixels = pixmap.pixels_mut();

    for i in (0..(width * height)).progress() {
        let x = i % width;
        let y = i / width;
        let v = noise.get([x as f64, y as f64]);
        let rgb = ((v + 1.) / 2. * 256.).clamp(0., 255.) as u8;
        pixels[i as usize] = PremultipliedColorU8::from_rgba(rgb, rgb, rgb, 255).unwrap();
    }
    pixmap
}
