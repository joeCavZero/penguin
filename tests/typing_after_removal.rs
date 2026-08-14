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
