use penguin::prelude::*;

fn run_source(source: &str, using_unit: &PengUnit) -> (PengEnv, PengBindedCell) {
    let mut env = PengEnv::new();
    let unit = env
        .load_script_from_source_using(source, using_unit, 0)
        .unwrap();
    let result = env.run(unit.require_init().unwrap(), &unit).unwrap();

    (env, result)
}

fn assert_cell(result: &PengBindedCell, expected: PengCell) {
    assert!(result.value().equals(&expected));
}

fn assert_string(env: &PengEnv, result: &PengBindedCell, expected: &str) {
    let value = env.get_value_from_cell(result.value().clone()).unwrap();

    match value {
        PengValue::Box(PengBox::String(value)) => {
            assert_eq!(value, expected);
        }

        _ => {
            panic!("expected string result");
        }
    }
}

fn custom_resolver(
    expected_args_count: usize,
    result: PengCell,
) -> impl Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    move |ctx| {
        for arg in 0..expected_args_count {
            if ctx.get_arg_cell(arg).is_none() {
                return Err(PengError::InvalidState("missing custom arg".to_string()));
            }
        }

        if ctx.get_arg_cell(expected_args_count).is_some() {
            return Err(PengError::InvalidState("unexpected custom arg".to_string()));
        }

        let function_result = result.clone();
        let function =
            PengFunction::new_native(move |_ctx| Ok(PengBinded::Mutable(function_result.clone())));
        let ptr = ctx.create_value(PengValue::Box(PengBox::Function(function)));

        Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
    }
}

#[test]
fn builtin_operations_still_use_builtin_path() {
    let unit = PengUnit::library();

    let (_, result) = run_source("return 1 + 2", &unit);
    assert_cell(&result, PengCell::Int(3));

    let (_, result) = run_source("return 10 - 3", &unit);
    assert_cell(&result, PengCell::Int(7));

    let (_, result) = run_source("return 4 * 5", &unit);
    assert_cell(&result, PengCell::Int(20));

    let (_, result) = run_source("return 10 / 2", &unit);
    assert_cell(&result, PengCell::Int(5));

    let (_, result) = run_source("return 2 ** 8", &unit);
    assert_cell(&result, PengCell::Int(256));

    let (_, result) = run_source("return 10 % 3", &unit);
    assert_cell(&result, PengCell::Int(1));

    let (_, result) = run_source("return -10", &unit);
    assert_cell(&result, PengCell::Int(-10));

    let (env, result) = run_source("return \"abc\" .. \"def\"", &unit);
    assert_string(&env, &result, "abcdef");

    let (_, result) = run_source("return true & true", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return true & false", &unit);
    assert_cell(&result, PengCell::Bool(false));

    let (_, result) = run_source("return false | true", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return false | false", &unit);
    assert_cell(&result, PengCell::Bool(false));

    let (_, result) = run_source("return !true", &unit);
    assert_cell(&result, PengCell::Bool(false));

    let (_, result) = run_source("return 1 == 1", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return 1 != 2", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return 10 > 3", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return 10 >= 10", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return 2 < 3", &unit);
    assert_cell(&result, PengCell::Bool(true));

    let (_, result) = run_source("return 2 <= 2", &unit);
    assert_cell(&result, PengCell::Bool(true));
}

#[test]
fn custom_arithmetic_operations_still_work() {
    let mut unit = PengUnit::library();
    unit.register_custom_add(custom_resolver(2, PengCell::Int(101)))
        .unwrap();
    unit.register_custom_subtract(custom_resolver(2, PengCell::Int(102)))
        .unwrap();
    unit.register_custom_multiply(custom_resolver(2, PengCell::Int(103)))
        .unwrap();
    unit.register_custom_divide(custom_resolver(2, PengCell::Int(104)))
        .unwrap();
    unit.register_custom_power(custom_resolver(2, PengCell::Int(105)))
        .unwrap();
    unit.register_custom_remainder(custom_resolver(2, PengCell::Int(106)))
        .unwrap();

    let (_, result) = run_source("return [] + []", &unit);
    assert_cell(&result, PengCell::Int(101));

    let (_, result) = run_source("return [] - []", &unit);
    assert_cell(&result, PengCell::Int(102));

    let (_, result) = run_source("return [] * []", &unit);
    assert_cell(&result, PengCell::Int(103));

    let (_, result) = run_source("return [] / []", &unit);
    assert_cell(&result, PengCell::Int(104));

    let (_, result) = run_source("return [] ** []", &unit);
    assert_cell(&result, PengCell::Int(105));

    let (_, result) = run_source("return [] % []", &unit);
    assert_cell(&result, PengCell::Int(106));
}

#[test]
fn custom_new_unary_and_binary_operations_work() {
    let mut unit = PengUnit::library();
    unit.register_custom_negate(custom_resolver(1, PengCell::Int(201)))
        .unwrap();
    unit.register_custom_concat(custom_resolver(2, PengCell::Int(202)))
        .unwrap();
    unit.register_custom_and(custom_resolver(2, PengCell::Int(203)))
        .unwrap();
    unit.register_custom_or(custom_resolver(2, PengCell::Int(204)))
        .unwrap();
    unit.register_custom_not(custom_resolver(1, PengCell::Int(205)))
        .unwrap();
    unit.register_custom_equals(custom_resolver(2, PengCell::Int(206)))
        .unwrap();
    unit.register_custom_not_equals(custom_resolver(2, PengCell::Int(207)))
        .unwrap();
    unit.register_custom_greater_than(custom_resolver(2, PengCell::Int(208)))
        .unwrap();
    unit.register_custom_greater_equals_than(custom_resolver(2, PengCell::Int(209)))
        .unwrap();
    unit.register_custom_less_than(custom_resolver(2, PengCell::Int(210)))
        .unwrap();
    unit.register_custom_less_equals_than(custom_resolver(2, PengCell::Int(211)))
        .unwrap();

    let (_, result) = run_source("return -[]", &unit);
    assert_cell(&result, PengCell::Int(201));

    let (_, result) = run_source("return [] .. []", &unit);
    assert_cell(&result, PengCell::Int(202));

    let (_, result) = run_source("return [] & []", &unit);
    assert_cell(&result, PengCell::Int(203));

    let (_, result) = run_source("return [] | []", &unit);
    assert_cell(&result, PengCell::Int(204));

    let (_, result) = run_source("return ![]", &unit);
    assert_cell(&result, PengCell::Int(205));

    let (_, result) = run_source("return [] == []", &unit);
    assert_cell(&result, PengCell::Int(206));

    let (_, result) = run_source("return [] != []", &unit);
    assert_cell(&result, PengCell::Int(207));

    let (_, result) = run_source("return [] > []", &unit);
    assert_cell(&result, PengCell::Int(208));

    let (_, result) = run_source("return [] >= []", &unit);
    assert_cell(&result, PengCell::Int(209));

    let (_, result) = run_source("return [] < []", &unit);
    assert_cell(&result, PengCell::Int(210));

    let (_, result) = run_source("return [] <= []", &unit);
    assert_cell(&result, PengCell::Int(211));
}
