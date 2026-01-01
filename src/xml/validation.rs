//! Validation functions for XMILE structures

use std::collections::{HashMap, HashSet};

use crate::{
    model::vars::{Variable, Var},
    types::ValidationResult,
    Identifier, Uid,
};

/// Extract variable name from a Variable enum variant
pub fn get_variable_name(var: &Variable) -> Option<&Identifier> {
    match var {
        Variable::Auxiliary(aux) => aux.name(),
        Variable::Stock(stock) => match stock {
            crate::model::vars::stock::Stock::Basic(b) => b.name(),
            crate::model::vars::stock::Stock::Conveyor(c) => c.name(),
            crate::model::vars::stock::Stock::Queue(q) => q.name(),
        },
        Variable::Flow(flow) => flow.name(),
        Variable::GraphicalFunction(gf) => gf.name(),
        #[cfg(feature = "submodels")]
        Variable::Module(module) => module.name(),
        Variable::Group(group) => Some(&group.name),
    }
}

/// Validate that variable names are unique within a model
pub fn validate_variable_name_uniqueness(variables: &[Variable]) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    let mut seen_names: HashMap<String, Vec<usize>> = HashMap::new();
    
    for (idx, var) in variables.iter().enumerate() {
        if let Some(name) = get_variable_name(var) {
            let name_str = name.to_string();
            seen_names.entry(name_str).or_insert_with(Vec::new).push(idx);
        }
    }
    
    for (name, indices) in seen_names {
        if indices.len() > 1 {
            errors.push(format!(
                "Variable name '{}' appears {} times at indices: {:?}",
                name, indices.len(), indices
            ));
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

/// Validate that UIDs are unique within a view
pub fn validate_view_uids_unique(view: &crate::view::View) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    let mut seen_uids: HashMap<Uid, Vec<String>> = HashMap::new();
    
    // Collect all UIDs from view objects
    for stock in &view.stocks {
        seen_uids.entry(stock.uid).or_insert_with(Vec::new).push(format!("stock '{}'", stock.name));
    }
    for flow in &view.flows {
        seen_uids.entry(flow.uid).or_insert_with(Vec::new).push(format!("flow '{}'", flow.name));
    }
    for aux in &view.auxes {
        seen_uids.entry(aux.uid).or_insert_with(Vec::new).push(format!("aux '{}'", aux.name));
    }
    for module in &view.modules {
        seen_uids.entry(module.uid).or_insert_with(Vec::new).push(format!("module '{}'", module.name));
    }
    for group in &view.groups {
        seen_uids.entry(group.uid).or_insert_with(Vec::new).push(format!("group '{}'", group.name));
    }
    for connector in &view.connectors {
        seen_uids.entry(connector.uid).or_insert_with(Vec::new).push("connector".to_string());
    }
    for alias in &view.aliases {
        seen_uids.entry(alias.uid).or_insert_with(Vec::new).push(format!("alias '{}'", alias.of));
    }
    for slider in &view.sliders {
        seen_uids.entry(slider.uid).or_insert_with(Vec::new).push("slider".to_string());
    }
    for knob in &view.knobs {
        seen_uids.entry(knob.uid).or_insert_with(Vec::new).push("knob".to_string());
    }
    for switch in &view.switches {
        seen_uids.entry(switch.uid).or_insert_with(Vec::new).push("switch".to_string());
    }
    for options in &view.options {
        seen_uids.entry(options.uid).or_insert_with(Vec::new).push("options".to_string());
    }
    for numeric_input in &view.numeric_inputs {
        seen_uids.entry(numeric_input.uid).or_insert_with(Vec::new).push("numeric_input".to_string());
    }
    for list_input in &view.list_inputs {
        seen_uids.entry(list_input.uid).or_insert_with(Vec::new).push("list_input".to_string());
    }
    for graphical_input in &view.graphical_inputs {
        seen_uids.entry(graphical_input.uid).or_insert_with(Vec::new).push("graphical_input".to_string());
    }
    for numeric_display in &view.numeric_displays {
        seen_uids.entry(numeric_display.uid).or_insert_with(Vec::new).push("numeric_display".to_string());
    }
    for lamp in &view.lamps {
        seen_uids.entry(lamp.uid).or_insert_with(Vec::new).push("lamp".to_string());
    }
    for gauge in &view.gauges {
        seen_uids.entry(gauge.uid).or_insert_with(Vec::new).push("gauge".to_string());
    }
    for graph in &view.graphs {
        seen_uids.entry(graph.uid).or_insert_with(Vec::new).push("graph".to_string());
    }
    for table in &view.tables {
        seen_uids.entry(table.uid).or_insert_with(Vec::new).push("table".to_string());
    }
    for text_box in &view.text_boxes {
        seen_uids.entry(text_box.uid).or_insert_with(Vec::new).push("text_box".to_string());
    }
    for graphics_frame in &view.graphics_frames {
        seen_uids.entry(graphics_frame.uid).or_insert_with(Vec::new).push("graphics_frame".to_string());
    }
    for button in &view.buttons {
        seen_uids.entry(button.uid).or_insert_with(Vec::new).push("button".to_string());
    }
    
    // Check for duplicates
    for (uid, locations) in seen_uids {
        if locations.len() > 1 {
            errors.push(format!(
                "UID {} appears {} times in view: {}",
                uid.value,
                locations.len(),
                locations.join(", ")
            ));
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

/// Validate that dimension names used in variables exist in the dimensions definition
#[cfg(feature = "arrays")]
pub fn validate_dimension_references(
    variables: &[Variable],
    dimensions: &Option<crate::dimensions::Dimensions>,
) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    // Build set of defined dimension names
    let defined_dims: HashSet<String> = dimensions
        .as_ref()
        .map(|dims| {
            dims.dims
                .iter()
                .map(|dim| dim.name.clone())
                .collect()
        })
        .unwrap_or_default();
    
    // Check each variable's dimensions
    for var in variables {
        let var_name = get_variable_name(var)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let var_dims = match var {
            Variable::Auxiliary(aux) => aux.dimensions.as_ref().map(|d| {
                d.dims.iter().map(|dim| dim.name.clone()).collect::<Vec<_>>()
            }),
            Variable::Stock(stock) => match stock {
                crate::model::vars::stock::Stock::Basic(b) => b.dimensions.as_ref().map(|d| d.clone()),
                crate::model::vars::stock::Stock::Conveyor(c) => c.dimensions.as_ref().map(|d| d.clone()),
                crate::model::vars::stock::Stock::Queue(q) => q.dimensions.as_ref().map(|d| d.clone()),
            },
            Variable::Flow(flow) => flow.dimensions.as_ref().map(|d| d.clone()),
            Variable::GraphicalFunction(gf) => gf.dimensions.as_ref().map(|d| d.clone()),
            _ => None,
        };
        
        if let Some(dims) = var_dims {
            for dim_name in &dims {
                if !defined_dims.contains(dim_name) {
                    errors.push(format!(
                        "Variable '{}' references undefined dimension '{}'",
                        var_name, dim_name
                    ));
                }
            }
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

/// Validate that view object names match variable names
pub fn validate_view_object_references(
    view: &crate::view::View,
    variables: &[Variable],
) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    // Build set of variable names
    let var_names: HashSet<String> = variables
        .iter()
        .filter_map(|v| get_variable_name(v).map(|n| n.to_string()))
        .collect();
    
    // Check stock objects
    for stock_obj in &view.stocks {
        let obj_name = stock_obj.name.to_string();
        if !var_names.contains(&obj_name) {
            errors.push(format!(
                "Stock object '{}' (UID {}) does not reference a valid variable",
                obj_name, stock_obj.uid.value
            ));
        }
    }
    
    // Check flow objects
    for flow_obj in &view.flows {
        let obj_name = flow_obj.name.to_string();
        if !var_names.contains(&obj_name) {
            errors.push(format!(
                "Flow object '{}' (UID {}) does not reference a valid variable",
                obj_name, flow_obj.uid.value
            ));
        }
    }
    
    // Check aux objects
    for aux_obj in &view.auxes {
        let obj_name = aux_obj.name.to_string();
        if !var_names.contains(&obj_name) {
            errors.push(format!(
                "Aux object '{}' (UID {}) does not reference a valid variable",
                obj_name, aux_obj.uid.value
            ));
        }
    }
    
    // Check module objects
    for module_obj in &view.modules {
        let obj_name = module_obj.name.to_string();
        if !var_names.contains(&obj_name) {
            errors.push(format!(
                "Module object '{}' (UID {}) does not reference a valid variable",
                obj_name, module_obj.uid.value
            ));
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}

/// Validate that group entity references exist
pub fn validate_group_entity_references(
    groups: &[crate::model::groups::Group],
    variables: &[Variable],
) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    // Build set of variable names
    let var_names: HashSet<String> = variables
        .iter()
        .filter_map(|v| get_variable_name(v).map(|n| n.to_string()))
        .collect();
    
    // Check each group's entities
    for group in groups {
        let group_name = group.name.to_string();
        for entity in &group.entities {
            let entity_name = entity.name.to_string();
            if !var_names.contains(&entity_name) {
                errors.push(format!(
                    "Group '{}' references undefined entity '{}'",
                    group_name, entity_name
                ));
            }
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}
