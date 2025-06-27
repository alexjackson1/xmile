use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "dim")]
pub struct Dimension {
    #[serde(rename = "@name")]
    pub name: String,
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
