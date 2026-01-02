//! Deserialization helpers for quick-xml.
//!
//! Provides:
//! - `Attrs`: typed attribute map with convenient getters
//! - `XmlCursor`: wrapper with peek/lookahead and path tracking
//! - `skip_element`: robust subtree skipping for unknown elements

use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::xml::deserialize::DeserializeError;

/// A parsed attribute map that owns decoded string values.
///
/// Use this instead of repeatedly iterating over `BytesStart::attributes()`
/// and calling `decode_and_unescape_value` for each attribute.
#[derive(Debug, Clone, Default)]
pub struct Attrs {
    map: HashMap<String, String>,
    /// The element name for error messages
    element_name: String,
}

impl Attrs {
    /// Create an empty Attrs (for empty elements or when no attributes expected).
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
            element_name: String::new(),
        }
    }

    /// Parse attributes from a `BytesStart` event.
    ///
    /// This decodes and unescapes all attribute values once, storing them in a HashMap.
    pub fn from_start<R: BufRead>(
        start: &BytesStart<'_>,
        reader: &Reader<R>,
    ) -> Result<Self, DeserializeError> {
        let element_name = String::from_utf8_lossy(start.name().as_ref()).to_string();
        let mut map = HashMap::new();

        for attr_result in start.attributes() {
            let attr = attr_result?;
            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
            let value = attr.decode_and_unescape_value(reader)?.to_string();
            map.insert(key, value);
        }

        Ok(Self { map, element_name })
    }

    /// Get the element name this Attrs was parsed from.
    pub fn element_name(&self) -> &str {
        &self.element_name
    }

    /// Get an optional string attribute.
    pub fn get_opt(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    /// Get a required string attribute, returning an error if missing.
    pub fn get_req(&self, key: &str) -> Result<&str, DeserializeError> {
        self.map
            .get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| DeserializeError::MissingField(format!("{}@{}", self.element_name, key)))
    }

    /// Get an optional string attribute as an owned String.
    pub fn get_opt_string(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    /// Get a required string attribute as an owned String.
    pub fn get_req_string(&self, key: &str) -> Result<String, DeserializeError> {
        self.get_req(key).map(|s| s.to_string())
    }

    /// Get an optional attribute parsed as a type implementing FromStr.
    pub fn get_opt_parsed<T: FromStr>(&self, key: &str) -> Result<Option<T>, DeserializeError>
    where
        T::Err: std::fmt::Display,
    {
        match self.map.get(key) {
            Some(s) => s.parse::<T>().map(Some).map_err(|e| {
                DeserializeError::Custom(format!(
                    "Invalid value for {}@{}: {} (got '{}')",
                    self.element_name, key, e, s
                ))
            }),
            None => Ok(None),
        }
    }

    /// Get a required attribute parsed as a type implementing FromStr.
    pub fn get_req_parsed<T: FromStr>(&self, key: &str) -> Result<T, DeserializeError>
    where
        T::Err: std::fmt::Display,
    {
        let s = self.get_req(key)?;
        s.parse::<T>().map_err(|e| {
            DeserializeError::Custom(format!(
                "Invalid value for {}@{}: {} (got '{}')",
                self.element_name, key, e, s
            ))
        })
    }

    /// Get an optional f64 attribute.
    pub fn get_opt_f64(&self, key: &str) -> Result<Option<f64>, DeserializeError> {
        self.get_opt_parsed(key)
    }

    /// Get a required f64 attribute.
    pub fn get_req_f64(&self, key: &str) -> Result<f64, DeserializeError> {
        self.get_req_parsed(key)
    }

    /// Get an optional i32 attribute.
    pub fn get_opt_i32(&self, key: &str) -> Result<Option<i32>, DeserializeError> {
        self.get_opt_parsed(key)
    }

    /// Get a required i32 attribute.
    pub fn get_req_i32(&self, key: &str) -> Result<i32, DeserializeError> {
        self.get_req_parsed(key)
    }

    /// Get an optional u32 attribute.
    pub fn get_opt_u32(&self, key: &str) -> Result<Option<u32>, DeserializeError> {
        self.get_opt_parsed(key)
    }

    /// Get a required u32 attribute.
    pub fn get_req_u32(&self, key: &str) -> Result<u32, DeserializeError> {
        self.get_req_parsed(key)
    }

    /// Get an optional bool attribute.
    ///
    /// Recognizes "true"/"false" (case-insensitive).
    pub fn get_opt_bool(&self, key: &str) -> Result<Option<bool>, DeserializeError> {
        match self.map.get(key) {
            Some(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "1" | "yes" => Ok(Some(true)),
                    "false" | "0" | "no" => Ok(Some(false)),
                    _ => Err(DeserializeError::Custom(format!(
                        "Invalid boolean for {}@{}: '{}' (expected true/false)",
                        self.element_name, key, s
                    ))),
                }
            }
            None => Ok(None),
        }
    }

    /// Get an optional bool attribute with a default value.
    pub fn get_bool_or(&self, key: &str, default: bool) -> Result<bool, DeserializeError> {
        Ok(self.get_opt_bool(key)?.unwrap_or(default))
    }

    /// Check if an attribute is present (regardless of value).
    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Get all keys (for debugging/logging).
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(|s| s.as_str())
    }
}

