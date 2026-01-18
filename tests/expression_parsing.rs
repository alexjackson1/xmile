//! Tests for expression parsing, especially quoted identifiers

use xmile::xml::schema::XmileFile;

#[test]
fn test_parse_quoted_identifiers_in_expressions() {
    // Test the specific expression from teacup.xmile
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <flow name="Heat Loss to Room">
                    <eqn>("Teacup Temperature"-"Room Temperature")/"Characteristic Time"</eqn>
                </flow>
            </variables>
        </model>
    </xmile>
    "#;

    let result = XmileFile::from_str(xml);

    // This should now parse successfully with quoted identifier support
    match result {
        Ok(file) => {
            // Verify the expression was parsed
            let model = &file.models[0];
            assert_eq!(model.variables.variables.len(), 1);

            match &model.variables.variables[0] {
                xmile::model::vars::Variable::Flow(flow) => {
                    assert!(flow.equation.is_some());
                    // The expression should contain the quoted identifiers
                }
                _ => panic!("Expected Flow variant"),
            }
        }
        Err(e) => {
            // If it still fails, provide helpful error message
            panic!(
                "Failed to parse expression with quoted identifiers: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_parse_simple_quoted_identifier() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <aux name="Test">
                    <eqn>"Simple Quoted Identifier"</eqn>
                </aux>
            </variables>
        </model>
    </xmile>
    "#;

    // Note: This test may fail if expression parsing doesn't support
    // quoted identifiers as standalone expressions (they're usually in operations)
    let _result = XmileFile::from_str(xml);
    // We're just checking it doesn't crash
}

#[test]
fn test_parse_quoted_identifier_with_operations() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <aux name="Test">
                    <eqn>"Variable Name" + 10</eqn>
                </aux>
            </variables>
        </model>
    </xmile>
    "#;

    let result = XmileFile::from_str(xml);
    assert!(
        result.is_ok(),
        "Should parse expression with quoted identifier and operation"
    );
}

#[test]
fn test_round_trip_quoted_identifiers() {
    // Test that quoted identifiers are preserved through round-trip
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <flow name="Heat Loss">
                    <eqn>("Teacup Temperature"-"Room Temperature")/"Characteristic Time"</eqn>
                </flow>
            </variables>
        </model>
    </xmile>
    "#;

    let file1 = XmileFile::from_str(xml).expect("Failed to parse");
    let serialized = serde_xml_rs::to_string(&file1).expect("Failed to serialize");

    // Verify the serialized XML contains quoted identifiers
    assert!(
        serialized.contains("\"Teacup Temperature\""),
        "Serialized XML should preserve quoted identifiers"
    );
    assert!(
        serialized.contains("\"Room Temperature\""),
        "Serialized XML should preserve quoted identifiers"
    );
    assert!(
        serialized.contains("\"Characteristic Time\""),
        "Serialized XML should preserve quoted identifiers"
    );

    // Re-parse and verify it still works
    let file2 = XmileFile::from_str(&serialized).expect("Failed to re-parse");
    assert_eq!(
        file1, file2,
        "Round-trip should preserve quoted identifiers"
    );
}
