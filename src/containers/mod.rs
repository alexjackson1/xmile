//! # XMILE Containers System
//!
//! This module defines the core container system for XMILE according to specification
//! section 3.2.4. It provides uniform interfaces and operations for all container types
//! in XMILE, ensuring consistent behaviour across graphical functions, arrays, conveyors,
//! and queues.
//!
//! ## XMILE Container Requirements
//!
//! According to the XMILE specification section 3.2.4:
//!
//! - **All containers are lists of numbers** with consistent syntax and operations
//! - **Uniform access patterns** using square bracket notation (section 3.7.1)
//! - **Built-in statistical functions** that apply to all container types
//! - **Size immutability** for graphical functions and arrays during simulation
//! - **Dynamic sizing** for conveyors (may change length) and queues (change as matter of course)
//!
//! ## Container Types in XMILE
//!
//! | Container Type      | Required | Size Behaviour     | Description                    |
//! |---------------------|----------|--------------------|--------------------------------|
//! | Graphical Functions | Yes      | Fixed during sim   | Lookup/table functions         |
//! | Arrays              | Optional | Fixed during sim   | Multi-dimensional data         |
//! | Conveyors           | Optional | May change length  | Material flow with delay       |
//! | Queues              | Optional | Changes during sim | First-in-first-out structures  |
//!
//! ## Core Container Interface
//!
//! The `Container` trait provides the fundamental interface that all XMILE containers
//! must implement:
//!
//! ```rust
//! use xmile::{
//!     Container,
//!     GraphicalFunction,
//!     GraphicalFunctionData,
//!     GraphicalFunctionType,
//!     Identifier
//! };
//!
//! fn analyse_container<T: Container>(container: &T) {
//!     // Uniform access across all container types
//!     println!("Length: {}", container.len());
//!     println!("Empty: {}", container.is_empty());
//!     
//!     // Statistical operations
//!     if let Some(mean) = container.mean() {
//!         println!("Mean: {}", mean);
//!     }
//!     
//!     if let Some((min, max)) = container.range() {
//!         println!("Range: {} to {}", min, max);
//!     }
//!     
//!     // Element access (square bracket notation)
//!     if !container.is_empty() {
//!         let first = &container[0];
//!         println!("First element: {}", first);
//!     }
//! }
//!
//! // Works with any XMILE container
//! let graphical_function = GraphicalFunction {
//!     name: Some(Identifier::parse_default("example").unwrap()),
//!     function_type: Some(GraphicalFunctionType::Continuous),
//!     data: GraphicalFunctionData::uniform_scale(
//!         (0.0, 3.0),
//!         vec![0.0, 1.0, 2.0, 3.0],
//!         Some((0.0, 3.0)),
//!     ),
//! };
//! analyse_container(&graphical_function);
//! ```
//!
//! ## Square Bracket Notation (Section 3.7.1)
//!
//! XMILE containers uniformly support index-based access using square brackets,
//! as specified in section 3.7.1:
//!
//! ```rust
//! use xmile::{Container, GraphicalFunction};
//!
//! let container = vec![1.0, 2.0, 3.0, 4.0];
//!
//! // Read access - zero-based indexing
//! let first_value = container[0];           // First element
//! let last_value = container[container.len() - 1];  // Last element
//! let middle = container[container.len() / 2];       // Middle element
//!
//! // The Container trait requires Index implementation for this to work
//! ```
//!
//! ## Mutable Container Operations
//!
//! For containers that support modification during model setup, the `ContainerMut`
//! trait extends the base interface with mutable access:
//!
//! ```rust
//! use xmile::{Container, ContainerMut, GraphicalFunction};
//!
//! let mut container = vec![0.0; 10];
//!
//! // Mutable element access during setup
//! container[0] = 1.0;           // Modify first element
//! container[5] = 0.75;          // Modify specific element
//!
//! // Still provides all immutable operations
//! let mean = container.mean();
//! ```
//!
//! ## Statistical Operations
//!
//! All containers provide built-in statistical functions as specified in section 3.2.4:
//!
//! ### Mean Calculation
//! ```rust
//! use xmile::Container;
//!
//! let container = vec![0.0, 1.0, 2.0, 3.0];
//!
//! match container.mean() {
//!     Some(mean) => println!("Average value: {}", mean),
//!     None => println!("Container is empty"),
//! }
//! ```
//!
//! ### Range Detection
//! ```rust
//! use xmile::Container;
//!
//! let container = vec![0.0, 1.0, 2.0, 3.0];
//!
//! // Individual min/max
//! let min_value = container.min();
//! let max_value = container.max();
//!
//! // Combined range for efficiency
//! match container.range() {
//!     Some((min, max)) => {
//!         println!("Values range from {} to {}", min, max);
//!         println!("Span: {}", max - min);
//!     },
//!     None => println!("No range - container is empty"),
//! }
//! ```
//!
//! ### Size and Emptiness
//! ```rust
//! let container = vec![0.0, 1.0, 2.0, 3.0];
//!
//! println!("Container size: {}", container.len());
//!
//! if container.is_empty() {
//!     println!("Container has no elements");
//! } else {
//!     println!("Container has {} elements", container.len());
//! }
//! ```
//!
//! ## Direct Value Access
//!
//! Containers provide direct access to their underlying numeric data:
//!
//! ```rust
//! use xmile::Container;
//!
//! let container = vec![0.0, 1.0, 2.0, 3.0];
//!
//! // Access underlying slice of values
//! let values: &[f64] = container.values();
//!
//! // Use with standard slice operations
//! for (index, value) in values.iter().enumerate() {
//!     println!("Element {}: {}", index, value);
//! }
//!
//! // Functional programming patterns
//! let sum: f64 = values.iter().sum();
//! let count = values.len();
//! let manual_mean = sum / count as f64;
//! ```
//!
//! ## Integration with XMILE Expression System
//!
//! The container system integrates seamlessly with XMILE expressions and functions:
//!
//! ```rust
//! // Containers work with built-in functions (when arrays are supported)
//! use xmile::Container;
//!
//! fn container_operations<T: Container>(container: &T) {
//!     // These operations would be available in the expression system:
//!     
//!     // Statistical functions
//!     let mean = container.mean();        // MEAN(container)
//!     let min = container.min();          // MIN(container)
//!     let max = container.max();          // MAX(container)
//!     
//!     // Size functions
//!     let size = container.len();         // SIZE(container)
//!     let empty = container.is_empty();   // EMPTY(container)
//!     
//!     // Element access
//!     if size > 0 {
//!         let first = &container[0];       // container[0]
//!         let last = &container[size - 1]; // container[SIZE(container)-1]
//!     }
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! The container system is designed for efficiency in simulation contexts:
//!
//! - **Zero-cost abstractions**: Traits compile to direct field access
//! - **Cache-friendly access**: Contiguous memory layout for numeric data
//! - **Minimal allocations**: Statistical operations use iterator-based calculations
//! - **Compile-time dispatch**: Generic functions eliminate runtime overhead
//!
//! ## Container Implementation Example
//!
//! Here's how a custom container type would implement the required traits:
//!
//! ```rust
//! use std::ops::{Index, IndexMut};
//! use xmile::{Container, ContainerMut};
//!
//! #[derive(Debug, Clone)]
//! pub struct CustomContainer {
//!     data: Vec<f64>,
//! }
//!
//! impl Container for CustomContainer {
//!     fn values(&self) -> &[f64] {
//!         &self.data
//!     }
//!
//!     fn len(&self) -> usize {
//!         self.data.len()
//!     }
//!
//!     fn mean(&self) -> Option<f64> {
//!         if self.data.is_empty() {
//!             None
//!         } else {
//!             Some(self.data.iter().sum::<f64>() / self.data.len() as f64)
//!         }
//!     }
//!
//!     fn min(&self) -> Option<f64> {
//!         self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied()
//!     }
//!
//!     fn max(&self) -> Option<f64> {
//!         self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
//!     }
//! }
//!
//! impl Index<usize> for CustomContainer {
//!     type Output = f64;
//!     
//!     fn index(&self, index: usize) -> &Self::Output {
//!         &self.data[index]
//!     }
//! }
//!
//! impl IndexMut<usize> for CustomContainer {
//!     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//!         &mut self.data[index]
//!     }
//! }
//!
//! impl ContainerMut for CustomContainer {
//!     fn values_mut(&mut self) -> &mut [f64] {
//!        &mut self.data
//!     }
//! }
//! ```
//!
//! ## Thread Safety and Simulation Context
//!
//! The container system is designed for use in single-threaded simulation contexts,
//! as is typical for system dynamics models:
//!
//! - **No internal mutability**: Containers are immutable during simulation
//! - **Setup-time modification**: Mutable access only during model construction
//! - **Deterministic behaviour**: Operations produce consistent results
//! - **Memory safety**: Rust's ownership system prevents data races
//!
//! ## Future Extensions
//!
//! The container system is designed to accommodate future XMILE container types:
//!
//! ```rust
//! // Hypothetical future container types
//! pub struct Array<const N: usize> {
//!     data: [f64; N],
//! }
//!
//! pub struct Conveyor {
//!     data: Vec<f64>,
//!     length: f64,  // May change during simulation
//! }
//!
//! pub struct Queue {
//!     data: std::collections::VecDeque<f64>,  // Changes during simulation
//! }
//!
//! // All would implement the same Container trait for uniform access
//! ```
//!
//! ## Integration with Model Validation
//!
//! The container system supports comprehensive model validation:
//!
//! ```rust
//! use xmile::Container;
//!
//! fn validate_container<T: Container>(container: &T, name: &str) -> Vec<String> {
//!     let mut errors = Vec::new();
//!     
//!     // Check for empty containers
//!     if container.is_empty() {
//!         errors.push(format!("Container '{}' is empty", name));
//!     }
//!     
//!     // Check for invalid values
//!     for (i, &value) in container.values().iter().enumerate() {
//!         if !value.is_finite() {
//!             errors.push(format!("Container '{}' has invalid value at index {}: {}",
//!                                name, i, value));
//!         }
//!     }
//!     
//!     // Domain-specific validation
//!     if let Some(range) = container.range() {
//!         if range.0 < 0.0 {
//!             errors.push(format!("Container '{}' has negative values", name));
//!         }
//!     }
//!     
//!     errors
//! }
//! ```
//!
//! ## Summary
//!
//! The XMILE container system provides:
//!
//! 1. **Unified Interface**: Consistent access patterns across all container types
//! 2. **Statistical Operations**: Built-in functions for common mathematical operations
//! 3. **Index-based Access**: Square bracket notation as specified in XMILE section 3.7.1
//! 4. **Type Safety**: Compile-time guarantees through Rust's trait system
//! 5. **Performance**: Zero-cost abstractions and efficient memory layout
//! 6. **Extensibility**: Easy addition of new container types whilst maintaining compatibility
//!
//! This foundation enables robust, efficient, and XMILE-compliant implementations of
//! system dynamics models with complex data structures and mathematical operations.

