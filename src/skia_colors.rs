//! Utility library for working with files

use std::num::ParseIntError;

use tiny_skia::Color;

const DEFAULT_PALETTE: &'static str = include_str!("../assets/colors/ocaso.hex");

#[derive(Debug, PartialEq)]
pub enum ParseHexColorError {
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

pub fn parse_hex_palette(s: &str) -> Result<Vec<Color>, ParseHexColorError> {
    s.lines().map(parse_hex_color).collect()
}

pub fn parse_hex_color(s: &str) -> Result<Color, ParseHexColorError> {
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

pub fn get_default_palette() -> Vec<Color> {
    parse_hex_palette(DEFAULT_PALETTE).unwrap()
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

    #[test]
    fn test_get_default_palette() {
        let palette = get_default_palette();
        assert!(palette.len() > 0);
    }
}
