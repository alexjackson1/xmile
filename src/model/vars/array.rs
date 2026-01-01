use serde::{Deserialize, Serialize};

use crate::{
    Expression,
    model::vars::gf::GraphicalFunction,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "dim")]
pub struct Dimension {
    #[serde(rename = "@name")]
    pub name: String,
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
    pub eqn: Option<Expression>,
    /// The graphical function for this array element (for graphical functions).
    pub gf: Option<GraphicalFunction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_deserialization() {
        let xml = r#"<dim name="Length" />"#;
        let dimension: Dimension = serde_xml_rs::from_str(xml).unwrap();
        assert_eq!(dimension.name, "Length");
    }

    #[test]
    fn test_dimension_serialization() {
        let dimension = Dimension {
            name: "Length".to_string(),
        };
        let xml = serde_xml_rs::to_string(&dimension).unwrap();
        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="UTF-8"?><dim name="Length" />"#
        );
    }
}
