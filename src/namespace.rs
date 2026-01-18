//! # XMILE Namespace Support
//!
//! This module implements XMILE namespace handling according to specification section 3.2.2.3.
//!
//! ## Overview
//!
//! Namespaces in XMILE provide a mechanism to avoid conflicts between identifiers
//! in different libraries of functions. Each library, whether vendor-specific or
//! user-defined, exists within its own namespace.
//!
//! ## Predefined Namespaces
//!
//! XMILE defines several standard namespaces:
//!
//! - **`std`**: All XMILE statement and function identifiers
//! - **`user`**: User-defined function and macro names
//! - **Vendor namespaces**: Each major System Dynamics tool has its own namespace
//!
//! ## Namespace Syntax
//!
//! Namespaces use dot notation for qualification:
//! - `std.function` - Function in the standard namespace
//! - `user.custom.helper` - Nested user namespace
//! - `vensim.lookup` - Vendor-specific function
//!
//! ## Examples
//!
//! ```rust
//! use xmile::Namespace;
//!
//! // Parse a single namespace
//! let ns = Namespace::from_part("std");
//! assert_eq!(ns, Namespace::Std);
//!
//! // Parse a namespace path
//! let path = Namespace::from_parts_str("user.custom.utils");
//! assert_eq!(path.len(), 3);
//! assert_eq!(path[0], Namespace::User);
//!
//! // Check namespace properties
//! assert!(Namespace::Std.is_predefined());
//! assert!(Namespace::Vensim.is_vendor());
//! assert!(!Namespace::Other("custom".to_string()).is_predefined());
//!
//! // Create namespace prefix
//! let prefix = Namespace::as_prefix(&path);
//! assert_eq!(prefix, "user.custom.utils");
//! ```

use std::hash::Hash;
use std::{fmt, ops};

use serde::{Deserialize, Serialize};

/// XMILE namespace enumeration supporting both predefined and custom namespaces.
///
/// This enum represents all predefined XMILE namespaces as well as custom
/// namespaces that may be defined by users or other vendors. The predefined
/// namespaces follow the XMILE specification exactly.
///
/// ## Predefined Namespaces
///
/// The XMILE specification defines these standard namespaces:
///
/// | Namespace | Purpose |
/// |-----------|---------|
/// | `std` | All XMILE statement and function identifiers |
/// | `user` | User-defined function and macro names |
/// | `anylogic` | All Anylogic identifiers |
/// | `forio` | All Forio Simulations identifiers |
/// | `insightmaker` | All Insight Maker identifiers |
/// | `isee` | All isee systems identifiers |
/// | `powersim` | All Powersim Software identifiers |
/// | `simanticssd` | All Simantics System Dynamics Tool identifiers |
/// | `simile` | All Simulistics identifiers |
/// | `sysdea` | All Strategy Dynamics identifiers |
/// | `vensim` | All Ventana Systems identifiers |
///
/// ## Custom Namespaces
///
/// The `Other(String)` variant allows for custom namespaces not covered
/// by the predefined list. These are parsed from any namespace identifier
/// that doesn't match a predefined name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Namespace {
    /// All XMILE statement and function identifiers
    ///
    /// This is the default namespace for all standard XMILE constructs.
    /// Most XMILE files specify that they use the std namespace to avoid
    /// requiring `std.` prefixes on standard functions.
    Std,

    /// User-defined function and macro names
    ///
    /// It is recommended that user-defined functions and macros be included
    /// in a child namespace of the global user namespace (e.g., `user.mylib.function`).
    User,

    /// All Anylogic identifiers
    ///
    /// Reserved for identifiers specific to the Anylogic System Dynamics tool.
    Anylogic,

    /// All Forio Simulations identifiers
    ///
    /// Reserved for identifiers specific to Forio Simulations platform.
    Forio,

    /// All Insight Maker identifiers
    ///
    /// Reserved for identifiers specific to the Insight Maker tool.
    Insightmaker,

    /// All isee systems identifiers
    ///
    /// Reserved for identifiers specific to isee systems tools (e.g., STELLA, iThink).
    Isee,

    /// All Powersim Software identifiers
    ///
    /// Reserved for identifiers specific to Powersim tools.
    Powersim,

    /// All Simantics System Dynamics Tool identifiers
    ///
    /// Reserved for identifiers specific to the Simantics System Dynamics Tool.
    Simanticssd,

    /// All Simulistics identifiers
    ///
    /// Reserved for identifiers specific to Simile and other Simulistics tools.
    Simile,

    /// All Strategy Dynamics identifiers
    ///
    /// Reserved for identifiers specific to Strategy Dynamics tools.
    Sysdea,

    /// All Ventana Systems identifiers
    ///
    /// Reserved for identifiers specific to Vensim and other Ventana Systems tools.
    Vensim,

    /// Custom namespace not in the predefined list
    ///
    /// This variant represents any namespace that doesn't match the predefined
    /// XMILE namespaces. The string contains the original namespace identifier
    /// as provided by the user.
    Other(String),
}

