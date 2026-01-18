use serde::{Deserialize, Deserializer, Serialize};

fn default_xmlns() -> String {
    "http://docs.oasis-open.org/xmile/ns/XMILE/v1.0".to_string()
}

use crate::{
    behavior::Behavior,
    data::Data,
    dimensions::Dimensions,
    header::Header,
    model::vars::Variable,
    model::vars::flow::Flow,
    model::vars::gf::{GraphicalFunction, GraphicalFunctionRegistry},
    model::vars::stock::Stock,
    specs::SimulationSpecs,
    types::{Validate, ValidationResult},
    units::ModelUnits,
    view::{Style, View},
    xml::validation::*,
};

#[cfg(feature = "macros")]
use crate::r#macro::{Macro, MacroRegistry};

#[cfg(feature = "arrays")]
use crate::model::vars::ArrayRegistry;

/// A XMILE file contains information about a whole-model, with a
/// well-specified structure. The file MUST be encoded in UTF-8. The entire
/// XMILE file is enclosed within a <xmile> tag as follows:
///
/// ```xml
/// <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
///    ...
/// </xmile>
/// ```
///
/// The version number MUST refer to the version of XMILE used (presently 1.0).
/// The XML namespace refers to tags and attributes used in this specification.
/// Both of these attributes are REQUIRED. Inside of the <xmile> tag are a
/// number of top-level tags, listed below. These tags are marked req (a single
/// instance is REQUIRED), opt (a single instance is OPTIONAL), * (zero or more
/// tags MAY occur) and + (one or more tags MAY occur). Top level tags MAY
/// occur in any order, but are RECOMMENDED to occur in the following order:
///
/// - `<header>` (req) - information about the origin of the file and required
///   capabilities.
/// - `<sim_specs>` (opt) - default simulation specifications for this file.
/// - `<model_units>` (opt) - definitions of model units used in this file.
/// - `<dimensions>` (opt) - definitions of array dimensions specific to this
///   file.
/// - `<behavior>` (opt) - simulation style definitions that are
///   inherited/cascaded through all models defined in this XMILE file.
/// - `<style>` (opt) - display style definitions that are inherited/cascaded
///   through all views defined in this XMILE file.
/// - `<data>` (opt) - definitions of persistent data import/export
///   connections.
/// - `<model>+` - definition of model equations and (optionally) diagrams.
/// - `<macro>*` - definition of macros that can be used in model equations.
///
/// These tags are specified in the subsequent sections of this chapter, after
/// XMILE namespaces are discussed.
///
/// When an XMILE file includes references to models contained in separate files
/// or at a specific URL, each such file may contain overlapping information,
/// most commonly in sim_specs, model_units and dimensions. When such overlap
/// is consistent, combining parts is done by taking the union of the different
/// component files. When an inconsistency is found, (for example, a dimension
/// with two distinct definitions) software reading the files MUST resolve the
/// inconsistency and SHOULD provide user feedback in doing so. Some
/// inconsistencies, such as conflicting Macro or Model names MUST be resolved
/// as detailed in section 2.11.3.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "xmile")]
pub struct XmileFile {
    /// The version of the XMILE specification used in this file.
    #[serde(rename = "@version")]
    pub version: String,
    /// The XML namespace for XMILE.
    #[serde(rename = "@xmlns", default = "default_xmlns")]
    pub xmlns: String,
    /// The header information for the XMILE file.
    pub header: Header,
    /// Optional simulation specifications for the XMILE file.
    pub sim_specs: Option<SimulationSpecs>,
    /// Optional model units defined in the XMILE file.
    pub model_units: Option<ModelUnits>,
    /// Optional dimensions defined in the XMILE file.
    pub dimensions: Option<Dimensions>,
    /// Optional behavior specifications for the XMILE file.
    pub behavior: Option<Behavior>,
    /// Optional style definitions for the XMILE file.
    pub style: Option<Style>,
    /// Optional data definitions for the XMILE file.
    pub data: Option<Data>,
    /// A list of models defined in the XMILE file.
    #[serde(rename = "model")]
    pub models: Vec<Model>,
    /// A list of macros defined in the XMILE file.
    #[cfg(feature = "macros")]
    #[serde(rename = "macro", default)]
    pub macros: Vec<Macro>,
}

