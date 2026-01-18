use serde::{Deserialize, Serialize};

use crate::types::{Validate, ValidationResult};

// All XMILE objects MAY have explicit ranges and scales that are used by default in input and output devices, respectively. These same properties can appear within the input and output devices to override the entity’s setting for that device.
//
// The <range> tag is used to specify the default input range for an input device.  Without it, any reasonable guess can be used (typically tied to the variable’s scale). The <scale> tag is used to specify the global scale of a variable. Without it, the scale of variable matches the (output) range of its values.  Both tags have two attributes:
//
//     Range/scale minimum:  min="…" with the minimum value for the range/scale
//     Range/scale maximum:  max="…" with the maximum value for the range/scale
//
// Note that it is REQUIRED that min <= max. For the <scale> tag only, two OPTIONAL attributes exist that can only be used when the <scale> tag appears within the definition of an output device (typically graphs):
//
//     Autoscale:  auto="…" with true/false to override the global scale within that output device; this is mutually exclusive with min, max, and group (default: false)
//     Autoscale group:  group="…" with a unique number identifying the group in that output device; note this implies auto="true" and is therefore mutually exclusive with min, max, and auto (default: not in a group)
//
// Groups require more than one variable in them and are specifically used to autoscale a group of variables to the same scale starting at the minimum value of all variables in the group and ending at the maximum value of all variables in the group. This is the default scaling for all variables in a comparative plot, so does not need to appear in that case.
//
// The <format> tag allows default formatting to be set for values of each variable.  Without it, the default settings for each attribute below takes effect:
//
//     Precision:  precision="…" with value indicating precision of least significant digit, e.g., “0.01” to round to the hundredths place or “0.5” to round to the nearest half (default: best guess based on the scale of the variable)
//     Magnitude scale:  scale_by="…" with the factor to scale all values by before displaying, e.g., “1000” to display thousands (default: no scaling, i.e., 1); note the precision applies to the scaled number
//     Special symbols:  display_as="…" with “number”, “currency”, or “percent” (default: number)
//     Include thousands separator:  delimit_000s="…" with true/false (default: false)
//
// These can also be overridden, using the same attribute names, in variable definitions of individual input or output devices.
pub trait Object {
    fn range(&self) -> Option<&DeviceRange>;
    fn scale(&self) -> Option<&DeviceScale>;
    fn format(&self) -> Option<&FormatOptions>;
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceRange {
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
}

impl DeviceRange {
    pub fn new(min: f64, max: f64) -> Self {
        DeviceRange { min, max }
    }
}

impl Validate for DeviceRange {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();
        if self.min > self.max {
            errors.push("Range minimum cannot be greater than maximum.".to_string());
        }
        if self.min.is_nan() || self.max.is_nan() {
            errors.push("Range values cannot be NaN.".to_string());
        }
        if self.min.is_infinite() || self.max.is_infinite() {
            errors.push("Range values cannot be infinite.".to_string());
        }
        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayAs {
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "currency")]
    Currency,
    #[serde(rename = "percent")]
    Percent,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeviceScale {
    MinMax { min: f64, max: f64 },
    Auto(bool),
    Group(u32),
}

impl DeviceScale {
    pub fn new(min: f64, max: f64) -> Self {
        DeviceScale::MinMax { min, max }
    }

    pub fn auto(auto: bool) -> Self {
        DeviceScale::Auto(auto)
    }

