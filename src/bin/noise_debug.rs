//! Draw the outputs of a noise function for debugging

use clap::{Parser, ValueEnum};
use indicatif::ProgressIterator;
use noise::{NoiseFn, Perlin, ScalePoint, Simplex};
use rand::{thread_rng, Rng};
use tiny_skia::{Pixmap, PremultipliedColorU8};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value_t = String::from("noise_debug.png"))]
    out: String,

    /// image width
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// image height
    #[arg(long, default_value_t = 600)]
    height: u32,

    #[arg(long, value_enum, default_value_t = NoiseType::Simplex)]
    noise_type: NoiseType,

    #[arg(long, default_value_t = 1.)]
    noise_scale: f64,

    /// normalize noise scale to size of image
    #[arg(long)]
    noise_norm: bool,
}

impl Args {
    fn get_noise_fn(&self, seed: u32) -> Box<dyn NoiseFn<f64, 2>> {
        let noise: Box<dyn NoiseFn<f64, 2>> = match self.noise_type {
            NoiseType::Perlin => Box::new(Perlin::new(seed)),
            NoiseType::Simplex => Box::new(Simplex::new(seed)),
        };
        let mut scale = self.noise_scale;
        if self.noise_norm {
            scale /= self.width.max(self.height) as f64;
        }
        let noise = ScalePoint::new(noise).set_scale(scale);
        Box::new(noise)
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum NoiseType {
    Simplex,
    Perlin,
}

pub fn main() {
    let args = Args::parse();
    let mut rng = thread_rng();
    let noise = args.get_noise_fn(rng.gen());
    let mut pixmap = Pixmap::new(args.width, args.height).unwrap();
    let pixels = pixmap.pixels_mut();

    for i in (0..(args.width * args.height)).progress() {
        let x = i % args.width;
        let y = i / args.width;
        let v = noise.get([x as f64, y as f64]);
        let rgb = ((v + 1.) / 2. * 256.).clamp(0., 255.) as u8;
        pixels[i as usize] = PremultipliedColorU8::from_rgba(rgb, rgb, rgb, 255).unwrap();
    }

    pixmap.save_png("noise_debug.png").unwrap();
}
