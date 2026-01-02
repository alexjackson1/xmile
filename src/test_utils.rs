#![cfg(test)]

// Helper function to assert floating point equality with tolerance
pub fn assert_float_eq(a: f64, b: f64, tolerance: f64) {
    assert!(
        (a - b).abs() < tolerance,
        "Expected {} to be approximately equal to {} (tolerance: {})",
        a,
        b,
        tolerance
    );
}

/// Wrap a variable XML snippet in a minimal XMILE document for parsing.
pub fn wrap_variable_xml(variable_xml: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <vendor>Test</vendor>
        <product version="1.0">Test</product>
    </header>
    <model>
        <variables>
            {}
        </variables>
    </model>
</xmile>"#,
        variable_xml
    )
}

/// Parse a stock variable from XML snippet.
/// Wraps the XML in a minimal XMILE document and extracts the first stock variable.
/// Returns the Stock enum which could be Basic, Conveyor, or Queue.
pub fn parse_stock(stock_xml: &str) -> crate::model::vars::stock::Stock {
    use crate::model::vars::Variable;
    use crate::xml::XmileFile;

    let full_xml = wrap_variable_xml(stock_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let var = file.models[0]
        .variables
        .variables
        .first()
        .expect("No variables found in model");

    match var {
        Variable::Stock(stock) => stock.clone(),
        _ => panic!("Expected Stock variable, got {:?}", var),
    }
}

/// Parse a stock and expect it to be a BasicStock.
pub fn parse_basic_stock(stock_xml: &str) -> crate::model::vars::stock::BasicStock {
    use crate::model::vars::stock::Stock;

    match parse_stock(stock_xml) {
        Stock::Basic(basic) => basic,
        other => panic!("Expected BasicStock, got {:?}", other),
    }
}

/// Parse a stock and expect it to be a ConveyorStock.
pub fn parse_conveyor_stock(stock_xml: &str) -> crate::model::vars::stock::ConveyorStock {
    use crate::model::vars::stock::Stock;

    match parse_stock(stock_xml) {
        Stock::Conveyor(conveyor) => conveyor,
        other => panic!("Expected ConveyorStock, got {:?}", other),
    }
}

/// Parse a stock and expect it to be a QueueStock.
pub fn parse_queue_stock(stock_xml: &str) -> crate::model::vars::stock::QueueStock {
    use crate::model::vars::stock::Stock;

    match parse_stock(stock_xml) {
        Stock::Queue(queue) => queue,
        other => panic!("Expected QueueStock, got {:?}", other),
    }
}

/// Parse a flow variable from XML snippet.
/// Wraps the XML in a minimal XMILE document and extracts the first flow variable.
pub fn parse_flow(flow_xml: &str) -> crate::model::vars::flow::BasicFlow {
    use crate::model::vars::Variable;
    use crate::xml::XmileFile;

    let full_xml = wrap_variable_xml(flow_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let var = file.models[0]
        .variables
        .variables
        .first()
        .expect("No variables found in model");

    match var {
        Variable::Flow(flow) => flow.clone(),
        _ => panic!("Expected Flow variable, got {:?}", var),
    }
}

/// Parse a graphical function from XML snippet.
/// Wraps the XML in a minimal XMILE document and extracts the first graphical function.
pub fn parse_graphical_function(gf_xml: &str) -> crate::model::vars::gf::GraphicalFunction {
    use crate::model::vars::Variable;
    use crate::xml::XmileFile;

    let full_xml = wrap_variable_xml(gf_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let var = file.models[0]
        .variables
        .variables
        .first()
        .expect("No variables found in model");

    match var {
        Variable::GraphicalFunction(gf) => gf.clone(),
        _ => panic!("Expected GraphicalFunction variable, got {:?}", var),
    }
}

/// Parse an auxiliary variable from XML snippet.
/// Wraps the XML in a minimal XMILE document and extracts the first auxiliary variable.
pub fn parse_auxiliary(aux_xml: &str) -> crate::model::vars::aux::Auxiliary {
    use crate::model::vars::Variable;
    use crate::xml::XmileFile;

    let full_xml = wrap_variable_xml(aux_xml);
    let file = XmileFile::from_str(&full_xml).expect("Failed to parse XMILE file");

    let var = file.models[0]
        .variables
        .variables
        .first()
        .expect("No variables found in model");

    match var {
        Variable::Auxiliary(aux) => aux.clone(),
        _ => panic!("Expected Auxiliary variable, got {:?}", var),
    }
}
