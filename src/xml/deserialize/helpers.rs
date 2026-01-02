//! Common utilities for XML deserialization.
//!
//! This module provides helper functions for:
//! - Reading text and numeric content from XML elements
//! - Parsing common attribute types (numbers, strings, enums)
//! - Parsing style-related enums (colors, fonts, borders, etc.)

use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::{
    view::style::{
        BorderStyle, BorderWidth, Color, FontStyle, FontWeight, Padding, PredefinedColor,
        TextAlign, TextDecoration, VerticalTextAlign,
    },
    xml::deserialize::DeserializeError,
};

/// Helper to read text content from an element.
pub fn read_text_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<String, DeserializeError> {
    let mut text = String::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Text(e) => {
                text.push_str(&e.unescape()?);
            }
            Event::CData(e) => {
                // CData content doesn't need unescaping, just convert to string
                text.push_str(&String::from_utf8_lossy(e.as_ref()));
            }
            Event::End(_) => break,
            Event::Eof => return Err(DeserializeError::UnexpectedEof),
            _ => {}
        }
        buf.clear();
    }

    Ok(text)
}

/// Helper to read a numeric value from an element.
pub fn read_number_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<f64, DeserializeError> {
    let text = read_text_content(reader, buf)?;
    text.parse::<f64>()
        .map_err(|e| DeserializeError::Custom(format!("Invalid number: {}", e)))
}

/// Helper to read an optional element (returns None if element is not present or empty).
pub fn read_optional_text<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    element_name: &str,
) -> Result<Option<String>, DeserializeError> {
    match reader.read_event_into(buf)? {
        Event::Start(e) if e.name().as_ref() == element_name.as_bytes() => {
            let text = read_text_content(reader, buf)?;
            Ok(Some(text))
        }
        Event::Empty(e) if e.name().as_ref() == element_name.as_bytes() => Ok(None),
        _ => {
            // Element not found, rewind
            // Note: quick-xml doesn't support rewinding, so we need to handle this differently
            // For now, return None if element is not the expected one
            Ok(None)
        }
    }
}

/// Parse a numeric attribute value from a string.
///
/// This is a common pattern used throughout deserialization for parsing
/// attributes like uid, x, y, width, height, etc.
pub fn parse_numeric_attr<T: FromStr>(value: &str, attr_name: &str) -> Result<T, DeserializeError>
where
    T::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|e| DeserializeError::Custom(format!("Invalid {} value: {}", attr_name, e)))
}

/// Parse a boolean attribute value from a string.
pub fn parse_bool_attr(value: &str, attr_name: &str) -> Result<bool, DeserializeError> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid {} value: expected true/false, got {}",
            attr_name, value
        ))),
    }
}

// ============================================================================
// Style parsing functions (consolidated from duplicate implementations)
// ============================================================================

/// Parse Color from string.
///
/// Handles both hex colors (#RRGGBB) and predefined color names.
/// Falls back to hex if the color name is not recognized.
pub fn parse_color(s: &str) -> Result<Color, DeserializeError> {
    if s.starts_with('#') {
        Ok(Color::Hex(s.to_string()))
    } else {
        // Try to parse as predefined color
        let predefined = match s.to_lowercase().as_str() {
            "aqua" => PredefinedColor::Aqua,
            "black" => PredefinedColor::Black,
            "blue" => PredefinedColor::Blue,
            "fuchsia" => PredefinedColor::Fuchsia,
            "gray" => PredefinedColor::Gray,
            "green" => PredefinedColor::Green,
            "lime" => PredefinedColor::Lime,
            "maroon" => PredefinedColor::Maroon,
            "navy" => PredefinedColor::Navy,
            "olive" => PredefinedColor::Olive,
            "purple" => PredefinedColor::Purple,
            "red" => PredefinedColor::Red,
            "silver" => PredefinedColor::Silver,
            "teal" => PredefinedColor::Teal,
            "white" => PredefinedColor::White,
            "yellow" => PredefinedColor::Yellow,
            _ => {
                // Fallback to hex for unknown colors (used in style deserialization)
                return Ok(Color::Hex(s.to_string()));
            }
        };
        Ok(Color::Predefined(predefined))
    }
}

/// Parse FontWeight from string.
pub fn parse_font_weight(s: &str) -> Result<FontWeight, DeserializeError> {
    match s.to_lowercase().as_str() {
        "normal" => Ok(FontWeight::Normal),
        "bold" => Ok(FontWeight::Bold),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid font_weight: {}",
            s
        ))),
    }
}

