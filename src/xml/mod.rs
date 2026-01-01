// An XMILE view contains XMILE display objects. A view can be thought of as a page, or a screen of a model’s stock and flow diagram, or its interface. An XMILE view has an OPTIONAL type parameter to notify users of the model as to whether or not the view is a part of the stock and flow diagram, or if the view is a part of the user interface. When a type is not specified on a view it is RECOMMENDED that the view be classified as a “stock_flow” view.  Other acceptable view types are popup (for popup windows) or vendor specific types.

// Views which have the type “stock_flow” are assumed to contain display objects which make up the stock and flow diagram. Typical objects appearing in a “stock_flow” view are stocks, flows, auxiliaries, aliases, and connectors. “stock_flow” views can also contain objects which are normally associated with an interface like sliders, graphs, and tables.

// Views which have the optional type “interface” are assumed to contain display objects which make up the interactive learning environment or user interface for the model. Any display object can be present in a view with the user interface type with the REQUIRED exception of the canonical representation of a model variable object (see Section 5.1.1). Said another way, model variables cannot be defined in a view marked as being a part of the user interface.

// XMILE views also contain an OPTIONAL order attribute which represents the order that views should be presented within the application. The order attribute is an integer starting from 0 and counting up. The lower the order number the earlier it appears in the list. Views with the type “stock_flow” are ordered separately from views with the type “interface”. If any view does not contain an order number it is RECOMMENDED that one is be assigned based on its position within the <views> tag.

// XMILE views are also REQUIRED to have a width and a height measured in pixels.  These properties describe the size of a view.  Views are REQUIRED to be rectangular.   Views may also have an OPTIONAL zoom specified as a double where 100 is default, 200 is 2x bigger by a factor of 2 and 50 is smaller by a factor of 2. In addition views can also have OPTIONAL attributes for storing the scroll position called scroll_x and scroll_y.  The scroll position is the origin (top left corner) of the screen in model coordinates (that is, when zoom has not been applied).  Also, views may contain an OPTIONAL background attribute that may be specified either as a color or as an external image resource using a file://url.

// In order for XMILE views to be printed each view is REQUIRED to specify its paging.  Therefore the following attributes are REQUIRED:

//     page_width<double> - The width of a printed page
//     page_height<double> - The height of a printed page
//     page_sequence<string> “row|column” – The ordering of page numbers.  With sequence type row, numbers are assigned going from left to right, then top to bottom.  With the sequence being column pages are ordered from top to bottom then left to right.
//     page_orientation<string> “landscape|portrait” – The orientation of the view on the printed page.
//     show_pages<bool> - Whether or not the software overlays page breaks and page numbers on the screen.

// In order for XMILE views to be more easily navigated views are REQUIRED to specify:

//     home_page<int> default: 0- The index of the printed page which is shown when any link with the home_page target is executed
//     home_view<bool> default: false – A marker property which is used to determine which view is navigated to when any link with the target home_view is executed.  Only one view for each view type is allowed to be marked as the home_view.

// 5.1.1 Referencing variable objects in an XMILE view

// Any object appearing in the <variables> tag (stock, flow, auxiliary, module, and group) is RECOMMENDED to have a related <stock|flow|aux|module|group> tag in at least one of the <view> tags associated with its model in order to be XMILE compatible.  A <stock|flow|aux|module|group> tag is linked to its associated model equation object through the use of its REQUIRED “name” attribute (see sample XMILE in beginning of chapter 5). A <stock|flow|aux|module> (note: not group) tag representing a model variable MUST NOT appear more than once in a single <view> tag. A <stock|flow|aux|module> (note: not group) tag representing a model variable may appear in separate <view> tags, but support of this feature is OPTIONAL. It is RECOMMENDED that in the case where this feature is not supported the lowest order view (or first view encountered is order is not specified) containing a <stock|flow|aux|module> tag representing a model variable is treated as the canonical display for that object and that any other encountered <stock|flow|aux|module> tag in any other <view> tag associated with that model representing the same model variable be treated as an alias (described in section 6.1.7).
// 5.1.2 XMILE view assumptions and attributes

// All visual objects contained within an XMILE <view> are laid out on a 2D Cartesian coordinate space measured in pixels, where 0,0 is the top left of the screen and height runs down while width runs right. An example coordinate space map looks like:

