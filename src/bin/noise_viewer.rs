//! An interactive program that allows the user to view the output of a noise
//! function in realtime. It is meant as a prototype for displaying a dynamically created image in bevy
//!
//! Controls
//! --------
//! Space: generate new random image

use bevy::{
    prelude::{
        default, App, Assets, Camera2dBundle, Commands, EventReader, EventWriter, Handle, Image,
        Input, KeyCode, Query, Res, ResMut, Resource,
    },
    sprite::SpriteBundle,
    window::Window,
    DefaultPlugins,
};
use clap::Parser;
use image::{DynamicImage, RgbaImage};
use indicatif::ProgressIterator;
use log::info;
use noise::{NoiseFn, ScalePoint};
use palette::{
    encoding::{Linear, Srgb},
    rgb::Rgb,
    Gradient, LinSrgb,
};
use proc_art::noise::NoiseSelector;
use rand::{distributions::Uniform, thread_rng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use tiny_skia::{
    Color as SkiaColor, FillRule, Paint, PathBuilder, Pixmap, PremultipliedColorU8,
    Transform as SkiaTransform,
};

#[derive(Parser, Resource, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// initial random seed
    #[arg(long)]
    seed: Option<u64>,

    /// type of random noise
    #[arg(long, value_enum, default_value_t = NoiseSelector::Perlin)]
    noise_type: NoiseSelector,

    /// noise scale
    #[arg(long, default_value_t = 4.)]
    noise_scale: f64,

    /// window width
    #[arg(long, default_value_t = 800.)]
    width: f64,

    /// window height
    #[arg(long, default_value_t = 600.)]
    height: f64,
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

    fn get_seed(&self) -> u64 {
        match self.seed {
            Some(s) => s,
            None => thread_rng().gen(),
        }
    }
}

#[derive(Resource, Default, Debug)]
struct DisplayImage(Handle<Image>);

/// Resource containing the current random seed.  This is different from the seed provided in Args, which is just the initial seed provided to the system.
#[derive(Resource, Default, Debug)]
struct RandomSeed(u64);

enum ResourceUpdatedEvent {
    Args,
    RandomSeed,
    WindowSize,
}

fn main() {
    let args = Args::parse();
    let seed = RandomSeed(args.get_seed());

    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<ResourceUpdatedEvent>()
        .insert_resource(args)
        .insert_resource(seed)
        .init_resource::<DisplayImage>()
        .add_startup_system(bevy_setup)
        .add_system(handle_input)
        .add_system(update_display)
        .run();
}

fn bevy_setup(
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

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut args: ResMut<Args>,
    mut seed: ResMut<RandomSeed>,
    mut ev_updated: EventWriter<ResourceUpdatedEvent>,
) {
    // Spacebar - generate new random seed
    if keys.just_pressed(KeyCode::Space) {
        *seed = RandomSeed(thread_rng().gen());
        info!("random seed: {}", seed.0);
        ev_updated.send(ResourceUpdatedEvent::RandomSeed);
    }

    if keys.just_pressed(KeyCode::Tab) {
        if keys.any_pressed([KeyCode::LShift, KeyCode::RShift]) {
            args.noise_type = args.noise_type.get_prev();
        } else {
            args.noise_type = args.noise_type.get_next();
        }
        info!("noise type: {:?}", args.noise_type);
        ev_updated.send(ResourceUpdatedEvent::Args);
    }
}

/// update the display image when spacebar is pressed
fn update_display(
    mut ev_updated: EventReader<ResourceUpdatedEvent>,
    display_img: Res<DisplayImage>,
    args: Res<Args>,
    seed: Res<RandomSeed>,
    window: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    // check if we care about anything that refreshed
    let mut should_refresh = false;
    for ev in ev_updated.iter() {
        match ev {
            ResourceUpdatedEvent::Args | ResourceUpdatedEvent::RandomSeed => {
                should_refresh = true;
            }
        }
    }
    ev_updated.clear();
    if !should_refresh {
        return;
    }

    info!("updating display...");

    // set up random noise
    let mut rng = ChaChaRng::seed_from_u64(seed.0);
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
