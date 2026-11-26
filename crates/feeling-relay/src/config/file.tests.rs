use super::*;

#[test]
fn creates_a_disabled_template_and_preserves_an_existing_profile() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profiles/default.toml");
    assert!(load(&path).unwrap().is_none());
    assert_eq!(fs::read_to_string(&path).unwrap(), TEMPLATE);
    let edited = TEMPLATE.replace("enabled = false", "enabled = false # local");
    fs::write(&path, &edited).unwrap();
    assert!(!create_profile(&path).unwrap());
    assert!(load(&path).unwrap().is_none());
    assert_eq!(fs::read_to_string(path).unwrap(), edited);
}

#[test]
fn provided_template_deserializes_when_enabled() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profile.toml");
    fs::write(&path, TEMPLATE.replace("enabled = false", "enabled = true")).unwrap();
    assert!(load(&path).unwrap().is_some());
}
