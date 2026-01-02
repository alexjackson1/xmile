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

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
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

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
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

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
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

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
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

#[cfg(feature = "submodels")]
#[test]
fn test_module_view_object_quick_xml_round_trip() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <module name="SubModel" resource="submodel.xmile">
                    <connect to="input_var" from="parent.output_var"/>
                </module>
            </variables>
            <views>
                <view width="800" height="600" page_width="800" page_height="600">
                    <module uid="1" name="SubModel" x="10" y="20" width="100" height="50">
                        <shape type="rectangle" width="90" height="40" corner_radius="5"/>
                    </module>
                </view>
            </views>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    let model = &file.models[0];
    match &model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            assert_eq!(module.resource.as_deref(), Some("submodel.xmile"));
            assert_eq!(module.connections.len(), 1);
            assert_eq!(module.connections[0].to, "input_var");
            assert_eq!(module.connections[0].from, "parent.output_var");
        }
        _ => panic!("Expected Module variant"),
    }

    let views = model.views.as_ref().expect("Expected views");
    let view = &views.views[0];
    assert_eq!(view.modules.len(), 1);
    let module_obj = &view.modules[0];
    assert_eq!(module_obj.name, "SubModel");
    assert_eq!(module_obj.x, 10.0);
    assert_eq!(module_obj.y, 20.0);
    assert_eq!(module_obj.width, 100.0);
    assert_eq!(module_obj.height, 50.0);
    match &module_obj.shape {
        Some(xmile::view::objects::Shape::Rectangle {
            width,
            height,
            corner_radius,
        }) => {
            assert_eq!(*width, 90.0);
            assert_eq!(*height, 40.0);
            assert_eq!(*corner_radius, Some(5.0));
        }
        _ => panic!("Expected rectangle shape for module object"),
    }

    // Round-trip through quick-xml serializer to ensure module objects and variables persist.
    let serialized = file.to_xml().expect("Failed to serialize XML");
    let round_trip = XmileFile::from_str(&serialized).expect("Failed to reparse serialized XML");
    let rt_model = &round_trip.models[0];
    match &rt_model.variables.variables[0] {
        xmile::model::vars::Variable::Module(module) => {
            assert_eq!(module.resource.as_deref(), Some("submodel.xmile"));
            assert_eq!(module.connections.len(), 1);
            assert_eq!(module.connections[0].to, "input_var");
            assert_eq!(module.connections[0].from, "parent.output_var");
        }
        _ => panic!("Expected Module variant after round-trip"),
    }
    let rt_view = rt_model
        .views
        .as_ref()
        .expect("Expected views after round-trip");
    assert_eq!(rt_view.views[0].modules.len(), 1);
}
