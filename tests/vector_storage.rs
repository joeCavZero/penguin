use penguin::*;

#[test]
fn vector_materialization_copies_values_and_preserves_references() {
    let mut env = PengEnv::new();
    let referenced_value = env.create_value(PengValue::String("shared".to_string()));

    let vector_ptr = env.create_vector_from_cells([
        PengCell::Int(10),
        PengCell::Bool(true),
        PengCell::Reference(referenced_value),
    ]);

    let vector = match env.get_value(vector_ptr) {
        Some(PengValue::Vector(vector)) => vector,
        _ => panic!("expected vector"),
    };

    assert_eq!(vector.len(), 3);

    let int_ptr = vector.get(0).expect("int pointer");
    let bool_ptr = vector.get(1).expect("bool pointer");
    let preserved_ptr = vector.get(2).expect("reference pointer");

    assert_ne!(int_ptr, referenced_value);
    assert_ne!(bool_ptr, referenced_value);
    assert_eq!(preserved_ptr, referenced_value);
    assert!(env
        .get_value(int_ptr)
        .is_some_and(|value| value.equals(&PengValue::Int(10))));
    assert!(env
        .get_value(bool_ptr)
        .is_some_and(|value| value.equals(&PengValue::Bool(true))));
    assert!(env.get_value(preserved_ptr).is_some_and(|value| {
        value.equals(&PengValue::String("shared".to_string()))
    }));
}