use std::ops::{Index, IndexMut};

/// Core trait for all XMILE containers providing uniform access and operations.
///
/// This trait defines the fundamental interface that all XMILE container types
/// must implement according to specification section 3.2.4. It ensures consistent
/// behaviour across graphical functions, arrays, conveyors, and queues.
///
/// ## Required Implementations
///
/// All containers must provide:
/// - Direct access to underlying numeric data
/// - Size information and emptiness checking  
/// - Statistical operations (mean, min, max, range)
/// - Index-based element access (via `Index` trait)
///
/// ## XMILE Compliance
///
/// This trait directly implements the container requirements from section 3.2.4:
/// - "All containers in XMILE are lists of numbers"
/// - "Syntax and operation of these containers are consistent"
/// - "Uniformly accessed with square bracket notation"
/// - "Built-in functions that apply to all of them"
pub trait Container: Index<usize, Output = f64> {
    /// Returns the underlying numeric data as a slice.
    ///
    /// This provides direct access to the container's values for iteration,
    /// functional operations, and integration with standard library functions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// let values = container.values();
    /// let sum: f64 = values.iter().sum();
    /// ```
    fn values(&self) -> &[f64];

    /// Returns the number of elements in the container.
    ///
    /// For graphical functions and arrays, this size is fixed during simulation.
    /// For conveyors and queues, this may change dynamically.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// println!("Container has {} elements", container.len());
    /// ```
    fn len(&self) -> usize {
        self.values().len()
    }

