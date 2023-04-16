//! An interactive program that allows the user to view the output of a noise
//! function in realtime. It is meant as a prototype for displaying a dynamically created image in bevy
//!
//! Controls
//! --------
//! Space: generate new random image

use bevy::{
    prelude::{
        default, App, Assets, Camera2dBundle, Commands, Handle, Image, Input, KeyCode, Query, Res,
        ResMut, Resource,
    },
    sprite::SpriteBundle,
    window::Window,
    DefaultPlugins,
};
use clap::Parser;
use image::{DynamicImage, RgbaImage};
use indicatif::ProgressIterator;
use noise::{NoiseFn, ScalePoint};
use palette::{
    encoding::{Linear, Srgb},
    rgb::Rgb,
    Gradient, LinSrgb,
};
use proc_art::noise::NoiseSelector;
use rand::{distributions::Uniform, thread_rng, Rng};
use tiny_skia::{
    Color as SkiaColor, FillRule, Paint, PathBuilder, Pixmap, PremultipliedColorU8,
    Transform as SkiaTransform,
};

#[derive(Parser, Resource, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// random seed
    #[arg(long)]
    seed: Option<u64>,

    #[arg(long, value_enum, default_value_t = NoiseSelector::Perlin)]
    noise_type: NoiseSelector,

    /// noise scale
    #[arg(long, default_value_t = 4.)]
    noise_scale: f64,
}

impl Args {
    fn get_scaled_noise(
        &self,
        seed: u32,
        window_width: u32,
        window_height: u32,
    ) -> Box<dyn NoiseFn<f64, 2>> {
        let noise_fn = self.noise_type.get_noise_2d(seed);
        let scale = self.noise_scale / window_width.max(window_height) as f64;
        let noise_fn = ScalePoint::new(noise_fn).set_scale(scale);
        Box::new(noise_fn)
    }
}

#[derive(Resource, Default, Debug)]
struct DisplayImage(Handle<Image>);

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(args)
        .init_resource::<DisplayImage>()
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
    args: Res<Args>,
    window: Query<&Window>,
) {
    if keys.just_pressed(KeyCode::Space) {
        println!("updating display...");
        // set up random generation of colors
        let mut rng = thread_rng();
        let window_w = window.single().resolution.width() as u32;
        let window_h = window.single().resolution.height() as u32;
        let noise_fn = args.get_scaled_noise(rng.gen(), window_w, window_h);

        // TODO: read from palette files
        let colors: Vec<_> = (0..5)
            .map(|i| {
                let range = Uniform::new(0., 1. / 5. * (i + 1) as f64);
                let r = rng.sample(range);
                let g = rng.sample(range);
                let b = rng.sample(range);
                LinSrgb::new(r, g, b)
            })
            .collect();

        // let pixmap = paint_noise(window_w, window_h, &mut rng);
        let pixmap = paint_noise(&noise_fn, &colors, window_w, window_h);

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

fn paint_noise<N: NoiseFn<f64, 2>>(
    noise_fn: &N,
    colors: &Vec<Rgb<Linear<Srgb>, f64>>,
    width: u32,
    height: u32,
) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).unwrap();
    let pixels = pixmap.pixels_mut();
    let gradient = Gradient::new(colors.clone());

    for i in (0..(width * height)).progress() {
        let x = i % width;
        let y = i / width;
        let v = ((noise_fn.get([x as f64, y as f64]) + 1.) / 2.).clamp(0., 1.);
        let color = gradient.get(v);
        // TODO: convert with convenience function
        pixels[i as usize] = PremultipliedColorU8::from_rgba(
            (color.red * 255.) as u8,
            (color.green * 255.) as u8,
            (color.blue * 255.) as u8,
            255,
        )
        .unwrap();
    }
    pixmap
}
