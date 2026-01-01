// When the <uses_arrays> XMILE option is set, a list of dimension names is REQUIRED. These dimension names must be consistent across the whole-model. The set of dimension names appear within a <dimensions> block as shown in the example below.
// <dimensions>
//    <dim name="N" size="5"/>   <!-- numbered indices -->
//    <dim name="Location">      <!-- named indices -->
//      <elem name="Boston"/>   <!-- name of 1st index -->
//      <elem name="Chicago"/>  <!-- name of 2nd index -->
//      <elem name="LA"/>       <!-- name of 3rd index -->
//    </dim>
// </dimensions>
// Each dimension name is identified with a <dim> tag and a REQUIRED name. If the elements are not named, a size attribute greater or equal to one MUST be given. If the elements have names, they appear in order in <elem> nodes. The dimension size MUST NOT appear when elements have names as the number of element names always determines the size of such dimensions.

use crate::types::{Validate, ValidationResult};

pub struct Dimensions {
    /// A list of dimension definitions in the XMILE file.
    pub dims: Vec<Dimension>,
}

impl Validate for Dimensions {
    fn validate(&self) -> ValidationResult<(), String, String> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut dim_names = std::collections::HashSet::new();

        for (idx, dim) in self.dims.iter().enumerate() {
            if !dim_names.insert(&dim.name) {
                errors.push(format!("Duplicate dimension name found: {}", dim.name));
            }

            match dim.validate() {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, ws) => {
                    let mut warning = format!("Warnings in dimension at index {}\n", idx);
                    ws.iter().for_each(|w| {
                        warning.push_str(&format!(" - {}\n", w));
                    });
                    warnings.push(warning);
                }
                ValidationResult::Invalid(ws, es) => {
                    let mut error = format!("Errors in dimension at index {}\n", idx);
                    es.iter().for_each(|e| {
                        error.push_str(&format!(" - {}\n", e));
                    });
                    errors.push(error);

                    let mut warning = format!("Warnings in dimension at index {}\n", idx);
                    ws.iter().for_each(|w| {
                        warning.push_str(&format!(" - {}\n", w));
                    });
                    warnings.push(warning);
                }
            }
        }

        if !errors.is_empty() {
            return ValidationResult::Invalid(warnings, errors);
        }

        if !warnings.is_empty() {
            return ValidationResult::Warnings((), warnings);
        }

        ValidationResult::Valid(())
    }
}

pub struct Dimension {
    /// The name of the dimension.
    pub name: String,
    /// The size of the dimension (if elements are not named).
    pub size: Option<usize>,
    /// A list of element names for the dimension (if named).
    pub elements: Vec<String>,
}

impl Validate for Dimension {
    fn validate(&self) -> ValidationResult<(), String, String> {
        let mut warnings = Vec::new();
        if let Some(size) = self.size {
            if size == 0 {
                warnings.push("Dimension size must be greater than zero.".to_string());
            }
        } else if self.elements.is_empty() {
            return ValidationResult::Invalid(
                warnings,
                vec!["Dimension must have either a size or named elements.".to_string()],
            );
        }

        if !warnings.is_empty() {
            return ValidationResult::Warnings((), warnings);
        }

        ValidationResult::Valid(())
    }
}