/// Skip an entire element subtree, consuming all nested content until the matching end tag.
///
/// Call this when you've just consumed a `Start` event and want to skip everything
/// inside it (including nested elements) until the corresponding `End` event.
///
/// For `Empty` elements, this is a no-op since there's no content to skip.
///
/// # Arguments
/// * `reader` - The XML reader
/// * `buf` - The read buffer
/// * `tag_name` - The name of the element being skipped (for matching the end tag)
pub fn skip_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
) -> Result<(), DeserializeError> {
    let mut depth = 1u32;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                // Nested element, increase depth
                if e.name().as_ref() == tag_name {
                    depth += 1;
                } else {
                    // Any start tag increases nesting
                    depth += 1;
                }
            }
            Event::End(e) => {
                depth -= 1;
                if depth == 0 && e.name().as_ref() == tag_name {
                    break;
                }
            }
            Event::Eof => {
                return Err(DeserializeError::UnexpectedEof);
            }
            // Text, CData, Comment, PI, Empty, Decl - just consume and continue
            _ => {}
        }
    }

    Ok(())
}

/// Represents a parsed child element event with its name and whether it's empty.
#[derive(Debug, Clone)]
pub struct ChildElement<'a> {
    /// The element name as bytes.
    pub name: &'a [u8],
    /// Whether this was an Empty element (`<tag/>`) vs Start element (`<tag>`).
    pub is_empty: bool,
}

/// A wrapper around quick-xml's Reader that provides:
/// - Path tracking for better error messages
/// - Convenient methods for common patterns
///
/// Unlike a full lookahead cursor, this is a lightweight helper that
/// works with the existing event-driven parsing style.
pub struct XmlCursor<'a, R: BufRead> {
    reader: &'a mut Reader<R>,
    buf: &'a mut Vec<u8>,
    /// Path stack for error context (e.g., ["xmile", "model", "variables", "stock"])
    path: Vec<String>,
}

impl<'a, R: BufRead> XmlCursor<'a, R> {
    /// Create a new cursor wrapping a reader and buffer.
    pub fn new(reader: &'a mut Reader<R>, buf: &'a mut Vec<u8>) -> Self {
        Self {
            reader,
            buf,
            path: Vec::new(),
        }
    }

    /// Push an element onto the path stack (for error context).
    pub fn push_path(&mut self, element: &str) {
        self.path.push(element.to_string());
    }

    /// Pop the most recent element from the path stack.
    pub fn pop_path(&mut self) {
        self.path.pop();
    }

    /// Get the current path as a string (for error messages).
    pub fn path_str(&self) -> String {
        self.path.join("/")
    }

    /// Create an error with path context.
    pub fn error(&self, message: impl Into<String>) -> DeserializeError {
        let msg = message.into();
        if self.path.is_empty() {
            DeserializeError::Custom(msg)
        } else {
            DeserializeError::Custom(format!("{} (at {})", msg, self.path_str()))
        }
    }

    /// Create a "missing field" error with path context.
    pub fn missing_field(&self, field: &str) -> DeserializeError {
        if self.path.is_empty() {
            DeserializeError::MissingField(field.to_string())
        } else {
            DeserializeError::MissingField(format!("{}/{}", self.path_str(), field))
        }
    }

