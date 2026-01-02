#[cfg(feature = "macros")]
use xmile::xml::schema::XmileFile;

#[cfg(feature = "macros")]
#[test]
fn test_macro_basic() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <macro name="test_macro">
            <eqn>param1 + param2</eqn>
        </macro>
        <model>
            <variables/>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    assert_eq!(file.macros.len(), 1);
    let macro_def = &file.macros[0];
    // Identifier normalizes underscores to spaces
    assert_eq!(&macro_def.name.to_string(), "test macro");
    assert_eq!(macro_def.parameters.len(), 0);
    assert!(macro_def.variables.is_none());
    assert!(macro_def.views.is_none());
}

#[cfg(feature = "macros")]
#[test]
fn test_macro_with_parameters() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <macro name="add">
            <parm>a</parm>
            <parm default="0">b</parm>
            <eqn>a + b</eqn>
        </macro>
        <model>
            <variables/>
        </model>
    </xmile>
    "#;

    let file = XmileFile::from_str(xml).expect("Failed to parse XML");
    assert_eq!(file.macros.len(), 1);
    let macro_def = &file.macros[0];
    assert_eq!(&macro_def.name.to_string(), "add");
    assert_eq!(macro_def.parameters.len(), 2);
    assert_eq!(&macro_def.parameters[0].name.to_string(), "a");
    assert_eq!(macro_def.parameters[0].default, None);
    assert_eq!(&macro_def.parameters[1].name.to_string(), "b");
    assert!(macro_def.parameters[1].default.is_some());
}
