//! Serialization helpers for quick-xml.
//!
//! Provides:
//! - `AttrList`: attribute builder that owns formatted values
//! - `XmlEmitter`: convenient element/text writing helpers

use std::io::Write;

use quick_xml::Writer;
use quick_xml::events::{BytesCData, BytesDecl, BytesStart, BytesText, Event};

use crate::xml::serialize::SerializeError;

/// A list of attributes with owned values.
///
/// This avoids the lifetime issues with quick-xml's `ElementWriter::with_attribute`
/// by owning all formatted string values until the element is written.
#[derive(Debug, Default)]
pub struct AttrList {
    /// Stored as (key, value) pairs where both are owned strings.
    attrs: Vec<(String, String)>,
}

impl AttrList {
    /// Create a new empty attribute list.
    pub fn new() -> Self {
        Self { attrs: Vec::new() }
    }

    /// Add a required string attribute.
    pub fn add(&mut self, key: &str, value: impl AsRef<str>) -> &mut Self {
        self.attrs
            .push((key.to_string(), value.as_ref().to_string()));
        self
    }

    /// Add an optional string attribute (only added if Some).
    pub fn add_opt(&mut self, key: &str, value: Option<impl AsRef<str>>) -> &mut Self {
        if let Some(v) = value {
            self.add(key, v);
        }
        self
    }

    /// Add a numeric attribute (f64).
    pub fn add_f64(&mut self, key: &str, value: f64) -> &mut Self {
        // Format without unnecessary decimal places
        let text = if value.fract() == 0.0 {
            format!("{}", value as i64)
        } else {
            format!("{}", value)
        };
        self.attrs.push((key.to_string(), text));
        self
    }

    /// Add an optional f64 attribute.
    pub fn add_opt_f64(&mut self, key: &str, value: Option<f64>) -> &mut Self {
        if let Some(v) = value {
            self.add_f64(key, v);
        }
        self
    }

    /// Add an i32 attribute.
    pub fn add_i32(&mut self, key: &str, value: i32) -> &mut Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    /// Add an optional i32 attribute.
    pub fn add_opt_i32(&mut self, key: &str, value: Option<i32>) -> &mut Self {
        if let Some(v) = value {
            self.add_i32(key, v);
        }
        self
    }

    /// Add a u32 attribute.
    pub fn add_u32(&mut self, key: &str, value: u32) -> &mut Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    /// Add an optional u32 attribute.
    pub fn add_opt_u32(&mut self, key: &str, value: Option<u32>) -> &mut Self {
        if let Some(v) = value {
            self.add_u32(key, v);
        }
        self
    }

    /// Add a bool attribute.
    pub fn add_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    /// Add a bool attribute only if it differs from the default.
    ///
    /// This is useful for attributes where the default is true and you only
    /// want to serialize when the value is false (or vice versa).
    pub fn add_bool_if_not_default(&mut self, key: &str, value: bool, default: bool) -> &mut Self {
        if value != default {
            self.add_bool(key, value);
        }
        self
    }

    /// Add an optional bool attribute.
    pub fn add_opt_bool(&mut self, key: &str, value: Option<bool>) -> &mut Self {
        if let Some(v) = value {
            self.add_bool(key, v);
        }
        self
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }

    /// Get the number of attributes.
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    /// Apply all attributes to a BytesStart element.
    pub fn apply_to(&self, start: &mut BytesStart<'_>) {
        for (key, value) in &self.attrs {
            start.push_attribute((key.as_str(), value.as_str()));
        }
    }
}

/// A helper for writing XML elements with less boilerplate.
///
/// This wraps a quick-xml Writer and provides convenient methods for
/// common patterns like writing text elements, empty elements, and
/// elements with attributes.
pub struct XmlEmitter<'a, W: Write> {
    writer: &'a mut Writer<W>,
}

impl<'a, W: Write> XmlEmitter<'a, W> {
    /// Create a new emitter wrapping a writer.
    pub fn new(writer: &'a mut Writer<W>) -> Self {
        Self { writer }
    }

    /// Get the underlying writer.
    pub fn writer(&mut self) -> &mut Writer<W> {
        self.writer
    }

    /// Write an XML declaration.
    pub fn xml_decl(&mut self) -> Result<(), SerializeError> {
        let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
        self.writer.write_event(Event::Decl(decl))?;
        Ok(())
    }

    /// Write a text element: `<name>text</name>`.
    pub fn text_elem(&mut self, name: &str, text: &str) -> Result<(), SerializeError> {
        self.writer
            .create_element(name)
            .write_text_content(BytesText::new(text))?;
        Ok(())
    }

    /// Write an optional text element (only if value is Some).
    pub fn opt_text_elem(&mut self, name: &str, text: Option<&str>) -> Result<(), SerializeError> {
        if let Some(t) = text {
            self.text_elem(name, t)?;
        }
        Ok(())
    }

    /// Write a numeric element: `<name>123</name>`.
    pub fn num_elem(&mut self, name: &str, value: f64) -> Result<(), SerializeError> {
        let text = if value.fract() == 0.0 {
            format!("{}", value as i64)
        } else {
            format!("{}", value)
        };
        self.text_elem(name, &text)
    }

    /// Write an optional numeric element.
    pub fn opt_num_elem(&mut self, name: &str, value: Option<f64>) -> Result<(), SerializeError> {
        if let Some(v) = value {
            self.num_elem(name, v)?;
        }
        Ok(())
    }

