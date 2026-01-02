//! Quick-xml helper utilities for XMILE (de)serialization.
//!
//! This module provides ergonomic wrappers around quick-xml's low-level APIs
//! to reduce boilerplate and improve correctness in the main serialize/deserialize modules.

pub mod de;
pub mod ser;

pub use de::{Attrs, VarAttrs, XmlCursor, skip_element};
pub use ser::{AttrList, XmlEmitter};
