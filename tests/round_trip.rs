//! Round-trip testing infrastructure for XMILE XML serialization/deserialization.
//!
//! This module provides tests to ensure that XML files can be:
//! 1. Parsed with quick-xml → Serialized with quick-xml → Re-parsed with quick-xml
//!
//! This validates that the serialization produces valid XML and that
//! the parsing/serialization cycle is consistent.

use thiserror::Error;
use xmile::xml::{ParseError, XmileFile};

/// Errors that can occur during round-trip testing.
#[derive(Debug, Error)]
pub enum RoundTripError {
    #[error("Initial parse failed: {0}")]
    InitialParse(#[from] ParseError),
    #[error("Serialization failed: {0}")]
    Serialization(#[from] xmile::xml::serialize::SerializeError),
    #[error("Re-parse failed: {0}")]
    Reparse(String),
    #[error("Round-trip mismatch: {0}")]
    Mismatch(String),
}

/// Perform a round-trip test: parse → serialize → re-parse.
///
/// This test:
/// 1. Parses the XML with quick-xml
/// 2. Serializes it with quick-xml
/// 3. Re-parses the serialized XML with quick-xml
///
/// The test passes if all steps succeed and key fields match.
pub fn round_trip_test(xml: &str) -> Result<(), RoundTripError> {
    // Step 1: Parse with quick-xml
    let file1 = XmileFile::from_str(xml)?;

    // Step 2: Serialize with quick-xml
    let serialized = file1.to_xml()?;

    // Step 3: Re-parse with quick-xml
    let file2 =
        XmileFile::from_str(&serialized).map_err(|e| RoundTripError::Reparse(e.to_string()))?;

    // Compare file1 and file2 for semantic equivalence
    // Note: We compare key fields, not exact byte-for-byte equality
    // since serialization may differ in whitespace, attribute order, etc.
    if file1.version != file2.version {
        return Err(RoundTripError::Mismatch(format!(
            "Version mismatch: {} != {}",
            file1.version, file2.version
        )));
    }

    if file1.xmlns != file2.xmlns {
        return Err(RoundTripError::Mismatch(format!(
            "XMLNS mismatch: {} != {}",
            file1.xmlns, file2.xmlns
        )));
    }

    if file1.models.len() != file2.models.len() {
        return Err(RoundTripError::Mismatch(format!(
            "Model count mismatch: {} != {}",
            file1.models.len(),
            file2.models.len()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_simple() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test Vendor</vendor>
        <product version="1.0">Test Product</product>
    </header>
    <sim_specs>
        <start>0.0</start>
        <stop>10.0</stop>
    </sim_specs>
    <model>
        <variables/>
    </model>
</xmile>"#;

        round_trip_test(xml).expect("Simple round-trip should succeed");
    }

    #[test]
    fn test_round_trip_teacup_example() {
        let xml = include_str!("../data/examples/teacup.xmile");
        round_trip_test(xml).expect("Teacup example round-trip should succeed");
    }

    // Test all variable types individually

    #[test]
    fn test_round_trip_stock() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <stock name="Test Stock">
                <eqn>100</eqn>
                <doc>Test stock variable</doc>
            </stock>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Stock round-trip should succeed");
    }

    #[test]
    fn test_round_trip_flow() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <flow name="Test Flow">
                <eqn>10</eqn>
                <doc>Test flow variable</doc>
            </flow>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Flow round-trip should succeed");
    }

    #[test]
    fn test_round_trip_auxiliary() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <aux name="Test Aux">
                <eqn>50</eqn>
                <doc>Test auxiliary variable</doc>
            </aux>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Auxiliary round-trip should succeed");
    }

    #[test]
    fn test_round_trip_graphical_function() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <gf name="Test GF">
                <xscale min="0" max="10"/>
                <yscale min="0" max="100"/>
                <ypts>0, 50, 100</ypts>
            </gf>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Graphical function round-trip should succeed");
    }

    // Test edge cases: empty tags, optional fields

    #[test]
    fn test_round_trip_empty_tags() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <stock name="Test Stock">
                <eqn>100</eqn>
            </stock>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Empty tags round-trip should succeed");
    }

    #[test]
    fn test_round_trip_optional_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <stock name="Test Stock">
                <eqn>100</eqn>
                <doc>Documentation</doc>
                <units>units</units>
            </stock>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Optional fields round-trip should succeed");
    }

    #[test]
    fn test_round_trip_nested_structures() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            <stock name="Stock1">
                <eqn>100</eqn>
                <inflow>Flow1</inflow>
            </stock>
            <flow name="Flow1">
                <eqn>10</eqn>
            </flow>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Nested structures round-trip should succeed");
    }

    // Test with all top-level optional elements

    #[test]
    fn test_round_trip_with_model_units() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model_units>
        <unit name="test_unit"/>
    </model_units>
    <model>
        <variables/>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Model units round-trip should succeed");
    }

    #[test]
    fn test_round_trip_with_dimensions() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <dimensions>
        <dim name="test_dim" size="10"/>
    </dimensions>
    <model>
        <variables/>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Dimensions round-trip should succeed");
    }

    #[test]
    fn test_round_trip_with_behavior() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <behavior>
        <non_negative/>
    </behavior>
    <model>
        <variables/>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Behavior round-trip should succeed");
    }

    #[test]
    fn test_round_trip_with_style() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <style color="blue"/>
    <model>
        <variables/>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Style round-trip should succeed");
    }

    #[test]
    fn test_round_trip_with_data() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <data>
        <import type="CSV" resource="test.csv"/>
    </data>
    <model>
        <variables/>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Data round-trip should succeed");
    }

    // Test complex models with all features

    #[test]
    fn test_round_trip_complex_model() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <sim_specs>
        <start>0.0</start>
        <stop>10.0</stop>
        <dt>0.1</dt>
    </sim_specs>
    <model>
        <variables>
            <stock name="Stock1">
                <eqn>100</eqn>
                <inflow>Flow1</inflow>
                <doc>Test stock</doc>
            </stock>
            <flow name="Flow1">
                <eqn>10</eqn>
                <doc>Test flow</doc>
            </flow>
            <aux name="Aux1">
                <eqn>50</eqn>
                <doc>Test aux</doc>
            </aux>
        </variables>
    </model>
</xmile>"#;
        round_trip_test(xml).expect("Complex model round-trip should succeed");
    }
}
