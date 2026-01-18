use serde_xml_rs::from_str;
use xmile::view::View;

#[test]
fn test_basic_view_parsing() {
    let xml = r#"
    <view uid="1" type="stock_flow" width="800" height="600" page_width="800" page_height="600" page_sequence="row" page_orientation="landscape" show_pages="false" home_page="0" home_view="false">
    </view>
    "#;

    let view: View = from_str(xml).expect("Failed to parse basic view");

    use xmile::Uid;
    assert_eq!(view.uid, Uid::new(1));
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
    assert!(!view.show_pages);
    assert_eq!(view.home_page, 0);
    assert!(!view.home_view);
}

#[test]
fn test_view_with_defaults() {
    // Test that defaults work when attributes are missing
    let xml = r#"
    <view uid="2" width="400" height="300" page_width="400" page_height="300">
    </view>
    "#;

    let view: View = from_str(xml).expect("Failed to parse view with defaults");

    use xmile::Uid;
    assert_eq!(view.uid, Uid::new(2));
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
    assert!(!view.show_pages);
    // home_page should default to 0
    assert_eq!(view.home_page, 0);
    // home_view should default to false
    assert!(!view.home_view);
}

#[test]
fn test_view_type_parsing() {
    // Test different view types
    let xml_interface = r#"
    <view uid="3" type="interface" width="800" height="600" page_width="800" page_height="600">
    </view>
    "#;

    let view: View = from_str(xml_interface).expect("Failed to parse interface view");
    assert!(matches!(view.view_type, xmile::view::ViewType::Interface));

    let xml_popup = r#"
    <view uid="4" type="popup" width="800" height="600" page_width="800" page_height="600">
    </view>
    "#;

    let view: View = from_str(xml_popup).expect("Failed to parse popup view");
    assert!(matches!(view.view_type, xmile::view::ViewType::Popup));
}

#[test]
fn test_view_type_defaults_to_stock_flow() {
    // Test that when type is not specified, it defaults to stock_flow
    let xml = r#"
    <view uid="5" width="800" height="600" page_width="800" page_height="600">
    </view>
    "#;

    let view: View = from_str(xml).expect("Failed to parse view without type");
    assert!(matches!(view.view_type, xmile::view::ViewType::StockFlow));
}

#[test]
fn test_vendor_specific_view_type() {
    // Test vendor-specific view types
    let xml = r#"
    <view uid="6" type="vensim:custom_view" width="800" height="600" page_width="800" page_height="600">
    </view>
    "#;

    let view: View = from_str(xml).expect("Failed to parse vendor-specific view");
    match view.view_type {
        xmile::view::ViewType::VendorSpecific(vendor, type_part) => {
            assert_eq!(vendor, xmile::Vendor::Vensim);
            assert_eq!(type_part, "custom_view");
        }
        _ => panic!("Expected VendorSpecific view type"),
    }
}