    /// Write an empty element: `<name/>`.
    pub fn empty_elem(&mut self, name: &str) -> Result<(), SerializeError> {
        self.writer.create_element(name).write_empty()?;
        Ok(())
    }

    /// Write an optional empty element (only if Some(true)).
    pub fn opt_empty_elem(
        &mut self,
        name: &str,
        present: Option<bool>,
    ) -> Result<(), SerializeError> {
        if present == Some(true) {
            self.empty_elem(name)?;
        }
        Ok(())
    }

    /// Write a CDATA element: `<name><![CDATA[content]]></name>`.
    pub fn cdata_elem(&mut self, name: &str, content: &str) -> Result<(), SerializeError> {
        self.writer.create_element(name).write_inner_content(
            |w| -> Result<(), SerializeError> {
                w.write_event(Event::CData(BytesCData::new(content)))?;
                Ok(())
            },
        )?;
        Ok(())
    }

    /// Write an optional CDATA element.
    pub fn opt_cdata_elem(
        &mut self,
        name: &str,
        content: Option<&str>,
    ) -> Result<(), SerializeError> {
        if let Some(c) = content {
            self.cdata_elem(name, c)?;
        }
        Ok(())
    }

    /// Write an empty element with attributes: `<name attr="value"/>`.
    pub fn empty_elem_with_attrs(
        &mut self,
        name: &str,
        attrs: &AttrList,
    ) -> Result<(), SerializeError> {
        let mut start = BytesStart::new(name);
        attrs.apply_to(&mut start);
        self.writer.write_event(Event::Empty(start))?;
        Ok(())
    }

    /// Write a text element with attributes: `<name attr="value">text</name>`.
    pub fn text_elem_with_attrs(
        &mut self,
        name: &str,
        attrs: &AttrList,
        text: &str,
    ) -> Result<(), SerializeError> {
        let mut start = BytesStart::new(name);
        attrs.apply_to(&mut start);
        self.writer.write_event(Event::Start(start.to_owned()))?;
        self.writer.write_event(Event::Text(BytesText::new(text)))?;
        self.writer
            .write_event(Event::End(start.to_end().into_owned()))?;
        Ok(())
    }

    /// Start an element with attributes and return the BytesStart for inner content.
    ///
    /// Use this when you need to write complex inner content using write_inner_content.
    pub fn start_elem(&mut self, name: &str, attrs: &AttrList) -> BytesStart<'static> {
        let mut start = BytesStart::new(name);
        attrs.apply_to(&mut start);
        start.into_owned()
    }

    /// Write a start element event.
    pub fn write_start(&mut self, start: &BytesStart<'_>) -> Result<(), SerializeError> {
        self.writer.write_event(Event::Start(start.clone()))?;
        Ok(())
    }

    /// Write an end element event.
    pub fn write_end(&mut self, name: &str) -> Result<(), SerializeError> {
        self.writer
            .write_event(Event::End(quick_xml::events::BytesEnd::new(name)))?;
        Ok(())
    }

    /// Write a non_negative element with the special semantics:
    /// - None -> don't write anything
    /// - Some(None) -> write empty tag `<non_negative/>`
    /// - Some(Some(false)) -> write `<non_negative>false</non_negative>`
    /// - Some(Some(true)) -> write `<non_negative>true</non_negative>`
    pub fn non_negative_elem(
        &mut self,
        non_negative: Option<Option<bool>>,
    ) -> Result<(), SerializeError> {
        if let Some(inner) = non_negative {
            match inner {
                None => self.empty_elem("non_negative")?,
                Some(v) => self.text_elem("non_negative", if v { "true" } else { "false" })?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::Writer;

    #[test]
    fn test_attr_list_basic() {
        let mut attrs = AttrList::new();
        attrs
            .add("name", "test")
            .add_f64("x", 100.0)
            .add_i32("z", -5)
            .add_bool("visible", true);

        assert_eq!(attrs.len(), 4);

        let mut start = BytesStart::new("element");
        attrs.apply_to(&mut start);
        // The attributes are applied; we can verify by checking the start
    }

    #[test]
    fn test_attr_list_optional() {
        let mut attrs = AttrList::new();
        let missing: Option<&str> = None;
        attrs
            .add("required", "yes")
            .add_opt("present", Some("value"))
            .add_opt("missing", missing)
            .add_opt_f64("has_num", Some(42.5))
            .add_opt_f64("no_num", None);

        assert_eq!(attrs.len(), 3); // required, present, has_num
    }

    #[test]
    fn test_attr_list_bool_default() {
        let mut attrs = AttrList::new();
        attrs
            .add_bool_if_not_default("show", true, true) // same as default, skip
            .add_bool_if_not_default("hide", false, true); // different, include

        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn test_emitter_text_elem() {
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        let mut emit = XmlEmitter::new(&mut writer);

        emit.text_elem("greeting", "hello").unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "<greeting>hello</greeting>");
    }

    #[test]
    fn test_emitter_num_elem() {
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        let mut emit = XmlEmitter::new(&mut writer);

        emit.num_elem("int", 42.0).unwrap();
        emit.num_elem("float", 3.14).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "<int>42</int><float>3.14</float>");
    }

    #[test]
    fn test_emitter_empty_with_attrs() {
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        let mut emit = XmlEmitter::new(&mut writer);

        let mut attrs = AttrList::new();
        attrs.add("id", "123").add_f64("x", 10.0);

        emit.empty_elem_with_attrs("point", &attrs).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("id=\"123\""));
        assert!(result.contains("x=\"10\""));
        assert!(result.ends_with("/>"));
    }
}