// All display objects contained within an XMILE file MUST have the following attributes:

//     Position:  x="<double>", y="<double>"
//     Size:  arbitrary rectangle or, for specific objects listed below, a <shape> tag
//                 width="<double>", height="<double>"

// A <shape> tag is specified as a child tag of other tags that are themselves child of a <view>.  Specifically, shape tags allow stock, auxiliary, module, or alias objects to be represented using a different symbol then the RECOMMENDED; rectangle for a stock, circle for an auxiliary and rounded rectangle for a module. It is OPTIONAL for these four object types to specify a <shape> tag to change their representation from the defaults specified above to any valid value for a <shape> tag described below with the following REQUIRED exceptions:

//     A stock MUST NOT be represented using a circle.
//     An auxiliary or flow MUST NOT be represented using a rectangle except if the equation contains a function or macro which contains a stock.

// Shape tags contain a REQUIRED type attribute which describes the shape of the object owning the tag. Valid type values are: rectangle, circle, and name_only. Shapes of type rectangle have two REQUIRED attributes, width and height, and an OPTIONAL attribute, the corner radius, all specified as doubles in pixels. Shapes of type circle contain one REQUIRED attribute:  radius. Shapes of type name_only are special:  they contain two OPTIONAL attributes width and height both measured in pixels and represented using a double. The name_only shape specifies that the object shall be represented by its name plate only. The optional width and height attributes are used for line wrapping hints.  These hints are only suggestions and may be ignored without consequence.

// The position referred to by the x and y attributes refers to the center of the object when using a <shape> tag. When using an arbitrary size, the x and y attributes refer to the top left corner of the object. All locations and sizes of XMILE objects are REQUIRED to be represented using double precision numbers.
// 5.1.3 Referring to specific XMILE display objects

// Display objects do not have names or any other way to specifically refer to individual objects. Therefore any display object which is referred to anywhere else in the XMILE file MUST provide a uid="<int>" attribute. This attribute is a unique linearly increasing integer which gives each display object a way to be referred to specifically while reading in an XMILE file. UIDs are NOT REQUIRED to be stable across successive reads and writes. Objects requiring a uid are listed in Chapter 6 of this specification. UIDs MUST be unique per XMILE model.

pub mod schema;
pub mod validation;
pub mod errors;

pub use schema::{XmileFile, Model, Views};
pub use errors::{ErrorCollection, ErrorContext, ToXmileError, XmileError};

use std::io::Read;
use std::path::Path;
use std::fs::File;

use thiserror::Error;
use crate::types::{Validate, ValidationResult};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parsing error: {0}")]
    Xml(String),
    #[error("Deserialization error: {0}")]
    Deserialize(String),
}

impl XmileFile {
    /// Parse an XMILE file from a string.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_str(xml: &str) -> Result<Self, ParseError> {
        let mut file: XmileFile = serde_xml_rs::from_str(xml)
            .map_err(|e| ParseError::Deserialize(e.to_string()))?;
        
        // Automatically resolve function calls in expressions
        if let Err(errors) = file.resolve_all_expressions() {
            return Err(ParseError::Deserialize(
                format!("Error resolving function calls: {}", errors.join("; "))
            ));
        }
        
