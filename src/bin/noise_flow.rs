extern crate nalgebra as na;
use clap::Parser;
use na::Vector2;
use noise::{NoiseFn, Simplex};
use rand::{Rng, SeedableRng};
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

    /// noise scale
    #[arg(long, default_value_t = 100.)]
    scale: f64,
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

    // draw flow tails
    {
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

        // save result
        pixmap.save_png(args.out).unwrap();
    }
}
