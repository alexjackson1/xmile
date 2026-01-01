#[cfg(feature = "submodels")]
use xmile::xml::schema::XmileFile;

#[cfg(feature = "submodels")]
#[test]
fn test_module_basic() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <module name="SubModel"/>
            </variables>
        </model>
    </xmile>
    "#;

    let file: XmileFile = serde_xml_rs::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    assert_eq!(model.variables.variables.len(), 1);
    
    match &model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            // Identifier may or may not normalize camelCase depending on parsing
            // The actual value is "SubModel" (not normalized)
            assert_eq!(&module.name.to_string(), "SubModel");
            assert!(module.resource.is_none());
            assert_eq!(module.connections.len(), 0);
            assert!(module.documentation.is_none());
        }
        _ => panic!("Expected Module variant"),
    }
}

#[cfg(feature = "submodels")]
#[test]
fn test_module_with_resource() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <module name="ExternalModel" resource="submodel.xmile"/>
            </variables>
        </model>
    </xmile>
    "#;

    let file: XmileFile = serde_xml_rs::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    match &model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            assert_eq!(&module.name.to_string(), "ExternalModel");
            assert_eq!(module.resource.as_ref().unwrap(), "submodel.xmile");
        }
        _ => panic!("Expected Module variant"),
    }
}

#[cfg(feature = "submodels")]
#[test]
fn test_module_with_connections() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <module name="SubModel">
                    <connect to="input_var" from="parent.output_var"/>
                    <connect to="another_input" from="parent.another_output"/>
                </module>
            </variables>
        </model>
    </xmile>
    "#;

    let file: XmileFile = serde_xml_rs::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    match &model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            assert_eq!(module.connections.len(), 2);
            assert_eq!(module.connections[0].to, "input_var");
            assert_eq!(module.connections[0].from, "parent.output_var");
            assert_eq!(module.connections[1].to, "another_input");
            assert_eq!(module.connections[1].from, "parent.another_output");
        }
        _ => panic!("Expected Module variant"),
    }
}

#[cfg(feature = "submodels")]
#[test]
fn test_module_with_documentation() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <module name="SubModel">
                    <doc>This is a submodel module</doc>
                </module>
            </variables>
        </model>
    </xmile>
    "#;

    let file: XmileFile = serde_xml_rs::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    match &model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            assert!(module.documentation.is_some());
            if let Some(doc) = &module.documentation {
                match doc {
                    xmile::model::object::Documentation::PlainText(text) => {
                        assert!(text.contains("submodel module"));
                    }
                    _ => panic!("Expected plain text documentation"),
                }
            }
        }
        _ => panic!("Expected Module variant"),
    }
}
