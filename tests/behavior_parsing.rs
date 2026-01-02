use xmile::xml::schema::XmileFile;

#[test]
fn test_behavior_global_non_negative() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <behavior>
            <non_negative/>
        </behavior>
        <model>
            <variables/>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    assert!(file.behavior.is_some());
    let behavior = file.behavior.as_ref().unwrap();
    assert_eq!(behavior.global.non_negative, Some(true));
    assert_eq!(behavior.entities.len(), 0);
}

#[test]
fn test_behavior_entity_specific() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <behavior>
            <flow>
                <non_negative/>
            </flow>
        </behavior>
        <model>
            <variables/>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    assert!(file.behavior.is_some());
    let behavior = file.behavior.as_ref().unwrap();
    assert_eq!(behavior.global.non_negative, None);
    assert_eq!(behavior.entities.len(), 1);
    assert_eq!(behavior.entities[0].entity_type, "flow");
    assert_eq!(behavior.entities[0].behavior.non_negative, Some(true));
}

#[test]
fn test_behavior_mixed() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <behavior>
            <non_negative/>
            <stock>
                <non_negative/>
            </stock>
        </behavior>
        <model>
            <variables/>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    assert!(file.behavior.is_some());
    let behavior = file.behavior.as_ref().unwrap();
    assert_eq!(behavior.global.non_negative, Some(true));
    assert_eq!(behavior.entities.len(), 1);
    assert_eq!(behavior.entities[0].entity_type, "stock");
    assert_eq!(behavior.entities[0].behavior.non_negative, Some(true));
}
