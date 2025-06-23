use std::str::FromStr;

use serde::{Deserialize, Deserializer};

use crate::model::gf::Points;

use super::{
    GraphicalFunction, GraphicalFunctionData, GraphicalFunctionScale, GraphicalFunctionType,
    Identifier,
};

/// Graphical Function Scale XML representation.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
struct RawGraphicalFunctionScale {
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
}

impl From<RawGraphicalFunctionScale> for GraphicalFunctionScale {
    fn from(raw: RawGraphicalFunctionScale) -> Self {
        GraphicalFunctionScale {
            min: raw.min,
            max: raw.max,
        }
    }
}

impl<'de> Deserialize<'de> for GraphicalFunctionScale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawGraphicalFunctionScale::deserialize(deserializer).map(Into::into)
    }
}

/// Points XML representation.
#[derive(Debug, Deserialize)]
struct RawPoints {
    #[serde(rename = "@sep")]
    separator: Option<String>,
    #[serde(rename = "#text")]
    data: String,
}

impl TryFrom<RawPoints> for Points {
    type Error = String;

    fn try_from(raw: RawPoints) -> Result<Self, Self::Error> {
        raw.data
            .split(raw.separator.as_deref().unwrap_or(","))
            .map(|val_str| val_str.trim().parse::<f64>())
            .collect::<Result<Vec<f64>, _>>()
            .map(|vals| Points::new(vals, raw.separator))
            .map_err(|e| format!("Failed to parse number: {}", e))
    }
}

/// Graphical Function Data XML representation.
#[derive(Debug, Deserialize)]
struct RawGraphicalFunctionData {
    #[serde(rename = "xscale")]
    x_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "yscale")]
    y_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "ypts")]
    y_pts: Option<RawPoints>,
    #[serde(rename = "xpts")]
    x_pts: Option<RawPoints>,
}

impl TryFrom<RawGraphicalFunctionData> for GraphicalFunctionData {
    type Error = String;

    fn try_from(raw: RawGraphicalFunctionData) -> Result<Self, Self::Error> {
        // Parse y-values (required for both variants)
        let y_values: Points = raw
            .y_pts
            .ok_or("ypts is required for graphical function data")?
            .try_into()?;

        if y_values.is_empty() {
            return Err("y-values cannot be empty".to_string());
        }

        // Determine which variant to create based on presence of x-scale vs x-pts
        match (raw.x_scale, raw.x_pts) {
            (Some(x_scale), None) => {
                if y_values.is_empty() {
                    return Err("y-values cannot be empty".to_string());
                }

                Ok(GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_scale: raw.y_scale,
                    y_values,
                })
            }
            (None, Some(raw_x)) => {
                let x_values: Points = raw_x.try_into()?;

                if x_values.len() != y_values.len() {
                    return Err("x-values and y-values must have the same length".to_string());
                }

                Ok(GraphicalFunctionData::XYPairs {
                    y_scale: raw.y_scale,
                    x_values,
                    y_values,
                })
            }
            (Some(_), Some(_)) => Err("Cannot have both xscale and xpts".to_string()),
            (None, None) => Err("Either xscale or xpts must be provided".to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for GraphicalFunctionData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawGraphicalFunctionData::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

// Helper struct for deserializing the raw XML structure
#[derive(Debug, Deserialize)]
struct RawGraphicalFunction {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
    // serde-xml-rs doesn't support flattening the below enum directly,
    // so we deserialize it as additional fields
    #[serde(rename = "xscale")]
    x_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "yscale")]
    y_scale: Option<GraphicalFunctionScale>,
    #[serde(rename = "ypts")]
    y_pts: Option<RawPoints>,
    #[serde(rename = "xpts")]
    x_pts: Option<RawPoints>,
}

impl From<RawGraphicalFunction> for RawGraphicalFunctionData {
    fn from(raw: RawGraphicalFunction) -> Self {
        RawGraphicalFunctionData {
            x_scale: raw.x_scale,
            y_scale: raw.y_scale,
            y_pts: raw.y_pts,
            x_pts: raw.x_pts,
        }
    }
}

impl TryFrom<RawGraphicalFunction> for GraphicalFunction {
    type Error = String;