    pub fn group(group: u32) -> Self {
        DeviceScale::Group(group)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RawDeviceScale {
    #[serde(rename = "@min")]
    min: Option<f64>,
    #[serde(rename = "@max")]
    max: Option<f64>,
    #[serde(rename = "@auto")]
    auto: Option<bool>,
    #[serde(rename = "@group")]
    group: Option<u32>,
}

impl From<DeviceScale> for RawDeviceScale {
    fn from(scale: DeviceScale) -> Self {
        match scale {
            DeviceScale::MinMax { min, max } => RawDeviceScale {
                min: Some(min),
                max: Some(max),
                auto: None,
                group: None,
            },
            DeviceScale::Auto(auto) => RawDeviceScale {
                min: None,
                max: None,
                auto: Some(auto),
                group: None,
            },
            DeviceScale::Group(group) => RawDeviceScale {
                min: None,
                max: None,
                auto: None,
                group: Some(group),
            },
        }
    }
}

impl<'de> Deserialize<'de> for DeviceScale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: RawDeviceScale = Deserialize::deserialize(deserializer)?;

        if let Some(auto) = raw.auto {
            if raw.min.is_some() || raw.max.is_some() || raw.group.is_some() {
                return Err(serde::de::Error::custom(
                    "DeviceScale: auto cannot be used with min, max, or group",
                ));
            }
            return Ok(DeviceScale::Auto(auto));
        }

        if let Some(group) = raw.group {
            if raw.min.is_some() || raw.max.is_some() || raw.auto.is_some() {
                return Err(serde::de::Error::custom(
                    "DeviceScale: group cannot be used with min, max, or auto",
                ));
            }
            return Ok(DeviceScale::Group(group));
        }

        match (raw.min, raw.max) {
            (Some(min), Some(max)) => {
                if min > max {
                    return Err(serde::de::Error::custom(
                        "DeviceScale: min cannot be greater than max",
                    ));
                }
                Ok(DeviceScale::MinMax { min, max })
            }
            _ => Err(serde::de::Error::custom(
                "DeviceScale: must specify min and max, auto, or group",
            )),
        }
    }
}

impl Serialize for DeviceScale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw: RawDeviceScale = (*self).into();
        raw.serialize(serializer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatOptions {
    #[serde(rename = "@precision")]
    pub precision: Option<f64>,
    #[serde(rename = "@scale_by")]
    pub scale_by: Option<f64>,
    #[serde(rename = "@display_as")]
    pub display_as: Option<DisplayAs>,
    #[serde(rename = "@delimit_000s")]
    pub delimit_000s: Option<bool>,
}

impl Validate for FormatOptions {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        if let Some(precision) = self.precision
            && precision.is_sign_negative()
        {
            errors.push("Precision cannot be negative.".to_string());
        }

        if let Some(scale_by) = self.scale_by
            && scale_by.is_sign_negative()
        {
            errors.push("Scale factor cannot be negative.".to_string());
        }

        if errors.is_empty() {
            ValidationResult::Valid(())
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Documentation {
    PlainText(String),
    Html(String),
}

impl<'de> Deserialize<'de> for Documentation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        if is_html_content(&s) {
            Ok(Documentation::Html(s))
        } else {
            Ok(Documentation::PlainText(s))
        }
    }
}

impl Serialize for Documentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Documentation::PlainText(text) => serializer.serialize_str(text),
            Documentation::Html(html) => serializer.serialize_str(html),
        }
    }
}

fn is_html_content(s: &str) -> bool {
    let trimmed = s.trim();

    // Must contain angle brackets
    if !trimmed.contains('<') || !trimmed.contains('>') {
        return false;
    }

    // Common HTML patterns
    let html_indicators = [
        // Common HTML tags
        "<p>",
        "</p>",
        "<div>",
        "</div>",
        "<span>",
        "</span>",
        "<br>",
        "<br/>",
        "<hr>",
        "<hr/>",
        "<strong>",
        "</strong>",
        "<em>",
        "</em>",
        "<a ",
        "<img ",
        "<h1>",
        "<h2>",
        "<h3>",
        // HTML entities
        "&lt;",
        "&gt;",
        "&amp;",
        "&quot;",
        "&nbsp;",
    ];

    let lower = trimmed.to_lowercase();
    html_indicators
        .iter()
        .any(|&indicator| lower.contains(indicator))
}

pub trait Document {
    /// Returns the documentation if available.
    fn documentation(&self) -> Option<&Documentation>;
}
