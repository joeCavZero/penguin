use penguin::prelude::*;

fn run_source(source: &str) -> Result<PengBindedCell, PengError> {
    let mut env = PengEnv::new();
    let unit = env.load_script_from_source_using(source, &PengUnit::library(), 0)?;

    env.run(unit.require_init()?, &unit)
}

fn assert_cell(result: &PengBindedCell, expected: PengCell) {
    assert!(result.value().equals(&expected));
}

#[test]
fn type_hints_still_work_after_type_cleanup() {
    let result = run_source(
        r#"
        var value: int = 40
        return value + 2
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Int(42));
}

#[test]
fn custom_types_and_methods_still_work_after_type_cleanup() {
    let result = run_source(
        r#"
        type Point {
            var x: int
            var y: int

            func sum(self) -> int {
                return self.x + self.y
            }
        }

        var point: Point = Point:{ x = 20, y = 22 }
        return point.sum()
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Int(42));
}

#[test]
fn super_types_still_work_after_type_cleanup() {
    let result = run_source(
        r#"
        type Base {
            var x: int
        }

        type Child: Base {
            var y: int
        }

        var child: Child = Child:{ x = 20, y = 22 }
        return child.x + child.y
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Int(42));
}

#[test]
fn pipe_in_type_hints_is_no_longer_accepted() {
    assert!(run_source("var value: int | uint = 1\nreturn value").is_err());
}

#[test]
fn type_prefix_is_optional_for_unambiguous_type_values() {
    let result = run_source(
        r#"
        var a = int
        var b = type int

        type Person {
            var name
        }

        var c = Person
        var d = type Person

        var number = [int, uint, f32, f64]
        var number_explicit = [type int, type uint, type f32, type f64]

        var vector_type = type [int]
        var meta_type = type type

        return true
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn bare_builtin_type_values_can_be_used_as_conversion_targets() {
    let result = run_source(
        r#"
        var int_value = 42 as int
        var uint_value = 42 as uint
        var byte_value = 42 as byte
        var f32_value = 42 as f32
        var f64_value = 42 as f64
        var string_value = 42 as string

        return int_value == 42
            & uint_value == 42u
            & byte_value == 42b
            & f32_value == 42.0f32
            & f64_value == 42.0f64
            & string_value == "42"
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn explicit_builtin_type_values_still_work_as_conversion_targets() {
    let result = run_source(
        r#"
        var int_value = 42 as type int
        var uint_value = 42 as type uint
        var byte_value = 42 as type byte
        var f32_value = 42 as type f32
        var f64_value = 42 as type f64
        var string_value = 42 as type string

        return int_value == 42
            & uint_value == 42u
            & byte_value == 42b
            & f32_value == 42.0f32
            & f64_value == 42.0f64
            & string_value == "42"
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn bare_and_explicit_custom_type_values_can_be_used_as_conversion_targets() {
    let result = run_source(
        r#"
        type Person {
            var name
        }

        var value = { name = "Ada" }
        var bare: Person = value as Person
        var explicit: Person = value as type Person

        return bare.name == "Ada" & explicit.name == "Ada"
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn ambiguous_structural_type_values_still_require_type_prefix() {
    let result = run_source(
        r#"
        var values = [1, 2, 3]
        var object_value = { name = "Ada" }

        return values[0] == 1 & object_value.name == "Ada"
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));

    let result = run_source(
        r#"
        var vector_type = type [int]
        var type_value = type { var name }

        return true
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn nil_remains_a_value_without_type_prefix_and_a_type_with_prefix() {
    let result = run_source(
        r#"
        var nil_value = nil
        var nil_type = type nil
        var converted = nil as nil_type

        return nil_value == converted
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn func_and_oper_are_type_values_without_literal_shape() {
    let result = run_source(
        r#"
        var function_type = func
        var explicit_function_type = type func
        var operation_type = oper
        var explicit_operation_type = type oper

        var function_value = func () { return 42 }
        var operation_value = oper (left, right) { return left + right }

        var converted_function = function_value as function_type
        var converted_function_explicit = function_value as explicit_function_type
        var converted_operation = operation_value as operation_type
        var converted_operation_explicit = operation_value as explicit_operation_type

        return converted_function() == 42
            & converted_function_explicit() == 42
            & 20 converted_operation 22 == 42
            & 20 converted_operation_explicit 22 == 42
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn mod_thread_any_and_type_meta_values_parse_consistently() {
    let result = run_source(
        r#"
        var module_type = mod
        var explicit_module_type = type mod
        var thread_type = thread
        var explicit_thread_type = type thread
        var any_type = any
        var explicit_any_type = type any
        var meta_type = type type
        var module_value = mod {}

        return true
        "#,
    )
    .unwrap();

    assert_cell(&result, PengCell::Bool(true));
}