        Ok(file)
    }

    /// Parse an XMILE file from a string with enhanced error reporting.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_str_with_context(xml: &str) -> Result<Self, XmileError> {
        let mut file: XmileFile = serde_xml_rs::from_str(xml).map_err(|e| {
            // Try to extract line number from error message if available
            let error_str = e.to_string();
            let context = extract_context_from_error(&error_str);
            
            XmileError::Deserialize {
                message: error_str,
                context,
            }
        })?;
        
        // Automatically resolve function calls in expressions
        if let Err(resolution_errors) = file.resolve_all_expressions() {
            return Err(XmileError::Validation {
                message: format!("Error resolving function calls: {}", resolution_errors.join("; ")),
                context: ErrorContext::new(),
                warnings: Vec::new(),
                errors: resolution_errors,
            });
        }
        
        Ok(file)
    }

    /// Parse an XMILE file from a reader.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, ParseError> {
        let mut file: XmileFile = serde_xml_rs::from_reader(reader)
            .map_err(|e| ParseError::Deserialize(e.to_string()))?;
        
        // Automatically resolve function calls in expressions
        if let Err(errors) = file.resolve_all_expressions() {
            return Err(ParseError::Deserialize(
                format!("Error resolving function calls: {}", errors.join("; "))
            ));
        }
        
        Ok(file)
    }

    /// Parse an XMILE file from a reader with enhanced error reporting.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_reader_with_context<R: Read>(reader: R) -> Result<Self, XmileError> {
        let mut file: XmileFile = serde_xml_rs::from_reader(reader).map_err(|e| {
            let error_str = e.to_string();
            let context = extract_context_from_error(&error_str);
            
            XmileError::Deserialize {
                message: error_str,
                context,
            }
        })?;
        
        // Automatically resolve function calls in expressions
        if let Err(resolution_errors) = file.resolve_all_expressions() {
            return Err(XmileError::Validation {
                message: format!("Error resolving function calls: {}", resolution_errors.join("; ")),
                context: ErrorContext::new(),
                warnings: Vec::new(),
                errors: resolution_errors,
            });
        }
        
        Ok(file)
    }

    /// Parse an XMILE file from a file path.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    /// Parse an XMILE file from a file path with enhanced error reporting.
    /// 
    /// After parsing, function calls in expressions are automatically resolved
    /// using the registries built from macros and model variables.
    pub fn from_file_with_context<P: AsRef<Path>>(path: P) -> Result<Self, XmileError> {
        let path_buf = path.as_ref().to_path_buf();
        let file = File::open(&path_buf)?;
        
        let mut xmile_file: XmileFile = serde_xml_rs::from_reader(file).map_err(|e| {
            let error_str = e.to_string();
            let mut context = extract_context_from_error(&error_str);
            context.file_path = Some(path_buf);
            
            XmileError::Deserialize {
                message: error_str,
                context,
            }
        })?;
        
        // Automatically resolve function calls in expressions
        if let Err(resolution_errors) = xmile_file.resolve_all_expressions() {
            return Err(XmileError::Validation {
                message: format!("Error resolving function calls: {}", resolution_errors.join("; ")),
                context: ErrorContext::new(),
                warnings: Vec::new(),
                errors: resolution_errors,
            });
        }
        
        Ok(xmile_file)
    }

    /// Validate the parsed XMILE file and return detailed errors if validation fails.
    /// 
    /// This includes validation of:
    /// - Model structure and variable definitions
    /// - Expression resolution (macros, graphical functions, arrays)
    /// - Function call resolution validation
    pub fn validate(&self) -> Result<(), XmileError> {
        let mut error_collection = ErrorCollection::new();
        
        // Validate macro resolution at file level
        #[cfg(feature = "macros")]
        {
            let macro_registry = self.build_macro_registry();
            let macro_registry_ref = if self.macros.is_empty() { None } else { Some(&macro_registry) };
            
            for (idx, model) in self.models.iter().enumerate() {
                let gf_registry = model.build_gf_registry();
                #[cfg(feature = "arrays")]
                let array_registry = Some(model.build_array_registry());
                
                for var in &model.variables.variables {
                    use crate::model::vars::Variable;
                    let validation_errors = match var {
                        Variable::Auxiliary(aux) => {
                            #[cfg(feature = "arrays")]
                            {
                                aux.equation.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                            }
                            #[cfg(not(feature = "arrays"))]
                            {
                                aux.equation.validate_resolved(macro_registry_ref, Some(&gf_registry))
                            }
                        }
                        Variable::Stock(stock) => {
                            use crate::model::vars::stock::Stock;
                            match stock {
                                Stock::Basic(basic) => {
                                    #[cfg(feature = "arrays")]
                                    {
                                        basic.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                                    }
                                    #[cfg(not(feature = "arrays"))]
                                    {
                                        basic.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry))
                                    }
                                }
                                Stock::Conveyor(conveyor) => {
                                    #[cfg(feature = "arrays")]
                                    {
                                        conveyor.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                                    }
                                    #[cfg(not(feature = "arrays"))]
                                    {
                                        conveyor.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry))
                                    }
                                }
                                Stock::Queue(queue) => {
                                    #[cfg(feature = "arrays")]
                                    {
                                        queue.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                                    }
                                    #[cfg(not(feature = "arrays"))]
                                    {
                                        queue.initial_equation.validate_resolved(macro_registry_ref, Some(&gf_registry))
                                    }
                                }
                            }
                        }
                        Variable::Flow(flow) => {
                            if let Some(ref eqn) = flow.equation {
                                #[cfg(feature = "arrays")]
                                {
                                    eqn.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                                }
                                #[cfg(not(feature = "arrays"))]
                                {
                                    eqn.validate_resolved(macro_registry_ref, Some(&gf_registry))
                                }
                            } else {
                                Vec::new()
                            }
                        }
                        Variable::GraphicalFunction(gf) => {
                            if let Some(ref eqn) = gf.equation {
                                #[cfg(feature = "arrays")]
                                {
                                    eqn.validate_resolved(macro_registry_ref, Some(&gf_registry), array_registry.as_ref())
                                }
                                #[cfg(not(feature = "arrays"))]
                                {
                                    eqn.validate_resolved(macro_registry_ref, Some(&gf_registry))
                                }
                            } else {
                                Vec::new()
                            }
                        }
                        _ => Vec::new(),
                    };
                    
                    if !validation_errors.is_empty() {
                        let context = ErrorContext::new()
                            .with_parsing(format!("model[{}]", idx));
                        error_collection.push(XmileError::Validation {
                            message: format!("Expression resolution validation failed: {}", validation_errors.join("; ")),
                            context,
                            warnings: Vec::new(),
                            errors: validation_errors,
                        });
                    }
                }
            }
        }
        
        // Merge file-level and model-level dimensions for array validation
        #[cfg(feature = "arrays")]
        let file_dimensions = &self.dimensions;
        
        for (idx, model) in self.models.iter().enumerate() {
            let context = ErrorContext::new()
                .with_parsing(format!("model[{}]", idx));
            
            // Validate model with file-level dimensions for array validation
            #[cfg(feature = "arrays")]
            {
                // Merge file and model dimensions (model overrides file)
                use std::collections::HashMap;
                let dim_map: HashMap<String, crate::dimensions::Dimension> = file_dimensions
                    .as_ref()
                    .map(|dims| {
                        dims.dims.iter().map(|dim| (dim.name.clone(), dim.clone())).collect()
                    })
                    .unwrap_or_default();
                
                // Note: Model doesn't currently have a dimensions field, but if it did,
                // we would override file dimensions with model dimensions here
                
                let merged_dimensions = if dim_map.is_empty() {
                    None
                } else {
                    Some(crate::dimensions::Dimensions {
                        dims: dim_map.into_values().collect(),
                    })
                };
                
                // Validate array elements with merged dimensions
                use crate::model::vars::Variable;
                use crate::model::vars::array::{Dimension, VariableDimensions};
                
                for var in &model.variables.variables {
                    let var_name = crate::xml::validation::get_variable_name(var)
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let (var_dims, elements): (Option<VariableDimensions>, Option<&Vec<crate::model::vars::array::ArrayElement>>) = match var {
                        Variable::Auxiliary(aux) => {
                            (aux.dimensions.clone(), Some(&aux.elements))
                        }
                        Variable::Stock(stock) => match stock {
                            crate::model::vars::stock::Stock::Basic(b) => {
                                let dims = b.dimensions.as_ref().map(|names| {
                                    VariableDimensions {
                                        dims: names.iter().map(|name| Dimension { name: name.clone() }).collect(),
                                    }
                                });
                                (dims, Some(&b.elements))
                            }
                            crate::model::vars::stock::Stock::Conveyor(c) => {
                                let dims = c.dimensions.as_ref().map(|names| {
                                    VariableDimensions {
                                        dims: names.iter().map(|name| Dimension { name: name.clone() }).collect(),
                                    }
                                });
                                (dims, Some(&c.elements))
                            }
                            crate::model::vars::stock::Stock::Queue(q) => {
                                let dims = q.dimensions.as_ref().map(|names| {
                                    VariableDimensions {
                                        dims: names.iter().map(|name| Dimension { name: name.clone() }).collect(),
                                    }
                                });
                                (dims, Some(&q.elements))
                            }
                        },
                        Variable::Flow(flow) => {
                            let dims = flow.dimensions.as_ref().map(|names| {
                                VariableDimensions {
                                    dims: names.iter().map(|name| Dimension { name: name.clone() }).collect(),
                                }
                            });
                            (dims, Some(&flow.elements))
                        }
                        Variable::GraphicalFunction(gf) => {
                            let dims = gf.dimensions.as_ref().map(|names| {
                                VariableDimensions {
                                    dims: names.iter().map(|name| Dimension { name: name.clone() }).collect(),
                                }
                            });
                            (dims, Some(&gf.elements))
                        }
                        _ => (None, None),
                    };
                    
                    if let (Some(dims), Some(elems)) = (var_dims, elements) {
                        if !elems.is_empty() {
                            match crate::xml::validation::validate_array_elements(
                                &var_name,
                                &dims,
                                elems,
                                &merged_dimensions,
                            ) {
                                ValidationResult::Valid(_) => {}
                                ValidationResult::Warnings(_, warns) => {
                                    for warn in warns {
                                        error_collection.push(XmileError::Validation {
                                            message: warn.clone(),
                                            context: context.clone().with_parsing(format!("model[{}].variable[{}]", idx, var_name)),
                                            warnings: vec![warn],
                                            errors: Vec::new(),
                                        });
                                    }
                                }
                                ValidationResult::Invalid(warns, errs) => {
                                    error_collection.push(XmileError::Validation {
                                        message: format!("Array validation failed for variable '{}'", var_name),
                                        context: context.clone().with_parsing(format!("model[{}].variable[{}]", idx, var_name)),
                                        warnings: warns,
                                        errors: errs,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            let validation_result = model.validate();
            if validation_result.is_invalid() {
                error_collection.push(validation_result.to_xmile_error(context));
            }
        }
        
        if let Some(error) = error_collection.into_error() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// Extract context information from error messages (line numbers, etc.).
/// 
/// Since serde-xml-rs doesn't provide structured error information,
/// we parse the error message string to extract what context we can.
/// This function handles various error message patterns that serde-xml-rs
/// and underlying XML parsers may produce.
fn extract_context_from_error(error_str: &str) -> ErrorContext {
    let mut context = ErrorContext::new();
    
    // Try to extract line number from various patterns:
    // - "line X"
    // - "at line X"
    // - "on line X"
    // - "Line X:"
    // - "line X, column Y"
    // - "line X:Y" (line:column)
    
    // Pattern: "line X" (most common)
    if let Some(line_start) = error_str.find("line ") {
        let after_line = &error_str[line_start + 5..];
        // Find the end of the number (non-digit or colon)
        let end = after_line
            .char_indices()
            .find(|(_, c)| !c.is_ascii_digit() && *c != ':')
            .map(|(i, _)| i)
            .unwrap_or(after_line.len());
        
        if let Ok(line) = after_line[..end].parse::<usize>() {
            context.line = Some(line);
            
            // Check for column after colon: "line X:Y"
            if end < after_line.len() && after_line.as_bytes()[end] == b':' {
                let after_colon = &after_line[end + 1..];
                let col_end = after_colon
                    .char_indices()
                    .find(|(_, c)| !c.is_ascii_digit())
                    .map(|(i, _)| i)
                    .unwrap_or(after_colon.len());
                
                if let Ok(column) = after_colon[..col_end].parse::<usize>() {
                    context.column = Some(column);
                }
            }
        }
    }
    
    // Pattern: "column X" (if line wasn't found)
    if context.column.is_none() {
        if let Some(col_start) = error_str.find("column ") {
            let after_col = &error_str[col_start + 7..];
            let col_end = after_col
                .char_indices()
                .find(|(_, c)| !c.is_ascii_digit())
                .map(|(i, _)| i)
                .unwrap_or(after_col.len());
            
            if let Ok(column) = after_col[..col_end].parse::<usize>() {
                context.column = Some(column);
            }
        }
    }
    
    // Pattern: "at position X" (byte position, less useful but we can try)
    // This is less reliable but sometimes the only info available
    
    context
}