    /// Checks if the container has no elements.
    ///
    /// This is equivalent to `self.len() == 0` but may be more semantically clear.
    /// Provided as a default implementation for convenience.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// if container.is_empty() {
    ///     println!("No data available");
    /// }
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Calculates the arithmetic mean of all values in the container.
    ///
    /// Returns `None` if the container is empty, `Some(mean)` otherwise.
    /// This is one of the built-in statistical functions mentioned in section 3.2.4.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// match container.mean() {
    ///     Some(mean) => println!("Average: {}", mean),
    ///     None => println!("Cannot calculate mean of empty container"),
    /// }
    /// ```
    fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            None
        } else {
            let sum: f64 = self.values().iter().sum();
            Some(sum / self.len() as f64)
        }
    }

    /// Finds the minimum value in the container.
    ///
    /// Returns `None` if the container is empty, `Some(min)` otherwise.
    /// Uses partial comparison to handle potential NaN values correctly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// if let Some(min) = container.min() {
    ///     println!("Minimum value: {}", min);
    /// }
    /// ```
    fn min(&self) -> Option<f64> {
        self.values().iter().fold(None, |acc, x| match acc {
            Some(min) => Some(min.min(*x)),
            None => Some(*x),
        })
    }

    /// Finds the maximum value in the container.
    ///
    /// Returns `None` if the container is empty, `Some(max)` otherwise.
    /// Uses partial comparison to handle potential NaN values correctly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// if let Some(max) = container.max() {
    ///     println!("Maximum value: {}", max);
    /// }
    /// ```
    fn max(&self) -> Option<f64> {
        self.values().iter().fold(None, |acc, x| match acc {
            Some(max) => Some(max.max(*x)),
            None => Some(*x),
        })
    }

    /// Returns the range (minimum, maximum) of values in the container.
    ///
    /// This is a convenience method that efficiently combines min and max operations.
    /// Returns `None` if the container is empty, `Some((min, max))` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Container;
    ///
    /// let container = vec![0.0, 1.0, 2.0, 3.0];
    /// match container.range() {
    ///     Some((min, max)) => {
    ///         println!("Range: {} to {} (span: {})", min, max, max - min);
    ///     },
    ///     None => println!("Empty container has no range"),
    /// }
    /// ```
    fn range(&self) -> Option<(f64, f64)> {
        match (self.min(), self.max()) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
    }
}

