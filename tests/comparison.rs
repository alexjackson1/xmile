//! Verification tests for quick-xml parsing.
//!
//! This module provides tests to verify that the quick-xml parser
//! correctly parses XMILE documents.

use thiserror::Error;
use xmile::xml::XmileFile;

/// Errors that can occur during verification testing.
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Verify that a document can be parsed correctly.
///
/// This function:
/// 1. Parses the XML with quick-xml
/// 2. Verifies key fields are present
///
/// Returns Ok(()) if parsing succeeds.
pub fn verify_parsing(xml: &str) -> Result<(), VerificationError> {
    let file =
        XmileFile::from_str(xml).map_err(|e| VerificationError::ParseFailed(e.to_string()))?;

    // Verify key fields
    if file.version.is_empty() {
        return Err(VerificationError::Validation(
            "Version is empty".to_string(),
        ));
    }

    if file.xmlns.is_empty() {
        return Err(VerificationError::Validation("XMLNS is empty".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_parsing_simple() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test Vendor</vendor>
        <product version="1.0">Test Product</product>
    </header>
    <model>
        <variables/>
    </model>
</xmile>"#;

        verify_parsing(xml).expect("Parsing should succeed");
    }

    #[test]
    fn test_verify_parsing_teacup() {
        let xml = include_str!("../data/examples/teacup.xmile");
        verify_parsing(xml).expect("Teacup example should parse correctly");
    }

    #[test]
    fn test_verify_parsing_with_variables() {
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
            <flow name="Test Flow">
                <eqn>10</eqn>
            </flow>
            <aux name="Test Aux">
                <eqn>50</eqn>
            </aux>
        </variables>
    </model>
</xmile>"#;

        verify_parsing(xml).expect("Parsing with variables should succeed");
    }

    #[test]
    fn test_verify_parsing_with_optional_fields() {
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
            <stock name="Test Stock">
                <eqn>100</eqn>
                <doc>Documentation</doc>
            </stock>
        </variables>
    </model>
</xmile>"#;

        verify_parsing(xml).expect("Parsing with optional fields should succeed");
    }
}
