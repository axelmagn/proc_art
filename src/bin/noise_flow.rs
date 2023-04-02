extern crate nalgebra as na;
use clap::Parser;
use indicatif::ProgressIterator;
use na::Vector2;
use noise::{NoiseFn, Simplex};
use palette::{Gradient, LinSrgb};
use rand::{distributions::Uniform, prelude::Distribution, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// output path
    #[arg(short, long, default_value_t = String::from("noise_flow.png"))]
    out: String,

    /// image width
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// image height
    #[arg(long, default_value_t = 600)]
    height: u32,

    /// random seed
    #[arg(long)]
    seed: Option<u64>,

    #[arg(long, default_value_t = 0.4)]
    bias_x: f64,

    #[arg(long, default_value_t = 0.3)]
    bias_y: f64,

    /// noise scale
    #[arg(long, default_value_t = 100.)]
    scale: f64,

    #[arg(long, default_value_t = false)]
    draw_flow_tails: bool,

    #[arg(long, default_value_t = true)]
    draw_flow_walks: bool,

    #[arg(long, default_value_t = 2000)]
    flow_walk_n: u32,

    #[arg(long, default_value_t = 1000)]
    flow_walk_steps: u32,

    #[arg(long, default_value_t = 4.)]
    flow_walk_step_size: f64,

    #[arg(long, default_value_t = 10.)]
    color_scale: f64,

    #[arg(long, default_value_t = 48.)]
    color_range: f64,
}

struct Noise2x2 {
    /// position scale.  Coordinates are multiplied by this value before being passed to noise functions.
    pub pos_scale: f64,
    pub normalize: bool,
    pub bias: Vector2<f64>,
    noise_x: Box<dyn NoiseFn<f64, 2>>,
    noise_y: Box<dyn NoiseFn<f64, 2>>,
}

impl Noise2x2 {
    fn new(rng: &mut impl Rng) -> Self {
        Noise2x2 {
            pos_scale: 1.,
            normalize: false,
            bias: Vector2::zeros(),
            noise_x: Box::new(Simplex::new(rng.gen())),
            noise_y: Box::new(Simplex::new(rng.gen())),
        }
    }

    fn sample(&self, pos: &Vector2<f64>) -> Vector2<f64> {
        let pos = pos / self.pos_scale;
        let mut out = Vector2::new(
            self.noise_x.get([pos.x, pos.y]),
            self.noise_y.get([pos.x, pos.y]),
        );
        out += self.bias;
        if self.normalize && out.norm() > 0. {
            out.normalize_mut();
        }
        out
    }
}

pub fn main() {
    let args = Args::parse();

    // set up canvas
    let mut pixmap = Pixmap::new(args.width, args.height).unwrap();
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    // set up RNG
    let mut rng = match args.seed {
        Some(n) => ChaCha8Rng::seed_from_u64(n),
        _ => ChaCha8Rng::from_entropy(),
    };

    // set up flow noise
    let mut flow_noise = Noise2x2::new(&mut rng);
    flow_noise.pos_scale = args.scale;
    flow_noise.normalize = true;
    // todo: args
    flow_noise.bias = Vector2::new(0.4, 0.4);

    // draw flow tails
    // todo: arg gate
    if args.draw_flow_tails {
        // tail grid parameters
        let stride: f64 = 32.;
        let tail_len: f64 = 16.;
        // set up paint
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 255, 255);
        paint.anti_alias = true;

        let transform = Transform::identity();

        let mut stroke = Stroke::default();
        stroke.width = 1.0;

        // draw flow tails
        let mut draw_tail = |pos: Vector2<f64>, dir: Vector2<f64>| {
            // source circle
            let p_circle =
                PathBuilder::from_circle(pos.x as f32, pos.y as f32, tail_len as f32 / 8.).unwrap();

            // tail
            let dst = pos + dir * tail_len;
            assert!(pos != Vector2::zeros());
            assert!(dst != Vector2::zeros());
            assert!((pos - dst).norm() - tail_len <= 0.001);
            let mut pb_line = PathBuilder::new();
            pb_line.move_to(pos.x as f32, pos.y as f32);
            pb_line.line_to(dst.x as f32, dst.y as f32);
            let p_line = pb_line.finish().unwrap();

            pixmap.stroke_path(&p_circle, &paint, &stroke, transform, None);
            pixmap.stroke_path(&p_line, &paint, &stroke, transform, None);
        };

        for i in 1..(args.width / stride as u32) {
            for j in 1..(args.height / stride as u32) {
                let pos = Vector2::new(i as f64 * stride, j as f64 * stride);
                let dir = flow_noise.sample(&pos);
                draw_tail(pos, dir);
            }
        }
    }

    // draw flow walks
    // todo: arg gate
    if args.draw_flow_walks {
        let n_walks = args.flow_walk_n;
        let walk_steps = args.flow_walk_steps;
        let step_size = args.flow_walk_step_size;

        let gradient = Gradient::new(vec![
            LinSrgb::new(0.00, 0.05, 0.20),
            LinSrgb::new(0.70, 0.10, 0.20),
            LinSrgb::new(0.95, 0.90, 0.30),
        ]);

        let taken_colors: Vec<_> = gradient.take(10).collect();
        let color_noise = Simplex::new(rng.gen());

        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;
        let transform = Transform::identity();
        let mut stroke = Stroke::default();
        stroke.width = 2.0;

        let mut draw_walk = |pos: &Vector2<f64>, color: Color| {
            // path
            let mut pb = PathBuilder::new();
            pb.move_to(pos.x as f32, pos.y as f32);
            // cursor
            let mut x = *pos;
            for _i in 0..walk_steps {
                let mut dx = flow_noise.sample(&x);
                let x2 = x + dx * step_size;
                dx = flow_noise.sample(&x2);
                let x3 = x2 + dx * step_size;
                dx = flow_noise.sample(&x3);
                let x4 = x3 + dx * step_size;
                pb.cubic_to(
                    x2.x as f32,
                    x2.y as f32,
                    x3.x as f32,
                    x3.y as f32,
                    x4.x as f32,
                    x4.y as f32,
                );
                x = x4;
            }
            let path = pb.finish().unwrap();
            paint.set_color(color);
            pixmap.stroke_path(&path, &paint, &stroke, transform, None)
        };

        let x_range = Uniform::new(0., args.width as f64);
        let y_range = Uniform::new(0., args.height as f64);
        // let color_range = Uniform::new(0, 10);
        for _i in (0..n_walks).progress() {
            let p = Vector2::new(x_range.sample(&mut rng), y_range.sample(&mut rng));
            let color_scale = args.scale * args.color_scale;
            let color_range = args.color_range;
            let color_i = ((color_noise.get([p.x / color_scale, p.y / color_scale]) * color_range)
                as usize)
                .clamp(0, 9);
            // println!("color_i: {}", color_i);
            let color = taken_colors[color_i];
            let r = (color.red * 255.) as u8;
            let g = (color.green * 255.) as u8;
            let b = (color.blue * 255.) as u8;
            let skia_color: Color = Color::from_rgba8(r, g, b, 255);
            draw_walk(&p, skia_color);
        }
    }

    // save result
    pixmap.save_png(args.out).unwrap();
}