    /// Read the next event from the reader.
    pub fn next_event(&mut self) -> Result<Event<'_>, DeserializeError> {
        self.buf.clear();
        let event = self.reader.read_event_into(self.buf)?;
        Ok(event.into_owned())
    }

    /// Read text content from the current element (after consuming Start event).
    ///
    /// Reads until the End event is encountered.
    pub fn read_text(&mut self) -> Result<String, DeserializeError> {
        let mut text = String::new();

        loop {
            self.buf.clear();
            match self.reader.read_event_into(self.buf)? {
                Event::Text(e) => {
                    text.push_str(&e.unescape()?);
                }
                Event::CData(e) => {
                    text.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
                Event::End(_) => break,
                Event::Eof => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
        }

        Ok(text)
    }

    /// Skip the current element's content (after consuming a Start event).
    pub fn skip_to_end(&mut self, tag_name: &[u8]) -> Result<(), DeserializeError> {
        skip_element(self.reader, self.buf, tag_name)
    }

    /// Parse attributes from a BytesStart event.
    pub fn parse_attrs(&self, start: &BytesStart<'_>) -> Result<Attrs, DeserializeError> {
        Attrs::from_start(start, self.reader)
    }

    /// Get the underlying reader (for when you need direct access).
    pub fn reader(&mut self) -> &mut Reader<R> {
        self.reader
    }

    /// Get the buffer (for when you need direct access).
    pub fn buf(&mut self) -> &mut Vec<u8> {
        self.buf
    }
}

/// Common variable attributes (name, access, autoexport).
/// Used by stock, flow, aux, and gf variables.
#[derive(Debug, Default)]
pub struct VarAttrs {
    pub name: Option<String>,
    pub access: Option<String>,
    pub autoexport: Option<bool>,
    pub gf_type: Option<String>,
    pub resource: Option<String>,
}

impl VarAttrs {
    /// Parse common variable attributes from an Attrs instance.
    pub fn from_attrs(attrs: &Attrs) -> Result<Self, DeserializeError> {
        Ok(Self {
            name: attrs.get_opt_string("name"),
            access: attrs.get_opt_string("access"),
            autoexport: attrs.get_opt_bool("autoexport")?,
            gf_type: attrs.get_opt_string("type"),
            resource: attrs.get_opt_string("resource"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::Reader;

    #[test]
    fn test_attrs_parsing() {
        let xml = r#"<element name="test" value="123" flag="true"/>"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(e) => {
                let attrs = Attrs::from_start(&e, &reader).unwrap();
                assert_eq!(attrs.get_req("name").unwrap(), "test");
                assert_eq!(attrs.get_req_i32("value").unwrap(), 123);
                assert_eq!(attrs.get_opt_bool("flag").unwrap(), Some(true));
                assert!(attrs.get_opt("missing").is_none());
            }
            _ => panic!("Expected Empty event"),
        }
    }

    #[test]
    fn test_attrs_missing_required() {
        let xml = r#"<element/>"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(e) => {
                let attrs = Attrs::from_start(&e, &reader).unwrap();
                let err = attrs.get_req("missing").unwrap_err();
                assert!(err.to_string().contains("element@missing"));
            }
            _ => panic!("Expected Empty event"),
        }
    }

    #[test]
    fn test_skip_element_simple() {
        let xml = r#"<root><child>text</child></root>"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();

        // Read <root>
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(e) if e.name().as_ref() == b"root" => {}
            e => panic!("Expected Start(root), got {:?}", e),
        }

        // Read <child>
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(e) if e.name().as_ref() == b"child" => {
                // Skip the rest of child
                skip_element(&mut reader, &mut buf, b"child").unwrap();
            }
            e => panic!("Expected Start(child), got {:?}", e),
        }

        // Should now be at </root>
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::End(e) if e.name().as_ref() == b"root" => {}
            e => panic!("Expected End(root), got {:?}", e),
        }
    }

    #[test]
    fn test_skip_element_nested() {
        let xml = r#"<outer><inner><deep>content</deep></inner></outer>"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();

        // Read <outer>
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(e) if e.name().as_ref() == b"outer" => {}
            e => panic!("Expected Start(outer), got {:?}", e),
        }

        // Read <inner>
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(e) if e.name().as_ref() == b"inner" => {
                // Skip the entire inner subtree
                skip_element(&mut reader, &mut buf, b"inner").unwrap();
            }
            e => panic!("Expected Start(inner), got {:?}", e),
        }

        // Should now be at </outer>
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::End(e) if e.name().as_ref() == b"outer" => {}
            e => panic!("Expected End(outer), got {:?}", e),
        }
    }

    #[test]
    fn test_cursor_read_text() {
        let xml = r#"<root>hello world</root>"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();

        // Read <root>
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(_) => {
                buf.clear();
                let mut cursor = XmlCursor::new(&mut reader, &mut buf);
                let text = cursor.read_text().unwrap();
                assert_eq!(text, "hello world");
            }
            _ => panic!("Expected Start event"),
        }
    }
}