/// The overall structure of a <model> tag appears below (sub-tags MUST appear in this order):
///
/// ```xml
/// <model name="..." resource="...">
///    <sim_specs>    <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </sim_specs>
///    <behavior>     <!-- OPTIONAL – see Chapter 2 -->
///      ...
///    </behavior>
///    <variables>    <!-- REQUIRED -->
///      ...
///    </variables>
///    <views>        <!-- OPTIONAL – see Chapters 5 & 6 -->
///      ...
///    </views>
/// </model>
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Optional name attribute for the model (required if model is a submodel).
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// Optional resource attribute referencing an external file containing the model.
    #[serde(rename = "@resource")]
    pub resource: Option<String>,
    /// Optional simulation specifications for this model.
    pub sim_specs: Option<SimulationSpecs>,
    /// Optional behavior specifications for this model.
    pub behavior: Option<Behavior>,
    /// The variables defined in this model (REQUIRED).
    pub variables: Variables,
    /// Optional views for this model.
    pub views: Option<Views>,
}

impl XmileFile {
    /// Builds a macro registry from the macros defined in this file.
    ///
    /// Returns an empty registry if there are no macros (still useful for checking if macros exist).
    #[cfg(feature = "macros")]
    pub fn build_macro_registry(&self) -> MacroRegistry {
        if self.macros.is_empty() {
            MacroRegistry::new()
        } else {
            MacroRegistry::from_macros(&self.macros)
        }
    }