    fn try_from(raw: RawGraphicalFunction) -> Result<Self, Self::Error> {
        // Optionally parse name if present using Identifier::from_str
        let name = match raw.name.as_ref() {
            Some(name_str) => Some(Identifier::from_str(&name_str).map_err(|e| e.to_string())?),
            None => None,
        };

        // Optionally parse type if present using GraphicalFunctionType::from_str
        let function_type = match raw.r#type.as_ref() {
            Some(type_str) => {
                Some(GraphicalFunctionType::from_str(&type_str).map_err(|e| e.to_string())?)
            }
            None => None,
        };

        // Convert raw data into GraphicalFunctionData
        let data = Into::<RawGraphicalFunctionData>::into(raw).try_into()?;

        Ok(GraphicalFunction {
            name,
            function_type,
            data,
        })
    }
}

impl<'de> Deserialize<'de> for GraphicalFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RawGraphicalFunction::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod xml_tests {
    use crate::Identifier;
    use crate::model::gf::{
        GraphicalFunction, GraphicalFunctionData, GraphicalFunctionScale, GraphicalFunctionType,
    };

    // Tests for examples directly from the XMILE specification
    mod specification_examples {
        use crate::types::Validate;

        use super::*;

        #[test]
        fn test_spec_minimal_named_function() {
            // From specification: smallest possible named graphical function
            let xml = r#"<gf name="rising">
                <xscale min="0" max="1"/>
                <ypts>0,0.1,0.5,0.9,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            assert_eq!(
                function.name,
                Some(Identifier::parse_default("rising").unwrap())
            );
            assert!(function.function_type.is_none()); // Should default to continuous

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_scale.min, 0.0);
                    assert_eq!(x_scale.max, 1.0);
                    assert_eq!(y_values.values, vec![0.0, 0.1, 0.5, 0.9, 1.0]);
                    assert!(y_scale.is_none());
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_spec_food_availability_uniform_scale() {
            // From specification: food_availability_multiplier_function with uniform scale
            let xml = r#"<gf name="food_availability_multiplier_function" type="continuous">
                <xscale min="0" max="1"/>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            assert_eq!(
                function.name,
                Some(Identifier::parse_default("food_availability_multiplier_function").unwrap())
            );
            assert_eq!(
                function.function_type,
                Some(GraphicalFunctionType::Continuous)
            );

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_scale.min, 0.0);
                    assert_eq!(x_scale.max, 1.0);
                    assert_eq!(
                        y_values.values,
                        vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                    );
                    assert!(y_scale.is_none());
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_spec_food_availability_xy_pairs() {
            // From specification: food_availability_multiplier_function with x-y pairs
            let xml = r#"<gf name="food_availability_multiplier_function" type="continuous">
                <xpts>0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1</xpts>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            assert_eq!(
                function.name,
                Some(Identifier::parse_default("food_availability_multiplier_function").unwrap())
            );
            assert_eq!(
                function.function_type,
                Some(GraphicalFunctionType::Continuous)
            );

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(
                        x_values.values,
                        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                    );
                    assert_eq!(
                        y_values.values,
                        vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                    );
                    assert!(y_scale.is_none());
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }

        #[test]
        fn test_spec_embedded_function() {
            // From specification: embedded function (anonymous)
            let xml = r#"<gf>
                <xscale min="0.0" max="1.0"/>
                <ypts>0,0.3,0.55,0.7,0.83,0.9,0.95,0.98,0.99,0.995,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            assert!(function.name.is_none()); // Anonymous/embedded
            assert!(function.function_type.is_none()); // Should default

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_scale.min, 0.0);
                    assert_eq!(x_scale.max, 1.0);
                    assert_eq!(
                        y_values.values,
                        vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0]
                    );
                    assert!(y_scale.is_none());
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_spec_overspecified_function_should_fail() {
            // From specification: overspecified function (has both xscale and xpts)
            let xml = r#"<gf name="overspecified">
                <xscale min="0" max="0.5"/>
                <yscale min="0" max="1"/>
                <xpts>0,0.1,0.2,0.3,0.4,0.5</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_spec_inconsistent_function_should_fail() {
            // From specification: inconsistent function (xpts don't match xscale)
            let xml = r#"<gf name="inconsistent">
                <xscale min="0" max="0.5"/>
                <yscale min="0" max="1"/>
                <xpts>0,1,2,3,4,5</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_spec_invalid_xpts_order_should_fail() {
            // From specification: invalid function with unordered x-values
            let xml = r#"<gf name="invalid">
                <yscale min="0" max="1"/>
                <xpts>2,1,3,0</xpts>
                <ypts>0.05,0.1,0.2,0.25</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            // This should fail during validation, but may parse successfully initially
            // The validation would catch the unordered x-values
            if let Ok(function) = result {
                // If it parses, validation should catch the error
                assert!(function.validate().is_invalid());
            }
        }

        #[test]
        fn test_spec_mismatched_xy_length_should_fail() {
            // From specification: mismatched x and y value counts
            let xml = r#"<gf name="invalid">
                <yscale min="0" max="1"/>
                <xpts>2,1,3,0</xpts>
                <ypts>0.05,0.1,0.2,0.25,0.3,0.33</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }
    }

    // Tests for valid configurations not explicitly shown in specification
    mod additional_valid_configurations {
        use super::*;

        #[test]
        fn test_all_function_types() {
            let continuous_xml = r#"<gf name="continuous_func" type="continuous">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let extrapolate_xml = r#"<gf name="extrapolate_func" type="extrapolate">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let discrete_xml = r#"<gf name="discrete_func" type="discrete">
                <xscale min="0" max="1"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let continuous_func: GraphicalFunction =
                serde_xml_rs::from_str(continuous_xml).unwrap();
            let extrapolate_func: GraphicalFunction =
                serde_xml_rs::from_str(extrapolate_xml).unwrap();
            let discrete_func: GraphicalFunction = serde_xml_rs::from_str(discrete_xml).unwrap();

            assert_eq!(
                continuous_func.function_type,
                Some(GraphicalFunctionType::Continuous)
            );
            assert_eq!(
                extrapolate_func.function_type,
                Some(GraphicalFunctionType::Extrapolate)
            );
            assert_eq!(
                discrete_func.function_type,
                Some(GraphicalFunctionType::Discrete)
            );
        }

        #[test]
        fn test_case_insensitive_function_types() {
            let xml = r#"<gf name="test_func" type="CONTINUOUS">
                <xscale min="0" max="1"/>
                <ypts>0,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();
            assert_eq!(
                function.function_type,
                Some(GraphicalFunctionType::Continuous)
            );
        }

        #[test]
        fn test_with_y_scale() {
            let xml = r#"<gf name="scaled_func">
                <xscale min="0" max="1"/>
                <yscale min="-5" max="10"/>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_scale, .. } => {
                    assert_eq!(y_scale, Some(GraphicalFunctionScale::new(-5.0, 10.0)));
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_negative_values() {
            let xml = r#"<gf name="negative_func">
                <xscale min="-10" max="10"/>
                <ypts>-1,-0.5,0,0.5,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => {
                    assert_eq!(x_scale.min, -10.0);
                    assert_eq!(x_scale.max, 10.0);
                    assert_eq!(y_values.values, vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_single_point_function() {
            let xml = r#"<gf name="single_point">
                <xscale min="5" max="5"/>
                <ypts>42</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => {
                    assert_eq!(x_scale.min, 5.0);
                    assert_eq!(x_scale.max, 5.0);
                    assert_eq!(y_values.values, vec![42.0]);
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_fractional_values() {
            let xml = r#"<gf name="fractional">
                <xscale min="0.1" max="0.9"/>
                <ypts>0.01,0.25,0.75,0.99</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => {
                    assert_eq!(x_scale.min, 0.1);
                    assert_eq!(x_scale.max, 0.9);
                    assert_eq!(y_values.values, vec![0.01, 0.25, 0.75, 0.99]);
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_scientific_notation() {
            let xml = r#"<gf name="scientific">
                <xscale min="1e-3" max="1e3"/>
                <ypts>1e-6,1e-3,1e0,1e3,1e6</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale {
                    x_scale, y_values, ..
                } => {
                    assert_eq!(x_scale.min, 0.001);
                    assert_eq!(x_scale.max, 1000.0);
                    assert_eq!(
                        y_values.values,
                        vec![0.000001, 0.001, 1.0, 1000.0, 1000000.0]
                    );
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_xy_pairs_with_y_scale() {
            let xml = r#"<gf name="xy_with_yscale">
                <yscale min="0" max="100"/>
                <xpts>0,0.5,1</xpts>
                <ypts>10,50,90</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values,
                    y_values,
                    y_scale,
                } => {
                    assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                    assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                    assert_eq!(y_scale, Some(GraphicalFunctionScale::new(0.0, 100.0)));
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }

        #[test]
        fn test_irregular_x_spacing() {
            let xml = r#"<gf name="irregular">
                <xpts>0,0.1,0.15,0.2,0.8,0.95,1</xpts>
                <ypts>0,0.2,0.3,0.4,0.7,0.9,1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => {
                    assert_eq!(x_values.values, vec![0.0, 0.1, 0.15, 0.2, 0.8, 0.95, 1.0]);
                    assert_eq!(y_values.values, vec![0.0, 0.2, 0.3, 0.4, 0.7, 0.9, 1.0]);
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }
    }

    // Tests for custom separators
    mod separator_tests {
        use super::*;

        #[test]
        fn test_semicolon_separator() {
            let xml = r#"<gf name="semicolon_sep">
                <xscale min="0" max="1"/>
                <ypts sep=";">0;0.25;0.5;0.75;1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_values, .. } => {
                    assert_eq!(y_values.values, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
                    assert_eq!(y_values.separator(), Some(";"));
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_pipe_separator() {
            let xml = r#"<gf name="pipe_sep">
                <xpts sep="|">0|0.5|1</xpts>
                <ypts sep="|">10|50|90</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => {
                    assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                    assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                    assert_eq!(x_values.separator(), Some("|"));
                    assert_eq!(y_values.separator(), Some("|"));
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }

        #[test]
        fn test_space_separator() {
            let xml = r#"<gf name="space_sep">
                <xscale min="0" max="2"/>
                <ypts sep=" ">0 1 4</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_values, .. } => {
                    assert_eq!(y_values.values, vec![0.0, 1.0, 4.0]);
                    assert_eq!(y_values.separator(), Some(" "));
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_tab_separator() {
            let xml = r#"<gf name="tab_sep">
                <xscale min="0" max="1"/>
                <ypts sep="	">0	0.5	1</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_values, .. } => {
                    assert_eq!(y_values.values, vec![0.0, 0.5, 1.0]);
                    assert_eq!(y_values.separator(), Some("\t"));
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_mixed_separators_xy_pairs() {
            // Different separators for x and y points
            let xml = r#"<gf name="mixed_sep">
                <xpts sep=";">0;0.5;1</xpts>
                <ypts sep=",">10,50,90</ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => {
                    assert_eq!(x_values.values, vec![0.0, 0.5, 1.0]);
                    assert_eq!(y_values.values, vec![10.0, 50.0, 90.0]);
                    assert_eq!(x_values.separator(), Some(";"));
                    assert_eq!(y_values.separator(), Some(","));
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }
    }

    // Tests for edge cases and error conditions
    mod error_condition_tests {
        use super::*;

        #[test]
        fn test_missing_ypts_should_fail() {
            let xml = r#"<gf name="no_ypts">
                <xscale min="0" max="1"/>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_missing_xscale_and_xpts_should_fail() {
            let xml = r#"<gf name="no_x_data">
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_empty_ypts_should_fail() {
            let xml = r#"<gf name="empty_ypts">
                <xscale min="0" max="1"/>
                <ypts></ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_number_in_ypts_should_fail() {
            let xml = r#"<gf name="invalid_number">
                <xscale min="0" max="1"/>
                <ypts>0,invalid,1</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_function_type_should_fail() {
            let xml = r#"<gf name="invalid_type" type="invalid_type">
                <xscale min="0" max="1"/>
                <ypts>0,1</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_both_xscale_and_xpts_should_fail() {
            let xml = r#"<gf name="both_x">
                <xscale min="0" max="1"/>
                <xpts>0,0.5,1</xpts>
                <ypts>0,0.5,1</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_mismatched_xy_lengths_should_fail() {
            let xml = r#"<gf name="mismatched">
                <xpts>0,0.5,1</xpts>
                <ypts>0,0.5</ypts>
            </gf>"#;

            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_separator_parsing() {
            let xml = r#"<gf name="bad_sep">
                <xscale min="0" max="1"/>
                <ypts sep=",">0;0.5;1</ypts>
            </gf>"#;

            // This should fail because separator is "," but values use ";"
            let result: Result<GraphicalFunction, _> = serde_xml_rs::from_str(xml);
            assert!(result.is_err());
        }
    }

    // Tests for whitespace handling
    mod whitespace_tests {
        use super::*;

        #[test]
        fn test_whitespace_in_values() {
            let xml = r#"<gf name="whitespace">
                <xscale min="0" max="1"/>
                <ypts> 0 , 0.25 , 0.5 , 0.75 , 1 </ypts>
            </gf>"#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_values, .. } => {
                    assert_eq!(y_values.values, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_multiline_xml() {
            let xml = r#"
            <gf name="multiline">
                <xscale min="0" max="1"/>
                <yscale min="0" max="100"/>
                <ypts>
                    0,
                    25,
                    50,
                    75,
                    100
                </ypts>
            </gf>
            "#;

            let function: GraphicalFunction = serde_xml_rs::from_str(xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale {
                    y_values, y_scale, ..
                } => {
                    assert_eq!(y_values.values, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
                    assert_eq!(y_scale, Some(GraphicalFunctionScale::new(0.0, 100.0)));
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }
    }

    // Tests for large datasets
    mod large_dataset_tests {
        use super::*;

        #[test]
        fn test_large_uniform_scale_function() {
            let y_values: Vec<String> =
                (0..1000).map(|i| (i as f64 / 1000.0).to_string()).collect();
            let ypts_str = y_values.join(",");

            let xml = format!(
                r#"<gf name="large_function">
                <xscale min="0" max="999"/>
                <ypts>{}</ypts>
            </gf>"#,
                ypts_str
            );

            let function: GraphicalFunction = serde_xml_rs::from_str(&xml).unwrap();

            match function.data {
                GraphicalFunctionData::UniformScale { y_values, .. } => {
                    assert_eq!(y_values.len(), 1000);
                    assert_eq!(y_values[0], 0.0);
                    assert_eq!(y_values[999], 0.999);
                }
                _ => panic!("Expected UniformScale variant"),
            }
        }

        #[test]
        fn test_large_xy_pairs_function() {
            let xy_pairs: Vec<(String, String)> = (0..100)
                .map(|i| {
                    let x = (i as f64 / 10.0).to_string();
                    let y = (i as f64 * i as f64 / 100.0).to_string();
                    (x, y)
                })
                .collect();

            let x_values: Vec<String> = xy_pairs.iter().map(|(x, _)| x.clone()).collect();
            let y_values: Vec<String> = xy_pairs.iter().map(|(_, y)| y.clone()).collect();

            let xml = format!(
                r#"<gf name="large_xy_function">
                <xpts>{}</xpts>
                <ypts>{}</ypts>
            </gf>"#,
                x_values.join(","),
                y_values.join(",")
            );

            let function: GraphicalFunction = serde_xml_rs::from_str(&xml).unwrap();

            match function.data {
                GraphicalFunctionData::XYPairs {
                    x_values, y_values, ..
                } => {
                    assert_eq!(x_values.len(), 100);
                    assert_eq!(y_values.len(), 100);
                    assert_eq!(x_values[0], 0.0);
                    assert_eq!(x_values[99], 9.9);
                }
                _ => panic!("Expected XYPairs variant"),
            }
        }
    }
}
