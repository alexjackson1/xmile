use xmile::types::Validate;
use xmile::xml::schema::XmileFile;

#[test]
fn test_validate_variable_name_uniqueness() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <stock name="TestStock">
                    <eqn>100</eqn>
                </stock>
                <aux name="TestStock">
                    <eqn>50</eqn>
                </aux>
            </variables>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    let result = model.validate();

    assert!(result.is_invalid());
    if let xmile::types::ValidationResult::Invalid(_, errors) = result {
        assert!(
            errors.iter().any(
                |e| e.contains("TestStock") && (e.contains("Duplicate") || e.contains("found"))
            )
        );
    } else {
        panic!("Expected Invalid result");
    }
}

#[test]
fn test_validate_unique_variable_names() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <stock name="Stock1">
                    <eqn>100</eqn>
                </stock>
                <aux name="Aux1">
                    <eqn>50</eqn>
                </aux>
            </variables>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    let result = model.validate();

    assert!(result.is_valid() || result.has_warnings());
}

#[test]
fn test_validate_view_object_references() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <name>Test Model</name>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <stock name="Stock1">
                    <eqn>100</eqn>
                </stock>
            </variables>
            <views>
                <view uid="1" width="800" height="600" page_width="800" page_height="600">
                    <stock uid="1" name="NonExistentStock" x="100" y="100" width="50" height="50"/>
                </view>
            </views>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    let result = model.validate();

    assert!(result.is_invalid());
    if let xmile::types::ValidationResult::Invalid(_, errors) = result {
        assert!(errors.iter().any(|e| e.contains("NonExistentStock")
            && e.contains("references a variable that does not exist")));
    } else {
        panic!("Expected Invalid result");
    }
}

#[test]
fn test_validate_group_entity_references() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <name>Test Model</name>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <stock name="Stock1">
                    <eqn>100</eqn>
                </stock>
                <group name="TestGroup">
                    <entity name="NonExistentEntity"/>
                </group>
            </variables>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    let result = model.validate();

    assert!(result.is_invalid());
    if let xmile::types::ValidationResult::Invalid(_, errors) = result {
        assert!(
            errors
                .iter()
                .any(|e| e.contains("NonExistentEntity") && e.contains("undefined entity"))
        );
    } else {
        panic!("Expected Invalid result");
    }
}
