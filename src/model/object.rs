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
    fn range(&self) -> Option<&Range>;
    fn scale(&self) -> Option<&Scale>;
    fn format(&self) -> Option<&FormatOptions>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Validate for Range {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayAs {
    Number,
    Currency,
    Percent,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scale {
    MinMax { min: f64, max: f64 },
    Auto(bool),
    Group(Option<u32>),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FormatOptions {
    pub precision: Option<f64>,
    pub scale_by: Option<f64>,
    pub display_as: Option<DisplayAs>,
    pub delimit_000s: Option<bool>,
}

impl Validate for FormatOptions {
    fn validate(&self) -> ValidationResult {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        if let Some(precision) = self.precision {
            if precision.is_sign_negative() {
                errors.push("Precision cannot be negative.".to_string());
            }
        }

        if let Some(scale_by) = self.scale_by {
            if scale_by.is_sign_negative() {
                errors.push("Scale factor cannot be negative.".to_string());
            }
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

pub trait Document {
    /// Returns the documentation if available.
    fn documentation(&self) -> Option<&Documentation>;
}
