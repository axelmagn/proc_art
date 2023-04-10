use clap::Parser;
use image::{Rgb, RgbImage};
use noise::{NoiseFn, Perlin};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Program to illustrate perlin noise flow
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// output path
    #[arg(short, long, default_value_t = String::from("featherweight.png"))]
    out: String,

    #[arg(long, default_value_t = 1024)]
    size: u32,

    #[arg(long, default_value_t = 10.)]
    scale: f64,

    #[arg(long)]
    seed: Option<u64>,

    /// Draw a visualization of flow in the background.
    #[arg(long)]
    draw_flow_bg: bool,

    // Draw a visualization of flow as tails
    #[arg(long)]
    draw_flow_tails: bool,

    /// how many flow tails to draw in a row across the image
    #[arg(long, default_value_t = 20)]
    flow_tail_freq: u32,

    #[arg(long, default_value_t = 50)]
    flow_tail_length: u32,

    #[arg(long, default_value_t = true)]
    draw_flow_walks: bool,

    #[arg(long, default_value_t = 1000)]
    flow_walk_freq: u32,

    #[arg(long, default_value_t = 1000)]
    flow_walk_length: u32,

    #[arg(long)]
    flow_walk_norm: bool,
}

pub fn main() {
    // parse args
    let args = Args::parse();

    // create image buffer
    let mut img = RgbImage::new(args.size, args.size);

    // generate flow directions from perlin noise
    // todo(axelmagn): seed from arg
    let mut rng = match args.seed {
        Some(n) => ChaCha8Rng::seed_from_u64(n),
        _ => ChaCha8Rng::from_entropy(),
    };
    let flow_x = Perlin::new(rng.gen());
    let flow_y = Perlin::new(rng.gen());

    // draw flow background
    if args.draw_flow_bg {
        for x in 0..args.size {
            for y in 0..args.size {
                let fx = x as f64 / args.size as f64 * args.scale;
                let fy = y as f64 / args.size as f64 * args.scale;
                let red = flow_x.get([fx, fy]);
                let red = ((red + 1.) / 2. * 255.) as u8;
                let blue = flow_y.get([fx, fy]);
                let blue = ((blue + 1.) / 2. * 255.) as u8;
                img.put_pixel(x, y, Rgb([red, blue, 0]));
            }
        }
    }

    // draw flow tails
    if args.draw_flow_tails {
        let stride = (args.size / args.flow_tail_freq) as usize;
        let tail_len = args.flow_tail_length;
        for x in (0..args.size).step_by(stride) {
            for y in (0..args.size).step_by(stride) {
                // floating point coords
                let mut fx = x as f64;
                let mut fy = y as f64;
                // noise coords
                let nx = fx / args.size as f64 * args.scale;
                let ny = fy / args.size as f64 * args.scale;
                // get flow vector at this point
                let vx = flow_x.get([nx, ny]);
                let vy = flow_y.get([nx, ny]);
                let red = ((vx + 1.) / 2. * 255.) as u8;
                let blue = ((vy + 1.) / 2. * 255.) as u8;
                let tail_color = Rgb([red, blue, 0]);
                for _i in 0..tail_len {
                    if fx < 0. || fx < 0. || fx >= args.size as f64 || fy >= args.size as f64 {
                        break;
                    }
                    img.put_pixel(fx as u32, fy as u32, tail_color);
                    fx += vx;
                    fy += vy;
                }
            }
        }
    }

    if args.draw_flow_walks {
        let walk_color = Rgb([255, 255, 255]);
        let walk_len = args.flow_walk_length;
        let between = Uniform::new(0., args.size as f64);
        for _i in 0..args.flow_walk_freq {
            // floating point coords
            let mut fx: f64 = between.sample(&mut rng);
            let mut fy: f64 = between.sample(&mut rng);
            for _i in 0..walk_len {
                if fx < 0. || fx < 0. || fx >= args.size as f64 || fy >= args.size as f64 {
                    break;
                }
                img.put_pixel(fx as u32, fy as u32, walk_color);

                // noise coords
                let nx = fx / args.size as f64 * args.scale;
                let ny = fy / args.size as f64 * args.scale;
                // get flow vector at this point
                let mut vx = flow_x.get([nx, ny]);
                let mut vy = flow_y.get([nx, ny]);
                // normalize velocity (optional)
                if args.flow_walk_norm {
                    let norm = (vx * vx + vy * vy).sqrt();
                    if norm > 0. {
                        vx /= norm;
                        vy /= norm;
                    }
                }
                fx += vx;
                fy += vy;
            }
        }
    }

    // save image
    img.save(args.out).unwrap();
}
