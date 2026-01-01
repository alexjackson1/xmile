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
            let var_list = if indices.len() == 2 {
                format!("positions {} and {}", indices[0], indices[1])
            } else {
                format!("positions {}", indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "))
            };
            errors.push(format!(
                "Duplicate variable name '{}' found {} times in the model (at {}). Each variable must have a unique name. Consider renaming one or more of these variables.",
                name, indices.len(), var_list
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
                "Duplicate UID {} found {} times in the same view (used by: {}). Each display object in a view must have a unique UID. This is likely a serialization error.",
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
                "Stock display object '{}' (UID {}) references a variable that does not exist. Ensure the variable '{}' is defined in the <variables> section of the model.",
                obj_name, stock_obj.uid.value, obj_name
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
                "Auxiliary display object '{}' (UID {}) references a variable that does not exist. Ensure the variable '{}' is defined in the <variables> section of the model.",
                obj_name, aux_obj.uid.value, obj_name
            ));
        }
    }
    
    // Check module objects
    for module_obj in &view.modules {
        let obj_name = module_obj.name.to_string();
        if !var_names.contains(&obj_name) {
            errors.push(format!(
                "Module display object '{}' (UID {}) references a variable that does not exist. Ensure the variable '{}' is defined in the <variables> section of the model.",
                obj_name, module_obj.uid.value, obj_name
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
                    "Group '{}' references undefined entity '{}'. The entity must be defined as a variable in the <variables> section before it can be referenced in a group.",
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

/// Validate array elements for a variable.
/// 
/// This validates:
/// - Subscript indices match dimension bounds
/// - All required elements are present for non-apply-to-all arrays
/// - Element ordering and completeness
#[cfg(feature = "arrays")]
pub fn validate_array_elements(
    var_name: &str,
    var_dims: &crate::model::vars::array::VariableDimensions,
    elements: &[crate::model::vars::array::ArrayElement],
    dimensions: &Option<crate::dimensions::Dimensions>,
) -> ValidationResult {
    let warnings = Vec::new();
    let mut errors = Vec::new();
    
    // If no dimensions defined, can't validate
    let Some(dims) = dimensions else {
        return ValidationResult::Valid(());
    };
    
    // Build a map of dimension name to dimension definition
    let dim_map: HashMap<String, &crate::dimensions::Dimension> = dims
        .dims
        .iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    
    // Get the dimension definitions for this variable in order
    let var_dim_defs: Vec<&crate::dimensions::Dimension> = var_dims
        .dims
        .iter()
        .filter_map(|d| dim_map.get(&d.name))
        .copied()
        .collect();
    
    if var_dim_defs.len() != var_dims.dims.len() {
        let missing: Vec<String> = var_dims.dims
            .iter()
            .filter_map(|d| {
                if !dim_map.contains_key(&d.name) {
                    Some(d.name.clone())
                } else {
                    None
                }
            })
            .collect();
        errors.push(format!(
            "Variable '{}' references {} dimension(s) that are not defined: {}. Define these dimensions in the <dimensions> section before using them in variables.",
            var_name, missing.len(), missing.join(", ")
        ));
        return ValidationResult::Invalid(warnings, errors);
    }
    
    // Calculate expected total number of elements
    let expected_count: usize = var_dim_defs
        .iter()
        .map(|dim| dim.size())
        .product();
    
    // Parse and validate each element's subscript
    let mut seen_subscripts = HashSet::new();
    let mut parsed_elements = Vec::new();
    
    for (idx, element) in elements.iter().enumerate() {
        // Parse the subscript string (comma-separated indices)
        let indices: Vec<&str> = element.subscript.split(',').map(|s| s.trim()).collect();
        
        if indices.len() != var_dims.dims.len() {
            errors.push(format!(
                "Array element {} of variable '{}' has {} index(es) in subscript '{}', but the variable has {} dimension(s). The subscript must provide exactly one index per dimension, separated by commas (e.g., '0,1' for a 2D array).",
                idx, var_name, indices.len(), element.subscript, var_dims.dims.len()
            ));
            continue;
        }
        
        // Validate each index against its dimension
        for (index_str, dim_def) in indices.iter().zip(var_dim_defs.iter()) {
            if !dim_def.is_valid_index(index_str) {
                if let Some(size) = dim_def.size {
                    // Numbered dimension
                    errors.push(format!(
                        "Array element {} of variable '{}': index '{}' for dimension '{}' is out of bounds (must be 0 to {})",
                        idx, var_name, index_str, dim_def.name, size - 1
                    ));
                } else {
                    // Named dimension
                    let element_names = dim_def.element_names();
                    errors.push(format!(
                        "Array element {} of variable '{}': index '{}' for dimension '{}' is not a valid element name (valid: {:?})",
                        idx, var_name, index_str, dim_def.name, element_names
                    ));
                }
            }
        }
        
        // Check for duplicate subscripts
        if !seen_subscripts.insert(element.subscript.clone()) {
            errors.push(format!(
                "Array element {} of variable '{}': duplicate subscript '{}'. Each array element must have a unique subscript. Remove the duplicate element.",
                idx, var_name, element.subscript
            ));
        }
        
        parsed_elements.push((element.subscript.clone(), indices));
    }
    
    // Check completeness: for non-apply-to-all arrays, all elements must be present
    if elements.len() != expected_count {
        errors.push(format!(
            "Variable '{}' is a non-apply-to-all array but has {} elements, expected {} (dimensions: {:?})",
            var_name,
            elements.len(),
            expected_count,
            var_dims.dims.iter().map(|d| d.name.clone()).collect::<Vec<_>>()
        ));
    }
    
    // Check that each element has either eqn or gf (but not both, and at least one)
    for (idx, element) in elements.iter().enumerate() {
        match (&element.eqn, &element.gf) {
            (None, None) => {
                errors.push(format!(
                    "Array element {} of variable '{}' with subscript '{}' must have either an <eqn> (equation) or a <gf> (graphical function). Add one of these to define the element's value.",
                    idx, var_name, element.subscript
                ));
            }
            (Some(_), Some(_)) => {
                errors.push(format!(
                    "Array element {} of variable '{}' with subscript '{}' cannot have both an <eqn> (equation) and a <gf> (graphical function). Choose one: either use an equation OR use a graphical function, but not both.",
                    idx, var_name, element.subscript
                ));
            }
            _ => {} // Valid: has either eqn or gf but not both
        }
    }
    
    if errors.is_empty() {
        ValidationResult::Valid(())
    } else {
        ValidationResult::Invalid(warnings, errors)
    }
}
