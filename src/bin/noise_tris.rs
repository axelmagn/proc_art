//! draw a grid of triangles, with colors derived from a noise function

use std::{fs, num::ParseIntError};

use clap::Parser;
use noise::{NoiseFn, Simplex, Worley};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng, RngCore};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Point, Transform};

const DEFAULT_PALETTE: &'static str = include_str!("../../assets/colors/golden-haze.hex");

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// output path
    #[arg(short, long, default_value_t = String::from("noise_tris.png"))]
    out: String,

    /// image width
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// image height
    #[arg(long, default_value_t = 600)]
    height: u32,

    #[arg(long, default_value_t = 32.)]
    triangle_size: f32,

    #[arg(long)]
    palette_file: Option<String>,
}

impl Args {
    fn load_palette(&self) -> Result<Vec<Color>, ParseHexColorError> {
        let contents = match &self.palette_file {
            Some(path) => fs::read_to_string(path).expect("could not read palette file"),
            None => String::from(DEFAULT_PALETTE),
        };
        parse_hex_palette(&contents)
    }

    fn get_height_fn<R: Rng>(&self, rng: &mut R) -> Box<dyn NoiseFn<f64, 2>> {
        // TODO: parameterize
        Box::new(Simplex::new(rng.gen()))
    }
}

fn main() {
    let args = Args::parse();
    let pixmap = paint_main(&args);
    pixmap.save_png(args.out).unwrap();
}

// struct PaintTask {}

// impl PaintTask {}

struct NoiseData {
    // TODO: flow
    // flow_x: Box<dyn NoiseFn<f32, 2>>,
    // flow_y: Box<dyn NoiseFn<f32, 2>>,
    height: Box<dyn NoiseFn<f64, 2>>,
}

fn paint_main(args: &Args) -> Pixmap {
    let triangle_side = args.triangle_size;
    let triangle_half_side = triangle_side / 2.;
    let triangle_height = triangle_side * (60_f32).to_radians().sin();
    let triangle_half_height = triangle_height / 2.;
    let palette = args.load_palette().expect("could not load palette");

    let mut rng = thread_rng();
    let color_range = Uniform::new(0, palette.len());
    let noise_data = NoiseData {
        height: args.get_height_fn(&mut rng),
    };

    let mut pixmap = Pixmap::new(800, 600).unwrap();

    let i_max = (800. / triangle_side) as u32 + 1;
    let j_max = (600. / triangle_height) as u32 + 1;
    for i in 0..i_max {
        for j in 0..j_max {
            let mut x = i as f32 * triangle_side;
            if j % 2 == 0 {
                x -= triangle_half_side;
            }
            let y = j as f32 * triangle_height;
            let pos = Point::from_xy(x, y);

            let mut paint = Paint::default();
            paint.anti_alias = true;

            let sample_x = (x + triangle_half_side) as f64;
            let sample_y = (y + triangle_half_height) as f64;
            let height =
                (noise_data.height.get([sample_x, sample_y]) + 1.) / 2. * palette.len() as f64;
            let color = palette[height as usize];
            paint.set_color(color);
            draw_top_triangle(pos, triangle_side, &paint, &mut pixmap);

            let sample_x = x as f64;
            let sample_y = (y + triangle_half_height) as f64;
            let height =
                (noise_data.height.get([sample_x, sample_y]) + 1.) / 2. * palette.len() as f64;
            let color = palette[height as usize];
            paint.set_color(color);
            draw_bottom_triangle(pos, triangle_side, &paint, &mut pixmap);
        }
    }

    pixmap
}

fn draw_top_triangle(pos: Point, triangle_side: f32, paint: &Paint, pixmap: &mut Pixmap) {
    let triangle_half_side = triangle_side / 2.;
    let triangle_height = triangle_side * (60_f32).to_radians().sin();
    let points = [
        pos,
        Point::from_xy(pos.x + triangle_side, pos.y),
        Point::from_xy(pos.x + triangle_half_side, pos.y + triangle_height),
    ];
    draw_triangle(&points, paint, pixmap)
}

fn draw_bottom_triangle(pos: Point, triangle_side: f32, paint: &Paint, pixmap: &mut Pixmap) {
    let triangle_half_side = triangle_side / 2.;
    let triangle_height = triangle_side * (60_f32).to_radians().sin();
    let points = [
        pos,
        Point::from_xy(pos.x + triangle_half_side, pos.y + triangle_height),
        Point::from_xy(pos.x - triangle_half_side, pos.y + triangle_height),
    ];
    draw_triangle(&points, paint, pixmap)
}

fn draw_triangle(points: &[Point; 3], paint: &Paint, pixmap: &mut Pixmap) {
    let mut pb = PathBuilder::new();
    pb.move_to(points[0].x, points[0].y);
    pb.line_to(points[1].x, points[1].y);
    pb.line_to(points[2].x, points[2].y);
    pb.close();
    let path = pb.finish().unwrap();
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[derive(Debug, PartialEq)]
enum ParseHexColorError {
    WrongColorStringLength {
        input_str: String,
        actual_length: usize,
        expected_length: usize,
    },
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseHexColorError {
    fn from(value: ParseIntError) -> Self {
        ParseHexColorError::ParseIntError(value)
    }
}

fn parse_hex_palette(s: &str) -> Result<Vec<Color>, ParseHexColorError> {
    s.lines().map(parse_hex_color).collect()
}

fn parse_hex_color(s: &str) -> Result<Color, ParseHexColorError> {
    if s.len() != 6 {
        return Err(ParseHexColorError::WrongColorStringLength {
            input_str: String::from(s),
            actual_length: s.len(),
            expected_length: 6,
        });
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    Ok(Color::from_rgba8(r, g, b, 255))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_palette() {
        let palette_str = r"000000
ff0000
00ff00
0000ff
ffffff";

        let expected_colors = vec![
            Color::from_rgba8(0, 0, 0, 255),
            Color::from_rgba8(255, 0, 0, 255),
            Color::from_rgba8(0, 255, 0, 255),
            Color::from_rgba8(0, 0, 255, 255),
            Color::from_rgba8(255, 255, 255, 255),
        ];

        assert_eq!(parse_hex_palette(palette_str), Ok(expected_colors));
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(
            parse_hex_color("000000"),
            Ok(Color::from_rgba8(0, 0, 0, 255))
        );
        assert_eq!(
            parse_hex_color("FF0000"),
            Ok(Color::from_rgba8(255, 0, 0, 255))
        );
    }
}
