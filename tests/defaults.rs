#[test]
fn default_entity() {
    let entity = siren_types::Entity::default();

    assert_eq!(
        entity.properties,
        serde_json::Value::Object(serde_json::Map::default())
    )
}
