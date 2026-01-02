use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    Expression, Identifier,
    model::vars::gf::GraphicalFunction,
    model::vars::{Var, Variable},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "dim")]
pub struct Dimension {
    #[serde(rename = "@name")]
    pub name: String,
}

/// Dimensions wrapper for variables.
/// Variables can have a `<dimensions>` tag containing `<dim name="..."/>` tags.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct VariableDimensions {
    #[serde(rename = "dim", default)]
    pub dims: Vec<Dimension>,
}

/// An array element for non-apply-to-all arrays.
/// According to XMILE spec section 4.5, non-apply-to-all arrayed variables
/// must have one `<element>` tag for each array entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayElement {
    /// Comma-separated indices in dimension order for the array element.
    #[serde(rename = "@subscript")]
    pub subscript: String,
    /// The equation for this array element (for non-graphical functions).
    #[serde(rename = "eqn", default, skip_serializing_if = "Option::is_none")]
    pub eqn: Option<Expression>,
    /// The graphical function for this array element (for graphical functions).
    #[serde(rename = "gf", default, skip_serializing_if = "Option::is_none")]
    pub gf: Option<GraphicalFunction>,
}

/// Registry for tracking which variables are arrays.
/// Used during expression parsing to resolve array function calls.
#[derive(Debug, Clone, Default)]
pub struct ArrayRegistry {
    /// Maps variable names (as strings) to whether they are arrays
    arrays: HashMap<String, bool>,
}

impl ArrayRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        ArrayRegistry {
            arrays: HashMap::new(),
        }
    }

    /// Build a registry from a list of variables.
    /// Checks each variable to see if it has dimensions (making it an array).
    pub fn from_variables(variables: &[Variable]) -> Self {
        let mut registry = ArrayRegistry::new();
        for var in variables {
            if let Some(name) = get_variable_name(var) {
                let is_array = is_array_variable(var);
                registry.register(name, is_array);
            }
        }
        registry
    }

    /// Register a variable as an array or non-array.
    pub fn register(&mut self, name: &Identifier, is_array: bool) {
        self.arrays.insert(name.to_string(), is_array);
    }

    /// Check if a variable with the given name is an array.
    pub fn contains(&self, name: &str) -> bool {
        self.arrays.get(name).copied().unwrap_or(false)
    }

    /// Get whether a variable is an array, returning None if not registered.
    pub fn get(&self, name: &str) -> Option<bool> {
        self.arrays.get(name).copied()
    }
}

/// Helper function to check if a variable is an array (has dimensions).
fn is_array_variable(var: &Variable) -> bool {
    #[cfg(feature = "arrays")]
    {
        match var {
            Variable::Auxiliary(aux) => aux.dimensions.is_some(),
            Variable::Stock(stock) => match stock {
                crate::model::vars::stock::Stock::Basic(b) => b.dimensions.is_some(),
                crate::model::vars::stock::Stock::Conveyor(c) => c.dimensions.is_some(),
                crate::model::vars::stock::Stock::Queue(q) => q.dimensions.is_some(),
            },
            Variable::Flow(flow) => flow.dimensions.is_some(),
            Variable::GraphicalFunction(gf) => gf.dimensions.is_some(),
            #[cfg(feature = "submodels")]
            Variable::Module(_) => false, // Modules are not arrays
            Variable::Group(_) => false, // Groups are not arrays
        }
    }
    #[cfg(not(feature = "arrays"))]
    {
        false // Arrays feature not enabled
    }
}

/// Helper function to get variable name from a Variable enum variant.
fn get_variable_name(var: &Variable) -> Option<&Identifier> {
    match var {
        Variable::Auxiliary(aux) => aux.name(),
        Variable::Stock(stock) => match stock {
            crate::model::vars::stock::Stock::Basic(b) => b.name(),
            crate::model::vars::stock::Stock::Conveyor(c) => c.name(),
            crate::model::vars::stock::Stock::Queue(q) => q.name(),
        },
        Variable::Flow(flow) => flow.name(),
        Variable::GraphicalFunction(gf) => gf.name(),
        #[cfg(feature = "submodels")]
        Variable::Module(module) => module.name(),
        Variable::Group(group) => Some(&group.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_creation() {
        let dimension = Dimension {
            name: "Length".to_string(),
        };
        assert_eq!(dimension.name, "Length");
    }

    #[test]
    fn test_variable_dimensions_creation() {
        let var_dims = VariableDimensions {
            dims: vec![
                Dimension {
                    name: "Row".to_string(),
                },
                Dimension {
                    name: "Column".to_string(),
                },
            ],
        };
        assert_eq!(var_dims.dims.len(), 2);
        assert_eq!(var_dims.dims[0].name, "Row");
        assert_eq!(var_dims.dims[1].name, "Column");
    }

    #[test]
    fn test_array_element_creation() {
        use crate::equation::numeric::NumericConstant;
        let element = ArrayElement {
            subscript: "1,1".to_string(),
            eqn: Some(crate::Expression::constant(NumericConstant(100.0))),
            gf: None,
        };
        assert_eq!(element.subscript, "1,1");
        assert!(element.eqn.is_some());
        assert!(element.gf.is_none());
    }
}
