use serde::{Deserialize, Serialize};

pub mod schema;

#[cfg(test)]
mod tests {
    use serde_xml_rs::{from_str, to_string};

    use super::*;

    #[test]
    fn test_xml_serialization() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?><Item><name>Banana</name><source>Store</source></Item>"#;
        let should_be = Mo {
            name: "Banana".to_string(),
            source: "Store".to_string(),
        };

        let item: Item = from_str(src).unwrap();
        assert_eq!(item, should_be);

        let reserialized_item = to_string(&item).unwrap();
        assert_eq!(src, reserialized_item);
    }
}
