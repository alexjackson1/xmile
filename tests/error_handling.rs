use xmile::xml::schema::XmileFile;
use xmile::xml::{ErrorContext, XmileError};

#[test]
fn test_enhanced_error_with_context() {
    let xml = r#"
    <xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
        <header>
            <vendor>Test</vendor>
            <product version="1.0">Test Product</product>
        </header>
        <model>
            <variables>
                <stock name="TestStock">
                    <eqn>100</eqn>
                </stock>
                <aux name="TestStock">
                    <eqn>50</eqn>
                </aux>
            </variables>
        </model>
    </xmile>
    "#;

    let result = XmileFile::from_str_with_context(xml);
    assert!(result.is_ok());

    let file = result.unwrap();
    let validation_result = file.validate();

    // Should have validation errors for duplicate variable names
    assert!(validation_result.is_err());

    if let Err(XmileError::Validation(validation_error)) = validation_result {
        assert!(!validation_error.errors.is_empty());
        assert!(
            validation_error.message.contains("TestStock")
                || validation_error
                    .errors
                    .iter()
                    .any(|e| e.contains("TestStock"))
        );
    } else {
        panic!("Expected Validation error");
    }
}

#[test]
fn test_error_context_display() {
    let context = ErrorContext::with_file_and_line("test.xmile", 42).with_parsing("stock variable");

    let error_msg = format!("{}", context);
    assert!(error_msg.contains("test.xmile"));
    assert!(error_msg.contains("line 42"));
    assert!(error_msg.contains("stock variable"));
}

#[test]
fn test_error_collection() {
    use xmile::xml::ErrorCollection;

    let mut collection = ErrorCollection::new();
    assert!(collection.is_empty());

    collection.push(XmileError::Validation(Box::new(
        xmile::xml::errors::ValidationError {
            message: "Error 1".to_string(),
            context: ErrorContext::new(),
            warnings: Vec::new(),
            errors: vec!["Error 1".to_string()],
        },
    )));

    collection.push(XmileError::Validation(Box::new(
        xmile::xml::errors::ValidationError {
            message: "Error 2".to_string(),
            context: ErrorContext::new(),
            warnings: Vec::new(),
            errors: vec!["Error 2".to_string()],
        },
    )));

    assert_eq!(collection.len(), 2);

    let error = collection.into_error().unwrap();
    match error {
        XmileError::Multiple(errors) => {
            assert_eq!(errors.len(), 2);
        }
        _ => panic!("Expected Multiple error variant"),
    }
}

// Note: ParseError::from(XmileError) is implemented in src/xml/mod.rs
// but we don't test it here to avoid circular dependencies
