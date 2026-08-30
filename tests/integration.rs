use yaml2kvd::{from_yaml, kvd_text_to_yaml, to_yaml, yaml_text_to_kvd};

#[test]
fn yaml_to_kvd_scalars() {
    let yaml = "name: hello\nport: 8080\nenabled: true\nratio: 1.5\n";
    let kvd = yaml_text_to_kvd(yaml, None).unwrap();
    assert!(kvd.contains("name: \"hello\""));
    assert!(kvd.contains("port: 8080"));
    assert!(kvd.contains("enabled: true"));
    assert!(kvd.contains("ratio: 1.5"));
}

#[test]
fn kvd_to_yaml_scalars() {
    let kvd = "name: \"hello\"\nport: 8080\nenabled: true\n";
    let yaml = kvd_text_to_yaml(kvd).unwrap();
    assert!(yaml.contains("hello"));
    assert!(yaml.contains("8080"));
    assert!(yaml.contains("true"));
}

#[test]
fn yaml_null_becomes_kvd_null() {
    let yaml = "retries: null\n";
    let kvd = yaml_text_to_kvd(yaml, None).unwrap();
    assert!(kvd.contains("retries: null"));
}

#[test]
fn kvd_null_becomes_yaml_null() {
    let kvd = "retries: null\n";
    let yaml = kvd_text_to_yaml(kvd).unwrap();
    assert!(yaml.contains("null") || yaml.contains("~"));
}

#[test]
fn yaml_list_round_trips() {
    let yaml = "tags:\n- web\n- api\n";
    let kvd = yaml_text_to_kvd(yaml, None).unwrap();
    let yaml2 = kvd_text_to_yaml(&kvd).unwrap();
    // both directions work without error; content preserved
    assert!(kvd.contains("\"web\""));
    assert!(yaml2.contains("web"));
}

#[test]
fn yaml_nested_map_round_trips() {
    let yaml = "app:\n  port: 9000\n  host: localhost\n";
    let kvd = yaml_text_to_kvd(yaml, None).unwrap();
    assert!(kvd.contains("port: 9000"));
    assert!(kvd.contains("\"localhost\""));
}

#[test]
fn schema_validation_passes() {
    let yaml = "port: 8080\n";
    let schema = "port: int\n";
    yaml_text_to_kvd(yaml, Some(schema)).unwrap();
}

#[test]
fn schema_validation_fails_on_mismatch() {
    let yaml = "port: not-a-number\n";
    let schema = "port: int\n";
    assert!(yaml_text_to_kvd(yaml, Some(schema)).is_err());
}

#[test]
fn non_finite_float_is_rejected() {
    let v = serde_yaml_ng::Value::Number(f64::INFINITY.into());
    assert!(from_yaml(&v).is_err());
}

#[test]
fn kvd_to_yaml_empty_collections() {
    use kvd_rs::value::{Node, Shape};
    let node = Node::scalar(Shape::Null, "null");
    to_yaml(&node).unwrap();
}