impl Namespace {
    /// Parses a namespace path from a dot-separated string.
    ///
    /// Takes a string like `"user.custom.utils"` and returns a vector of
    /// namespace components. Each component is parsed independently using
    /// `from_part()`.
    ///
    /// # Arguments
    ///
    /// * `s` - A dot-separated namespace path string
    ///
    /// # Returns
    ///
    /// A vector of `Namespace` values representing the path components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// let path = Namespace::from_parts_str("user.custom.utils");
    /// assert_eq!(path.len(), 3);
    /// assert_eq!(path[0], Namespace::User);
    /// assert_eq!(path[1], Namespace::Other("custom".to_string()));
    /// assert_eq!(path[2], Namespace::Other("utils".to_string()));
    /// ```
    pub fn from_parts_str(s: &str) -> Vec<Self> {
        // Split the string by dot and parse each part
        s.split('.').map(Namespace::from_part).collect()
    }

    /// Parses a single namespace component from a string.
    ///
    /// Converts a string to the appropriate `Namespace` variant, using
    /// case-insensitive matching for predefined namespaces.
    ///
    /// # Arguments
    ///
    /// * `s` - A single namespace identifier
    ///
    /// # Returns
    ///
    /// The corresponding `Namespace` variant. If the string doesn't match
    /// any predefined namespace, returns `Namespace::Other(s.to_string())`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// assert_eq!(Namespace::from_part("std"), Namespace::Std);
    /// assert_eq!(Namespace::from_part("STD"), Namespace::Std); // Case-insensitive
    /// assert_eq!(Namespace::from_part("custom"), Namespace::Other("custom".to_string()));
    /// ```
    pub fn from_part(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "std" => Namespace::Std,
            "user" => Namespace::User,
            "anylogic" => Namespace::Anylogic,
            "forio" => Namespace::Forio,
            "insightmaker" => Namespace::Insightmaker,
            "isee" => Namespace::Isee,
            "powersim" => Namespace::Powersim,
            "simanticssd" => Namespace::Simanticssd,
            "simile" => Namespace::Simile,
            "sysdea" => Namespace::Sysdea,
            "vensim" => Namespace::Vensim,
            _ => Namespace::Other(s.to_string()),
        }
    }

    /// Returns the string representation of the namespace.
    ///
    /// This provides the canonical lowercase string form for each namespace,
    /// which matches the XMILE specification naming conventions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// assert_eq!(Namespace::Std.as_str(), "std");
    /// assert_eq!(Namespace::Vensim.as_str(), "vensim");
    /// assert_eq!(Namespace::Other("custom".to_string()).as_str(), "custom");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Namespace::Std => "std",
            Namespace::User => "user",
            Namespace::Anylogic => "anylogic",
            Namespace::Forio => "forio",
            Namespace::Insightmaker => "insightmaker",
            Namespace::Isee => "isee",
            Namespace::Powersim => "powersim",
            Namespace::Simanticssd => "simanticssd",
            Namespace::Simile => "simile",
            Namespace::Sysdea => "sysdea",
            Namespace::Vensim => "vensim",
            Namespace::Other(s) => s,
        }
    }

    /// Checks if this is a predefined XMILE namespace.
    ///
    /// Returns `true` for all namespaces defined in the XMILE specification,
    /// `false` for custom namespaces (`Other` variant).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// assert!(Namespace::Std.is_predefined());
    /// assert!(Namespace::Vensim.is_predefined());
    /// assert!(!Namespace::Other("custom".to_string()).is_predefined());
    /// ```
    pub fn is_predefined(&self) -> bool {
        !matches!(self, Namespace::Other(_))
    }

    /// Checks if this is a vendor-specific namespace.
    ///
    /// Returns `true` for namespaces reserved for specific System Dynamics
    /// tools and vendors. Does not include `std` or `user` namespaces.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// assert!(!Namespace::Std.is_vendor());
    /// assert!(!Namespace::User.is_vendor());
    /// assert!(Namespace::Vensim.is_vendor());
    /// assert!(Namespace::Isee.is_vendor());
    /// assert!(!Namespace::Other("custom".to_string()).is_vendor());
    /// ```
    pub fn is_vendor(&self) -> bool {
        matches!(
            self,
            Namespace::Anylogic
                | Namespace::Forio
                | Namespace::Insightmaker
                | Namespace::Isee
                | Namespace::Powersim
                | Namespace::Simanticssd
                | Namespace::Simile
                | Namespace::Sysdea
                | Namespace::Vensim
        )
    }

    /// Returns a list of all reserved namespaces.
    ///
    /// This includes all predefined namespaces as well as vendor-specific namespaces.
    ///
    /// # Returns
    ///
    /// A vector containing all reserved `Namespace` values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// let reserved = Namespace::reserved_namespaces();
    /// assert!(reserved.contains(&Namespace::Std));
    /// assert!(reserved.contains(&Namespace::Vensim));
    /// assert!(!reserved.contains(&Namespace::Other("custom".to_string())));
    /// ```
    pub fn reserved_namespaces() -> Vec<Namespace> {
        vec![
            Namespace::Std,
            Namespace::User,
            Namespace::Anylogic,
            Namespace::Forio,
            Namespace::Insightmaker,
            Namespace::Isee,
            Namespace::Powersim,
            Namespace::Simanticssd,
            Namespace::Simile,
            Namespace::Sysdea,
            Namespace::Vensim,
        ]
    }

    /// Converts a namespace path to a dot-separated string prefix.
    ///
    /// Takes a slice of namespace components and joins them with dots
    /// to create a namespace prefix suitable for qualified identifiers.
    ///
    /// # Arguments
    ///
    /// * `path` - A slice of namespace components
    ///
    /// # Returns
    ///
    /// A dot-separated string representing the namespace path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// let path = vec![Namespace::User, Namespace::Other("custom".to_string())];
    /// let prefix = Namespace::as_prefix(&path);
    /// assert_eq!(prefix, "user.custom");
    /// ```
    pub fn as_prefix(path: &[Self]) -> String {
        path.iter()
            .map(Namespace::as_str)
            .collect::<Vec<&str>>()
            .join(".")
    }
}

impl fmt::Display for Namespace {
    /// Formats the namespace for display using its string representation.
    ///
    /// This allows namespaces to be easily printed and logged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// let ns = Namespace::Std;
    /// println!("Namespace: {}", ns); // Prints "Namespace: std"
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ops::Deref for Namespace {
    type Target = str;

    /// Allows namespace to be dereferenced to its string representation.
    ///
    /// This enables convenient string operations on namespace values
    /// without explicit conversion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmile::Namespace;
    ///
    /// let ns = Namespace::Std;
    /// assert_eq!(ns.len(), 3); // Calls str::len() on "std"
    /// assert!(ns.starts_with("st")); // Calls str::starts_with()
    /// ```
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
