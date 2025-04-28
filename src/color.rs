use serde::Deserialize;
use serde::Serialize;
extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};

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
    pub fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates a new color from RGB values (assumes alpha is 255).
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    /// Creates a new color from a hex string.
    /// The string can be in the formats: "#RGB", "#RGBA", "#RRGGBB", "#RRGGBBAA", "RGB", "RGBA", "RRGGBB", or "RRGGBBAA" (case-insensitive). Any other format will fail.
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        if hex.len() == 0 {
            return Err(ColorError::EmptyHexString());
        }

        let trimmed_hex = hex.trim_start_matches('#');

        match trimmed_hex.len() {
            0 => return Err(ColorError::EmptyTrimmedHexString()),
            3 => {
                let red = u8::from_str_radix(&trimmed_hex[0..1], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[0..1].to_owned(),
                    )
                })? * 17; // multiply by 17 to turn value of single hex digit to the value of if it was repeated, ex: "f" -> "ff"
                let green = u8::from_str_radix(&trimmed_hex[1..2], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[1..2].to_owned(),
                    )
                })? * 17;
                let blue = u8::from_str_radix(&trimmed_hex[2..3], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[2..3].to_owned(),
                    )
                })? * 17;
                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha: 255,
                })
            }
            4 => {
                let red = u8::from_str_radix(&trimmed_hex[0..1], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[0..1].to_owned(),
                    )
                })? * 17;
                let green = u8::from_str_radix(&trimmed_hex[1..2], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[1..2].to_owned(),
                    )
                })? * 17;
                let blue = u8::from_str_radix(&trimmed_hex[2..3], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[2..3].to_owned(),
                    )
                })? * 17;
                let alpha = u8::from_str_radix(&trimmed_hex[3..4], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[3..4].to_owned(),
                    )
                })? * 17;
                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha,
                })
            }
            6 => {
                let red = u8::from_str_radix(&trimmed_hex[0..2], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[0..2].to_owned(),
                    )
                })?;
                let green = u8::from_str_radix(&trimmed_hex[2..4], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[2..4].to_owned(),
                    )
                })?;
                let blue = u8::from_str_radix(&trimmed_hex[4..6], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[4..6].to_owned(),
                    )
                })?;
                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha: 255,
                })
            }
            8 => {
                let red = u8::from_str_radix(&trimmed_hex[0..2], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[0..2].to_owned(),
                    )
                })?;
                let green = u8::from_str_radix(&trimmed_hex[2..4], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[2..4].to_owned(),
                    )
                })?;
                let blue = u8::from_str_radix(&trimmed_hex[4..6], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[4..6].to_owned(),
                    )
                })?;
                let alpha = u8::from_str_radix(&trimmed_hex[6..8], 16).map_err(|_| {
                    ColorError::InvalidHexStringValue(
                        format!("#{}", trimmed_hex),
                        trimmed_hex[6..8].to_owned(),
                    )
                })?;
                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha,
                })
            }
            _ => Err(ColorError::InvalidHexStringLength(
                trimmed_hex.to_owned(),
                trimmed_hex.len(),
            )),
        }
    }

    /// Converts the color to a hex string.
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue, self.alpha
        )
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
}

#[cfg(feature = "std")]
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
            red,
            green,
            blue,
            alpha,
        }
    }
}