/// Trait for containers that support mutable access to their elements.
///
/// This trait extends the base `Container` trait for containers that allow
/// modification of their values during model setup (before simulation begins).
///
/// ## XMILE Size Constraints
///
/// According to section 3.2.4:
/// - **Graphical functions and arrays**: Size SHALL NOT change during simulation
/// - **Conveyors**: Length MAY change during simulation  
/// - **Queues**: Size changes as a matter of course during simulation
///
/// This trait is used for setup-time modifications and for containers that
/// support dynamic sizing during simulation.
///
/// ## Required Traits
///
/// Implementors must also implement:
/// - `Container`: For all read-only operations
/// - `IndexMut<usize>`: For mutable element access via square brackets
///
/// # Examples
///
/// ```rust
/// use xmile::{Container, ContainerMut};
///
/// let mut container = vec![0.0, 0.0, 0.0];
///
/// // Modify elements during setup
/// container[0] = 1.0;
/// let index = container.len() - 1;
/// container[index] = 10.0;
///
/// // All Container operations still available
/// let mean = container.mean();
/// ```
pub trait ContainerMut: Container + IndexMut<usize, Output = f64> {
    /// Returns the underlying numeric data as a mutable slice.
    ///
    /// This provides direct access to the container's values for iteration,
    /// functional operations, and integration with standard library functions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::ContainerMut;
    ///
    /// let mut container = vec![0.0, 0.0, 0.0];
    /// let mut values = container.values_mut();
    /// values[0] = 1.0;
    /// ```
    fn values_mut(&mut self) -> &mut [f64];
}

impl Container for Vec<f64> {
    fn values(&self) -> &[f64] {
        self.as_slice()
    }
}

impl ContainerMut for Vec<f64> {
    fn values_mut(&mut self) -> &mut [f64] {
        self.as_mut_slice()
    }
}
