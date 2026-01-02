//! Tests for view parsing through XMILE documents

use xmile::xml::XmileFile;

/// Helper to create a minimal XMILE document with a view
fn wrap_view_xml(view_xml: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables></variables>
        <views>
            {}
        </views>
    </model>
</xmile>"#,
        view_xml
    )
}

#[test]
fn test_basic_view_parsing() {
    let view_xml = r#"<view type="stock_flow" width="800" height="600" page_width="800" page_height="600" page_sequence="row" page_orientation="landscape" show_pages="false" home_page="0" home_view="false">
    </view>"#;

    let full_xml = wrap_view_xml(view_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let views = file.models[0].views.as_ref().expect("No views found");
    let view = &views.views[0];

    assert!(matches!(view.view_type, xmile::view::ViewType::StockFlow));
    assert_eq!(view.width, 800.0);
    assert_eq!(view.height, 600.0);
    assert_eq!(view.page_width, 800.0);
    assert_eq!(view.page_height, 600.0);
    assert_eq!(view.page_sequence, xmile::view::PageSequence::Row);
    assert_eq!(
        view.page_orientation,
        xmile::view::PageOrientation::Landscape
    );
    assert_eq!(view.show_pages, false);
    assert_eq!(view.home_page, 0);
    assert_eq!(view.home_view, false);
}

#[test]
fn test_view_with_defaults() {
    // Test that defaults work when attributes are missing
    let view_xml = r#"<view width="400" height="300" page_width="400" page_height="300">
    </view>"#;

    let full_xml = wrap_view_xml(view_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let views = file.models[0].views.as_ref().expect("No views found");
    let view = &views.views[0];

    // type should default to StockFlow
    assert!(matches!(view.view_type, xmile::view::ViewType::StockFlow));
    // page_sequence should default to Row
    assert_eq!(view.page_sequence, xmile::view::PageSequence::Row);
    // page_orientation should default to Landscape
    assert_eq!(
        view.page_orientation,
        xmile::view::PageOrientation::Landscape
    );
    // show_pages should default to false
    assert_eq!(view.show_pages, false);
    // home_page should default to 0
    assert_eq!(view.home_page, 0);
    // home_view should default to false
    assert_eq!(view.home_view, false);
}

#[test]
fn test_view_type_parsing() {
    // Test interface view type
    let interface_xml = r#"<view type="interface" width="800" height="600" page_width="800" page_height="600">
    </view>"#;

    let full_xml = wrap_view_xml(interface_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let views = file.models[0].views.as_ref().expect("No views found");
    let view = &views.views[0];
    assert!(matches!(view.view_type, xmile::view::ViewType::Interface));

    // Test popup view type
    let popup_xml = r#"<view type="popup" width="800" height="600" page_width="800" page_height="600">
    </view>"#;

    let full_xml = wrap_view_xml(popup_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let views = file.models[0].views.as_ref().expect("No views found");
    let view = &views.views[0];
    assert!(matches!(view.view_type, xmile::view::ViewType::Popup));
}

#[test]
fn test_view_type_defaults_to_stock_flow() {
    // Test that when type is not specified, it defaults to stock_flow
    let view_xml = r#"<view width="800" height="600" page_width="800" page_height="600">
    </view>"#;

    let full_xml = wrap_view_xml(view_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let views = file.models[0].views.as_ref().expect("No views found");
    let view = &views.views[0];
    assert!(matches!(view.view_type, xmile::view::ViewType::StockFlow));
}