/// Parse FontStyle from string.
pub fn parse_font_style(s: &str) -> Result<FontStyle, DeserializeError> {
    match s.to_lowercase().as_str() {
        "normal" => Ok(FontStyle::Normal),
        "italic" => Ok(FontStyle::Italic),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid font_style: {}",
            s
        ))),
    }
}

/// Parse TextDecoration from string.
pub fn parse_text_decoration(s: &str) -> Result<TextDecoration, DeserializeError> {
    match s.to_lowercase().as_str() {
        "normal" => Ok(TextDecoration::Normal),
        "underline" => Ok(TextDecoration::Underline),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid text_decoration: {}",
            s
        ))),
    }
}

/// Parse TextAlign from string.
pub fn parse_text_align(s: &str) -> Result<TextAlign, DeserializeError> {
    match s.to_lowercase().as_str() {
        "left" => Ok(TextAlign::Left),
        "right" => Ok(TextAlign::Right),
        "center" => Ok(TextAlign::Center),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid text_align: {}",
            s
        ))),
    }
}

/// Parse VerticalTextAlign from string.
pub fn parse_vertical_text_align(s: &str) -> Result<VerticalTextAlign, DeserializeError> {
    match s.to_lowercase().as_str() {
        "top" => Ok(VerticalTextAlign::Top),
        "bottom" => Ok(VerticalTextAlign::Bottom),
        "center" => Ok(VerticalTextAlign::Center),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid vertical_text_align: {}",
            s
        ))),
    }
}

/// Parse BorderWidth from string.
pub fn parse_border_width(s: &str) -> Result<BorderWidth, DeserializeError> {
    match s {
        "thick" => Ok(BorderWidth::Thick),
        "thin" => Ok(BorderWidth::Thin),
        _ => {
            // Try to parse as number
            let px = s
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid border_width: {}", e)))?;
            Ok(BorderWidth::Px(px))
        }
    }
}

/// Parse BorderStyle from string.
pub fn parse_border_style(s: &str) -> Result<BorderStyle, DeserializeError> {
    match s {
        "none" => Ok(BorderStyle::None),
        "solid" => Ok(BorderStyle::Solid),
        _ => Err(DeserializeError::Custom(format!(
            "Invalid border_style: {}",
            s
        ))),
    }
}

/// Parse text padding from string.
///
/// Returns a tuple of (top, right, bottom, left) as Option<f64>.
/// Supports 1-4 values (CSS-style padding).
pub fn parse_text_padding(
    s: &str,
) -> Result<(Option<f64>, Option<f64>, Option<f64>, Option<f64>), DeserializeError> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    match parts.len() {
        1 => {
            let val = parts[0]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            Ok((Some(val), None, None, None))
        }
        2 => {
            let top = parts[0]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let right = parts[1]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            Ok((Some(top), Some(right), None, None))
        }
        3 => {
            let top = parts[0]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let right = parts[1]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let bottom = parts[2]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            Ok((Some(top), Some(right), Some(bottom), None))
        }
        4 => {
            let top = parts[0]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let right = parts[1]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let bottom = parts[2]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            let left = parts[3]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding value: {}", e)))?;
            Ok((Some(top), Some(right), Some(bottom), Some(left)))
        }
        _ => Err(DeserializeError::Custom(format!(
            "Invalid padding format: {}",
            s
        ))),
    }
}

/// Parse Padding from string (used in style deserialization).
///
/// Returns a Padding struct with at least a top value required.
pub fn parse_padding(s: &str) -> Result<Padding, DeserializeError> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err(DeserializeError::Custom(
            "Padding must have at least one value".to_string(),
        ));
    }

    let top = parts[0]
        .parse::<f64>()
        .map_err(|e| DeserializeError::Custom(format!("Invalid padding top: {}", e)))?;
    let right = if parts.len() > 1 {
        Some(
            parts[1]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding right: {}", e)))?,
        )
    } else {
        None
    };
    let bottom = if parts.len() > 2 {
        Some(
            parts[2]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding bottom: {}", e)))?,
        )
    } else {
        None
    };
    let left = if parts.len() > 3 {
        Some(
            parts[3]
                .parse::<f64>()
                .map_err(|e| DeserializeError::Custom(format!("Invalid padding left: {}", e)))?,
        )
    } else {
        None
    };

    Ok(Padding {
        top,
        right,
        bottom,
        left,
    })
}