    /// Resolves all function calls in expressions throughout all models in this file.
    ///
    /// This method builds registries from macros and model variables, then resolves
    /// all function calls in expressions to use the correct FunctionTarget variants.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all expressions were resolved successfully, or a vector of error messages if any resolution failed.
    #[cfg(all(feature = "macros", feature = "arrays"))]
    pub fn resolve_all_expressions(&mut self) -> Result<(), Vec<String>> {
        let macro_registry = self.build_macro_registry();
        let macro_registry_ref = if self.macros.is_empty() {
            None
        } else {
            Some(&macro_registry)
        };

        let mut all_errors = Vec::new();
        for model in &mut self.models {
            let gf_registry = model.build_gf_registry();
            let array_registry = Some(model.build_array_registry());

            if let Err(errors) = model.resolve_all_expressions(
                macro_registry_ref,
                &gf_registry,
                array_registry.as_ref(),
            ) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Resolves all function calls in expressions throughout all models in this file.
    ///
    /// This is the version when `macros` is enabled but `arrays` is not.
    #[cfg(all(feature = "macros", not(feature = "arrays")))]
    pub fn resolve_all_expressions(&mut self) -> Result<(), Vec<String>> {
        let macro_registry = self.build_macro_registry();
        let macro_registry_ref = if self.macros.is_empty() {
            None
        } else {
            Some(&macro_registry)
        };

        let mut all_errors = Vec::new();
        for model in &mut self.models {
            let gf_registry = model.build_gf_registry();

            if let Err(errors) = model.resolve_all_expressions(macro_registry_ref, &gf_registry) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Resolves all function calls in expressions throughout all models in this file.
    ///
    /// This is the version when `macros` is disabled but `arrays` is enabled.
    #[cfg(all(not(feature = "macros"), feature = "arrays"))]
    pub fn resolve_all_expressions(&mut self) -> Result<(), Vec<String>> {
        let mut all_errors = Vec::new();
        for model in &mut self.models {
            let gf_registry = model.build_gf_registry();
            let array_registry = Some(model.build_array_registry());

            if let Err(errors) =
                model.resolve_all_expressions(&gf_registry, array_registry.as_ref())
            {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Resolves all function calls in expressions throughout all models in this file.
    ///
    /// This is the version when both `macros` and `arrays` features are disabled.
    #[cfg(all(not(feature = "macros"), not(feature = "arrays")))]
    pub fn resolve_all_expressions(&mut self) -> Result<(), Vec<String>> {
        let mut all_errors = Vec::new();
        for model in &mut self.models {
            let gf_registry = model.build_gf_registry();

            if let Err(errors) = model.resolve_all_expressions(&gf_registry) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}

impl Model {
    /// Builds a graphical function registry from the variables in this model.
    /// Only named graphical functions are included in the registry.
    pub fn build_gf_registry(&self) -> GraphicalFunctionRegistry {
        let gfs: Vec<GraphicalFunction> = self
            .variables
            .variables
            .iter()
            .filter_map(|v| {
                if let Variable::GraphicalFunction(gf) = v {
                    Some(gf.clone())
                } else {
                    None
                }
            })
            .collect();
        GraphicalFunctionRegistry::from_functions(&gfs)
    }

    /// Builds an array registry from the variables in this model.
    /// Returns `None` if the arrays feature is not enabled.
    #[cfg(feature = "arrays")]
    pub fn build_array_registry(&self) -> ArrayRegistry {
        ArrayRegistry::from_variables(&self.variables.variables)
    }

    /// Resolves all function calls in expressions throughout this model using the provided registries.
    ///
    /// This method iterates through all variables and resolves function calls in their expressions,
    /// replacing them with properly resolved versions that use the correct FunctionTarget variants.
    ///
    /// # Arguments
    ///
    /// * `macro_registry` - Optional registry of macros (only available when `macros` feature is enabled)
    /// * `gf_registry` - Registry of named graphical functions
    /// * `array_registry` - Optional registry of array variables (only available when `arrays` feature is enabled)
    ///
    /// # Returns
    ///
    /// `Ok(())` if all expressions were resolved successfully, or a vector of error messages if any resolution failed.
    #[cfg(all(feature = "macros", feature = "arrays"))]
    pub fn resolve_all_expressions(
        &mut self,
        macro_registry: Option<&MacroRegistry>,
        gf_registry: &GraphicalFunctionRegistry,
        array_registry: Option<&ArrayRegistry>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for var in &mut self.variables.variables {
            match var {
                Variable::Auxiliary(aux) => {
                    match aux.equation.resolve_function_calls(
                        macro_registry,
                        Some(gf_registry),
                        array_registry,
                    ) {
                        Ok(resolved) => aux.equation = resolved,
                        Err(e) => errors.push(format!(
                            "Error resolving expression in auxiliary '{}': {}",
                            aux.name, e
                        )),
                    }
                    // Resolve expressions in array elements
                    #[cfg(feature = "arrays")]
                    for element in &mut aux.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(macro_registry, Some(gf_registry), array_registry) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => errors.push(format!("Error resolving expression in array element of auxiliary '{}': {}", aux.name, e)),
                            }
                        }
                    }
                }
                Variable::Stock(stock) => match stock.as_mut() {
                    Stock::Basic(basic) => {
                        match basic.initial_equation.resolve_function_calls(
                            macro_registry,
                            Some(gf_registry),
                            array_registry,
                        ) {
                            Ok(resolved) => basic.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in stock '{}': {}",
                                basic.name, e
                            )),
                        }
                        #[cfg(feature = "arrays")]
                        for element in &mut basic.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(macro_registry, Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of stock '{}': {}", basic.name, e)),
                                    }
                            }
                        }
                    }
                    Stock::Conveyor(conveyor) => {
                        match conveyor.initial_equation.resolve_function_calls(
                            macro_registry,
                            Some(gf_registry),
                            array_registry,
                        ) {
                            Ok(resolved) => conveyor.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in conveyor stock '{}': {}",
                                conveyor.name, e
                            )),
                        }
                        #[cfg(feature = "arrays")]
                        for element in &mut conveyor.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(macro_registry, Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of conveyor stock '{}': {}", conveyor.name, e)),
                                    }
                            }
                        }
                    }
                    Stock::Queue(queue) => {
                        match queue.initial_equation.resolve_function_calls(
                            macro_registry,
                            Some(gf_registry),
                            array_registry,
                        ) {
                            Ok(resolved) => queue.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in queue stock '{}': {}",
                                queue.name, e
                            )),
                        }
                        #[cfg(feature = "arrays")]
                        for element in &mut queue.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(macro_registry, Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of queue stock '{}': {}", queue.name, e)),
                                    }
                            }
                        }
                    }
                },
                Variable::Flow(flow) => {
                    if let Some(ref mut eqn) = flow.equation {
                        match eqn.resolve_function_calls(
                            macro_registry,
                            Some(gf_registry),
                            array_registry,
                        ) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in flow '{}': {}",
                                flow.name, e
                            )),
                        }
                    }
                    #[cfg(feature = "arrays")]
                    for element in &mut flow.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(
                                macro_registry,
                                Some(gf_registry),
                                array_registry,
                            ) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => errors.push(format!(
                                    "Error resolving expression in array element of flow '{}': {}",
                                    flow.name, e
                                )),
                            }
                        }
                    }
                }
                Variable::GraphicalFunction(gf) => {
                    if let Some(ref mut eqn) = gf.equation {
                        match eqn.resolve_function_calls(
                            macro_registry,
                            Some(gf_registry),
                            array_registry,
                        ) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => {
                                let name = gf
                                    .name
                                    .as_ref()
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "unnamed".to_string());
                                errors.push(format!(
                                    "Error resolving expression in graphical function '{}': {}",
                                    name, e
                                ));
                            }
                        }
                    }
                    #[cfg(feature = "arrays")]
                    for element in &mut gf.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(
                                macro_registry,
                                Some(gf_registry),
                                array_registry,
                            ) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => {
                                    let name = gf
                                        .name
                                        .as_ref()
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|| "unnamed".to_string());
                                    errors.push(format!("Error resolving expression in array element of graphical function '{}': {}", name, e));
                                }
                            }
                        }
                    }
                }
                #[cfg(feature = "submodels")]
                Variable::Module(_) => {
                    // Modules may have expressions, but we'll handle them separately if needed
                }
                Variable::Group(_) => {
                    // Groups don't have expressions
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Resolves all function calls in expressions throughout this model using the provided registries.
    ///
    /// This is the version when `macros` is enabled but `arrays` is not.
    #[cfg(all(feature = "macros", not(feature = "arrays")))]
    pub fn resolve_all_expressions(
        &mut self,
        macro_registry: Option<&MacroRegistry>,
        gf_registry: &GraphicalFunctionRegistry,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for var in &mut self.variables.variables {
            match var {
                Variable::Auxiliary(aux) => {
                    match aux
                        .equation
                        .resolve_function_calls(macro_registry, Some(gf_registry))
                    {
                        Ok(resolved) => aux.equation = resolved,
                        Err(e) => errors.push(format!(
                            "Error resolving expression in auxiliary '{}': {}",
                            aux.name, e
                        )),
                    }
                }
                Variable::Stock(stock) => match stock.as_ref() {
                    Stock::Basic(basic) => {
                        match basic
                            .initial_equation
                            .resolve_function_calls(macro_registry, Some(gf_registry))
                        {
                            Ok(resolved) => basic.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in stock '{}': {}",
                                basic.name, e
                            )),
                        }
                    }
                    Stock::Conveyor(conveyor) => {
                        match conveyor
                            .initial_equation
                            .resolve_function_calls(macro_registry, Some(gf_registry))
                        {
                            Ok(resolved) => conveyor.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in conveyor stock '{}': {}",
                                conveyor.name, e
                            )),
                        }
                    }
                    Stock::Queue(queue) => {
                        match queue
                            .initial_equation
                            .resolve_function_calls(macro_registry, Some(gf_registry))
                        {
                            Ok(resolved) => queue.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in queue stock '{}': {}",
                                queue.name, e
                            )),
                        }
                    }
                },
                Variable::Flow(flow) => {
                    if let Some(ref mut eqn) = flow.equation {
                        match eqn.resolve_function_calls(macro_registry, Some(gf_registry)) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in flow '{}': {}",
                                flow.name, e
                            )),
                        }
                    }
                }
                Variable::GraphicalFunction(gf) => {
                    if let Some(ref mut eqn) = gf.equation {
                        match eqn.resolve_function_calls(macro_registry, Some(gf_registry)) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => {
                                let name = gf
                                    .name
                                    .as_ref()
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "unnamed".to_string());
                                errors.push(format!(
                                    "Error resolving expression in graphical function '{}': {}",
                                    name, e
                                ));
                            }
                        }
                    }
                }
                #[cfg(feature = "submodels")]
                Variable::Module(_) => {}
                Variable::Group(_) => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Resolves all function calls in expressions throughout this model using the provided registries.
    ///
    /// This is the version when `macros` is disabled but `arrays` is enabled.
    #[cfg(all(not(feature = "macros"), feature = "arrays"))]
    pub fn resolve_all_expressions(
        &mut self,
        gf_registry: &GraphicalFunctionRegistry,
        array_registry: Option<&ArrayRegistry>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for var in &mut self.variables.variables {
            match var {
                Variable::Auxiliary(aux) => {
                    match aux
                        .equation
                        .resolve_function_calls(Some(gf_registry), array_registry)
                    {
                        Ok(resolved) => aux.equation = resolved,
                        Err(e) => errors.push(format!(
                            "Error resolving expression in auxiliary '{}': {}",
                            aux.name, e
                        )),
                    }
                    for element in &mut aux.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => errors.push(format!("Error resolving expression in array element of auxiliary '{}': {}", aux.name, e)),
                            }
                        }
                    }
                }
                Variable::Stock(stock) => match stock.as_ref() {
                    Stock::Basic(basic) => {
                        match basic
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry), array_registry)
                        {
                            Ok(resolved) => basic.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in stock '{}': {}",
                                basic.name, e
                            )),
                        }
                        for element in &mut basic.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of stock '{}': {}", basic.name, e)),
                                    }
                            }
                        }
                    }
                    Stock::Conveyor(conveyor) => {
                        match conveyor
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry), array_registry)
                        {
                            Ok(resolved) => conveyor.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in conveyor stock '{}': {}",
                                conveyor.name, e
                            )),
                        }
                        for element in &mut conveyor.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of conveyor stock '{}': {}", conveyor.name, e)),
                                    }
                            }
                        }
                    }
                    Stock::Queue(queue) => {
                        match queue
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry), array_registry)
                        {
                            Ok(resolved) => queue.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in queue stock '{}': {}",
                                queue.name, e
                            )),
                        }
                        for element in &mut queue.elements {
                            if let Some(ref mut eqn) = element.eqn {
                                match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                        Ok(resolved) => *eqn = resolved,
                                        Err(e) => errors.push(format!("Error resolving expression in array element of queue stock '{}': {}", queue.name, e)),
                                    }
                            }
                        }
                    }
                },
                Variable::Flow(flow) => {
                    if let Some(ref mut eqn) = flow.equation {
                        match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in flow '{}': {}",
                                flow.name, e
                            )),
                        }
                    }
                    for element in &mut flow.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => errors.push(format!(
                                    "Error resolving expression in array element of flow '{}': {}",
                                    flow.name, e
                                )),
                            }
                        }
                    }
                }
                Variable::GraphicalFunction(gf) => {
                    if let Some(ref mut eqn) = gf.equation {
                        match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => {
                                let name = gf
                                    .name
                                    .as_ref()
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "unnamed".to_string());
                                errors.push(format!(
                                    "Error resolving expression in graphical function '{}': {}",
                                    name, e
                                ));
                            }
                        }
                    }
                    for element in &mut gf.elements {
                        if let Some(ref mut eqn) = element.eqn {
                            match eqn.resolve_function_calls(Some(gf_registry), array_registry) {
                                Ok(resolved) => *eqn = resolved,
                                Err(e) => {
                                    let name = gf
                                        .name
                                        .as_ref()
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|| "unnamed".to_string());
                                    errors.push(format!("Error resolving expression in array element of graphical function '{}': {}", name, e));
                                }
                            }
                        }
                    }
                }
                #[cfg(feature = "submodels")]
                Variable::Module(_) => {}
                Variable::Group(_) => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Resolves all function calls in expressions throughout this model using the provided registries.
    ///
    /// This is the version when both `macros` and `arrays` features are disabled.
    #[cfg(all(not(feature = "macros"), not(feature = "arrays")))]
    pub fn resolve_all_expressions(
        &mut self,
        gf_registry: &GraphicalFunctionRegistry,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for var in &mut self.variables.variables {
            match var {
                Variable::Auxiliary(aux) => {
                    match aux.equation.resolve_function_calls(Some(gf_registry)) {
                        Ok(resolved) => aux.equation = resolved,
                        Err(e) => errors.push(format!(
                            "Error resolving expression in auxiliary '{}': {}",
                            aux.name, e
                        )),
                    }
                }
                Variable::Stock(stock) => match stock.as_mut() {
                    Stock::Basic(basic) => {
                        match basic
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry))
                        {
                            Ok(resolved) => basic.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in stock '{}': {}",
                                basic.name, e
                            )),
                        }
                    }
                    Stock::Conveyor(conveyor) => {
                        match conveyor
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry))
                        {
                            Ok(resolved) => conveyor.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in conveyor stock '{}': {}",
                                conveyor.name, e
                            )),
                        }
                    }
                    Stock::Queue(queue) => {
                        match queue
                            .initial_equation
                            .resolve_function_calls(Some(gf_registry))
                        {
                            Ok(resolved) => queue.initial_equation = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in queue stock '{}': {}",
                                queue.name, e
                            )),
                        }
                    }
                },
                Variable::Flow(flow) => {
                    if let Some(ref mut eqn) = flow.equation {
                        match eqn.resolve_function_calls(Some(gf_registry)) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => errors.push(format!(
                                "Error resolving expression in flow '{}': {}",
                                flow.name, e
                            )),
                        }
                    }
                }
                Variable::GraphicalFunction(gf) => {
                    if let Some(ref mut eqn) = gf.equation {
                        match eqn.resolve_function_calls(Some(gf_registry)) {
                            Ok(resolved) => *eqn = resolved,
                            Err(e) => {
                                let name = gf
                                    .name
                                    .as_ref()
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "unnamed".to_string());
                                errors.push(format!(
                                    "Error resolving expression in graphical function '{}': {}",
                                    name, e
                                ));
                            }
                        }
                    }
                }
                #[cfg(feature = "submodels")]
                Variable::Module(_) => {}
                Variable::Group(_) => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for Model {
    fn validate(&self) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate variable name uniqueness
        match validate_variable_name_uniqueness(&self.variables.variables) {
            ValidationResult::Valid(_) => {}
            ValidationResult::Warnings(_, warns) => warnings.extend(warns),
            ValidationResult::Invalid(warns, errs) => {
                warnings.extend(warns);
                errors.extend(errs);
            }
        }

        // Validate that all function calls are properly resolved
        // Note: This validation uses only model-level registries (GFs and arrays).
        // Macro validation happens at the file level since macros are file-level.
        let gf_registry = self.build_gf_registry();

        #[cfg(all(feature = "macros", feature = "arrays"))]
        {
            // Note: We can't validate macros here since they're file-level, not model-level
            // Macro validation should happen at XmileFile::validate()
            let array_registry = Some(self.build_array_registry());

            for var in &self.variables.variables {
                let validation_errors = match var {
                    Variable::Auxiliary(aux) => aux.equation.validate_resolved(
                        None,
                        Some(&gf_registry),
                        array_registry.as_ref(),
                    ),
                    Variable::Stock(stock) => match stock.as_ref() {
                        Stock::Basic(basic) => basic.initial_equation.validate_resolved(
                            None,
                            Some(&gf_registry),
                            array_registry.as_ref(),
                        ),
                        Stock::Conveyor(conveyor) => conveyor.initial_equation.validate_resolved(
                            None,
                            Some(&gf_registry),
                            array_registry.as_ref(),
                        ),
                        Stock::Queue(queue) => queue.initial_equation.validate_resolved(
                            None,
                            Some(&gf_registry),
                            array_registry.as_ref(),
                        ),
                    },
                    Variable::Flow(flow) => {
                        if let Some(ref eqn) = flow.equation {
                            eqn.validate_resolved(None, Some(&gf_registry), array_registry.as_ref())
                        } else {
                            Vec::new()
                        }
                    }
                    Variable::GraphicalFunction(gf) => {
                        if let Some(ref eqn) = gf.equation {
                            eqn.validate_resolved(None, Some(&gf_registry), array_registry.as_ref())
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                };
                errors.extend(validation_errors);
            }
        }

        #[cfg(all(feature = "macros", not(feature = "arrays")))]
        {
            // Note: We can't validate macros here since they're file-level, not model-level
            for var in &self.variables.variables {
                let validation_errors = match var {
                    Variable::Auxiliary(aux) => {
                        aux.equation.validate_resolved(None, Some(&gf_registry))
                    }
                    Variable::Stock(stock) => match stock {
                        Stock::Basic(basic) => basic
                            .initial_equation
                            .validate_resolved(None, Some(&gf_registry)),
                        Stock::Conveyor(conveyor) => conveyor
                            .initial_equation
                            .validate_resolved(None, Some(&gf_registry)),
                        Stock::Queue(queue) => queue
                            .initial_equation
                            .validate_resolved(None, Some(&gf_registry)),
                    },
                    Variable::Flow(flow) => {
                        if let Some(ref eqn) = flow.equation {
                            eqn.validate_resolved(None, Some(&gf_registry))
                        } else {
                            Vec::new()
                        }
                    }
                    Variable::GraphicalFunction(gf) => {
                        if let Some(ref eqn) = gf.equation {
                            eqn.validate_resolved(None, Some(&gf_registry))
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                };
                errors.extend(validation_errors);
            }
        }

        #[cfg(all(not(feature = "macros"), feature = "arrays"))]
        {
            let array_registry = Some(self.build_array_registry());

            for var in &self.variables.variables {
                let validation_errors = match var {
                    Variable::Auxiliary(aux) => aux
                        .equation
                        .validate_resolved(Some(&gf_registry), array_registry.as_ref()),
                    Variable::Stock(stock) => match stock {
                        Stock::Basic(basic) => basic
                            .initial_equation
                            .validate_resolved(Some(&gf_registry), array_registry.as_ref()),
                        Stock::Conveyor(conveyor) => conveyor
                            .initial_equation
                            .validate_resolved(Some(&gf_registry), array_registry.as_ref()),
                        Stock::Queue(queue) => queue
                            .initial_equation
                            .validate_resolved(Some(&gf_registry), array_registry.as_ref()),
                    },
                    Variable::Flow(flow) => {
                        if let Some(ref eqn) = flow.equation {
                            eqn.validate_resolved(Some(&gf_registry), array_registry.as_ref())
                        } else {
                            Vec::new()
                        }
                    }
                    Variable::GraphicalFunction(gf) => {
                        if let Some(ref eqn) = gf.equation {
                            eqn.validate_resolved(Some(&gf_registry), array_registry.as_ref())
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                };
                errors.extend(validation_errors);
            }
        }

        #[cfg(all(not(feature = "macros"), not(feature = "arrays")))]
        {
            for var in &self.variables.variables {
                let validation_errors = match var {
                    Variable::Auxiliary(aux) => aux.equation.validate_resolved(Some(&gf_registry)),
                    Variable::Stock(stock) => match stock.as_ref() {
                        Stock::Basic(basic) => {
                            basic.initial_equation.validate_resolved(Some(&gf_registry))
                        }
                        Stock::Conveyor(conveyor) => conveyor
                            .initial_equation
                            .validate_resolved(Some(&gf_registry)),
                        Stock::Queue(queue) => {
                            queue.initial_equation.validate_resolved(Some(&gf_registry))
                        }
                    },
                    Variable::Flow(flow) => {
                        if let Some(ref eqn) = flow.equation {
                            eqn.validate_resolved(Some(&gf_registry))
                        } else {
                            Vec::new()
                        }
                    }
                    Variable::GraphicalFunction(gf) => {
                        if let Some(ref eqn) = gf.equation {
                            eqn.validate_resolved(Some(&gf_registry))
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                };
                errors.extend(validation_errors);
            }
        }

        // Validate dimension references and array elements
        #[cfg(feature = "arrays")]
        {
            // Note: Model::validate() doesn't have access to file-level dimensions.
            // File-level dimension merging is handled in XmileFile::validate().
            // For now, we can't validate array elements here without file-level context.
            // This will be handled at the file level.
            let merged_dimensions = None;
            match crate::xml::validation::validate_dimension_references(
                &self.variables.variables,
                &merged_dimensions,
            ) {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                ValidationResult::Invalid(warns, errs) => {
                    warnings.extend(warns);
                    errors.extend(errs);
                }
            }

            // Validate array elements for variables that have them
            for var in &self.variables.variables {
                let var_name = crate::xml::validation::get_variable_name(var)
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Extract dimensions and elements based on variable type
                // Note: Stock uses Vec<String> for dimensions, others use VariableDimensions
                use crate::model::vars::array::{Dimension, VariableDimensions};
                let (var_dims, elements): (
                    Option<VariableDimensions>,
                    Option<&Vec<crate::model::vars::array::ArrayElement>>,
                ) = match var {
                    Variable::Auxiliary(aux) => (aux.dimensions.clone(), Some(&aux.elements)),
                    Variable::Stock(stock) => match stock.as_ref() {
                        crate::model::vars::stock::Stock::Basic(b) => {
                            // Convert Vec<String> to VariableDimensions
                            let dims = b.dimensions.as_ref().map(|names| VariableDimensions {
                                dims: names
                                    .iter()
                                    .map(|name| Dimension { name: name.clone() })
                                    .collect(),
                            });
                            (dims, Some(&b.elements))
                        }
                        crate::model::vars::stock::Stock::Conveyor(c) => {
                            let dims = c.dimensions.as_ref().map(|names| VariableDimensions {
                                dims: names
                                    .iter()
                                    .map(|name| Dimension { name: name.clone() })
                                    .collect(),
                            });
                            (dims, Some(&c.elements))
                        }
                        crate::model::vars::stock::Stock::Queue(q) => {
                            let dims = q.dimensions.as_ref().map(|names| VariableDimensions {
                                dims: names
                                    .iter()
                                    .map(|name| Dimension { name: name.clone() })
                                    .collect(),
                            });
                            (dims, Some(&q.elements))
                        }
                    },
                    Variable::Flow(flow) => {
                        // Convert Vec<String> to VariableDimensions
                        let dims = flow.dimensions.as_ref().map(|names| VariableDimensions {
                            dims: names
                                .iter()
                                .map(|name| Dimension { name: name.clone() })
                                .collect(),
                        });
                        (dims, Some(&flow.elements))
                    }
                    Variable::GraphicalFunction(gf) => {
                        // Convert Vec<String> to VariableDimensions
                        let dims = gf.dimensions.as_ref().map(|names| VariableDimensions {
                            dims: names
                                .iter()
                                .map(|name| Dimension { name: name.clone() })
                                .collect(),
                        });
                        (dims, Some(&gf.elements))
                    }
                    _ => (None, None),
                };

                // If variable has dimensions and elements, validate them
                if let (Some(dims), Some(elems)) = (var_dims, elements) {
                    if !elems.is_empty() {
                        // Non-apply-to-all array: validate elements
                        match crate::xml::validation::validate_array_elements(
                            &var_name,
                            &dims,
                            elems,
                            &merged_dimensions,
                        ) {
                            ValidationResult::Valid(_) => {}
                            ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                            ValidationResult::Invalid(warns, errs) => {
                                warnings.extend(warns);
                                errors.extend(errs);
                            }
                        }
                    }
                }
            }
        }

        // Validate view object references
        if let Some(ref views) = self.views {
            for view in &views.views {
                match validate_view_object_references(view, &self.variables.variables) {
                    ValidationResult::Valid(_) => {}
                    ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                    ValidationResult::Invalid(warns, errs) => {
                        warnings.extend(warns);
                        errors.extend(errs);
                    }
                }

                // Validate UID uniqueness within each view
                match validate_view_uids_unique(view) {
                    ValidationResult::Valid(_) => {}
                    ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                    ValidationResult::Invalid(warns, errs) => {
                        warnings.extend(warns);
                        errors.extend(errs);
                    }
                }
            }
        }

        // Validate group entity references
        let groups: Vec<_> = self
            .variables
            .variables
            .iter()
            .filter_map(|v| {
                if let Variable::Group(g) = v {
                    Some(g.clone())
                } else {
                    None
                }
            })
            .collect();

        if !groups.is_empty() {
            match validate_group_entity_references(&groups, &self.variables.variables) {
                ValidationResult::Valid(_) => {}
                ValidationResult::Warnings(_, warns) => warnings.extend(warns),
                ValidationResult::Invalid(warns, errs) => {
                    warnings.extend(warns);
                    errors.extend(errs);
                }
            }
        }

        if errors.is_empty() {
            if warnings.is_empty() {
                ValidationResult::Valid(())
            } else {
                ValidationResult::Warnings((), warnings)
            }
        } else {
            ValidationResult::Invalid(warnings, errors)
        }
    }
}

/// A wrapper for deserializing variables from XML.
/// The <variables> tag contains a mix of <stock>, <flow>, <aux>, <gf>, and <module> tags.
#[derive(Debug, PartialEq, Clone)]
pub struct Variables {
    pub variables: Vec<Variable>,
}

impl Variables {
    pub fn new(variables: Vec<Variable>) -> Self {
        Variables { variables }
    }
}

// Custom deserialization for Variables to handle mixed tag names
// In XML, <variables> contains a mix of <stock>, <flow>, <aux>, <gf>, and <module> tags
impl<'de> Deserialize<'de> for Variables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct VariablesVisitor;

        impl<'de> Visitor<'de> for VariablesVisitor {
            type Value = Variables;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a variables element")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut variables = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "stock" => {
                            let stock: crate::model::vars::Stock = map.next_value()?;
                            variables.push(Variable::Stock(Box::new(stock)));
                        }
                        "flow" => {
                            let flow: Flow = map.next_value()?;
                            match flow {
                                Flow::Basic(basic) => {
                                    variables.push(Variable::Flow(basic));
                                }
                                _ => {
                                    return Err(de::Error::custom(
                                        "Only basic flows are supported in variables section",
                                    ));
                                }
                            }
                        }
                        "aux" => {
                            let aux: crate::model::vars::Auxiliary = map.next_value()?;
                            variables.push(Variable::Auxiliary(aux));
                        }
                        "gf" => {
                            let gf: crate::model::vars::GraphicalFunction = map.next_value()?;
                            variables.push(Variable::GraphicalFunction(gf));
                        }
                        #[cfg(feature = "submodels")]
                        "module" => {
                            let module: crate::model::vars::Module = map.next_value()?;
                            variables.push(Variable::Module(module));
                        }
                        "group" => {
                            let group: crate::model::groups::Group = map.next_value()?;
                            variables.push(Variable::Group(group));
                        }
                        _ => {
                            // Skip unknown tags
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(Variables { variables })
            }
        }

        deserializer.deserialize_map(VariablesVisitor)
    }
}

impl Serialize for Variables {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.variables.len()))?;

        for var in &self.variables {
            match var {
                Variable::Stock(stock) => {
                    map.serialize_entry("stock", stock)?;
                }
                Variable::Flow(flow) => {
                    map.serialize_entry("flow", flow)?;
                }
                Variable::Auxiliary(aux) => {
                    map.serialize_entry("aux", aux)?;
                }
                Variable::GraphicalFunction(gf) => {
                    map.serialize_entry("gf", gf)?;
                }
                #[cfg(feature = "submodels")]
                Variable::Module(module) => {
                    map.serialize_entry("module", module)?;
                }
                Variable::Group(group) => {
                    map.serialize_entry("group", group)?;
                }
            }
        }
        map.end()
    }
}

/// The <views> tag contains a list of one or many <view> tags which describes
/// the layout, content and appearance of the user interface and stock and flow diagram.
/// The <views> tag can also contain an OPTIONAL visible_view attribute specifying
/// the index of the view which the user desires to be active upon loading of the file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Views {
    /// The index of the view which should be active upon loading.
    /// The index refers to the full list of views regardless of the view's type.
    #[serde(rename = "@visible_view")]
    pub visible_view: Option<u32>,
    /// A list of views defined in this model.
    #[serde(rename = "view")]
    pub views: Vec<View>,
    /// Optional style definitions that apply to all views within this <views> tag.
    pub style: Option<Style>,
}
