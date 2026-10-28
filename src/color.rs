use serde::Deserialize;
use serde::Serialize;
use std::{borrow::ToOwned, format, string::String};

use crate::ColorError;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Color {
    pub alpha: u8,
    pub blue: u8,
    pub green: u8,
    pub red: u8,
}

impl Color {
    /// Creates a new color from RGBA values.
    #[must_use]
    pub fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            alpha,
            blue,
            green,
            red,
        }
    }

    /// Creates a new color from RGB values (assumes alpha is 255).
    #[must_use]
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    /// Creates a new color from a hex string.
    ///
    /// The string can be in the formats: "#RGB", "#RGBA", "#RRGGBB",
    /// "#RRGGBBAA", "RGB", "RGBA", "RRGGBB", or "RRGGBBAA"
    /// (case-insensitive). Any other format will fail.
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        if hex.is_empty() {
            return Err(ColorError::EmptyHexString());
        }

        let trimmed_hex = hex.trim_start_matches('#');

        match trimmed_hex.len() {
            0 => Err(ColorError::EmptyTrimmedHexString()),
            3 => Self::parse_short_hex(trimmed_hex, false),
            4 => Self::parse_short_hex(trimmed_hex, true),
            6 => Self::parse_long_hex(trimmed_hex, false),
            8 => Self::parse_long_hex(trimmed_hex, true),
            _ => Err(ColorError::InvalidHexStringLength(
                trimmed_hex.to_owned(),
                trimmed_hex.len(),
            )),
        }
    }

    /// Parses RGB or RGBA hex strings with one digit per channel.
    fn parse_short_hex(hex: &str, has_alpha: bool) -> Result<Self, ColorError> {
        let red = Self::parse_hex_channel(hex, 0, 1)? * 17;
        let green = Self::parse_hex_channel(hex, 1, 2)? * 17;
        let blue = Self::parse_hex_channel(hex, 2, 3)? * 17;

        let alpha = if has_alpha {
            Self::parse_hex_channel(hex, 3, 4)? * 17
        } else {
            255
        };

        Ok(Color {
            red,
            green,
            blue,
            alpha,
        })
    }

    /// Parses RGB or RGBA hex strings with two digits per channel.
    fn parse_long_hex(hex: &str, has_alpha: bool) -> Result<Self, ColorError> {
        let red = Self::parse_hex_channel(hex, 0, 2)?;
        let green = Self::parse_hex_channel(hex, 2, 4)?;
        let blue = Self::parse_hex_channel(hex, 4, 6)?;

        let alpha = if has_alpha {
            Self::parse_hex_channel(hex, 6, 8)?
        } else {
            255
        };

        Ok(Color {
            red,
            green,
            blue,
            alpha,
        })
    }

    /// Parses a hex channel and preserves the original error format.
    fn parse_hex_channel(
        hex: &str,
        start: usize,
        end: usize,
    ) -> Result<u8, ColorError> {
        let channel = &hex[start..end];

        u8::from_str_radix(channel, 16).map_err(|_| {
            ColorError::InvalidHexStringValue(
                format!("#{hex}"),
                channel.to_owned(),
            )
        })
    }

    /// Converts the color to a hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue, self.alpha
        )
    }

    #[must_use]
    pub fn to_rgba_f32(self) -> [f32; 4] {
        self.into()
    }

    #[must_use]
    pub fn from_rgba_f32(arr: [f32; 4]) -> Self {
        arr.into()
    }
}

/// Common color constants
impl Color {
    pub const RED: Self = Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    pub const GREEN: Self = Color {
        red: 0,
        green: 255,
        blue: 0,
        alpha: 255,
    };
    pub const BLUE: Self = Color {
        red: 0,
        green: 0,
        blue: 255,
        alpha: 255,
    };
    pub const WHITE: Self = Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };
    pub const BLACK: Self = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    pub const DEFAULT: Self = Color {
        red: 32,
        green: 32,
        blue: 32,
        alpha: 255,
    };
    pub const DEFAULT_GRADIENT: Self = Color {
        red: 128,
        green: 128,
        blue: 128,
        alpha: 255,
    };
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgba({}, {}, {}, {})",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Color::from_rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Color::from_rgba(rgba.0, rgba.1, rgba.2, rgba.3)
    }
}

impl From<&str> for Color {
    fn from(hex: &str) -> Self {
        Color::from_hex(hex).unwrap_or_default()
    }
}

impl From<u32> for Color {
    fn from(hex: u32) -> Self {
        let red = ((hex >> 24) & 0xFF) as u8;
        let green = ((hex >> 16) & 0xFF) as u8;
        let blue = ((hex >> 8) & 0xFF) as u8;
        let alpha = (hex & 0xFF) as u8;
        Color {
            alpha,
            blue,
            green,
            red,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [
            f32::from(c.red) / 255.0,
            f32::from(c.green) / 255.0,
            f32::from(c.blue) / 255.0,
            f32::from(c.alpha) / 255.0,
        ]
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        // Allow justified because casting is necesary for this color format conversion
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        fn to_u8(v: f32) -> u8 {
            (v.clamp(0.0, 1.0) * 255.0).round() as u8
        }

        Color {
            red: to_u8(arr[0]),
            green: to_u8(arr[1]),
            blue: to_u8(arr[2]),
            alpha: to_u8(arr[3]),
        }
    }
}
