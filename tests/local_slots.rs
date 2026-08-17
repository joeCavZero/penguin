use penguin::prelude::*;

fn run_main(source: &str) -> Result<PengBindedCell, PengError> {
    let mut env = PengEnv::new();
    let unit = match env.load_program_from_source(source, 0) {
        Ok(unit) => unit,
        Err(e) => return Err(e),
    };

    let init = match unit.require_init() {
        Ok(init) => init,
        Err(e) => return Err(e),
    };

    match env.run(init, &unit) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    env.run_global_function("main", &unit, Vec::new())
}

fn vector_len_access(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match ctx.get_arg_value(0) {
        Some(value) => value,
        None => {
            return Err(PengError::InvalidState("missing len receiver".to_string()));
        }
    };

    match value.value() {
        PengValue::Box(PengBox::Vector(vector)) => {
            Ok(PengBinded::Mutable(PengCell::Uint(vector.values.len())))
        }
        _ => Err(PengError::InvalidState(
            "len receiver is not a vector".to_string(),
        )),
    }
}

fn native_sqrt(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match ctx.get_arg_cell(0) {
        Some(value) => value.value().clone(),
        None => {
            return Err(PengError::InvalidState("missing sqrt argument".to_string()));
        }
    };

    match value {
        PengCell::Float32(value) => Ok(PengBinded::Mutable(PengCell::Float32(value.sqrt()))),
        PengCell::Float64(value) => Ok(PengBinded::Mutable(PengCell::Float32(value.sqrt() as f32))),
        PengCell::Int(value) => Ok(PengBinded::Mutable(PengCell::Float32(
            (value as f32).sqrt(),
        ))),
        PengCell::Uint(value) => Ok(PengBinded::Mutable(PengCell::Float32(
            (value as f32).sqrt(),
        ))),
        PengCell::Byte(value) => Ok(PengBinded::Mutable(PengCell::Float32(
            (value as f32).sqrt(),
        ))),
        _ => Err(PengError::InvalidState(
            "sqrt argument is not numeric".to_string(),
        )),
    }
}

fn run_main_with_native_len_and_sqrt(source: &str) -> Result<PengBindedCell, PengError> {
    let mut env = PengEnv::new();
    let mut using_unit = PengUnit::library();

    match using_unit.register_custom_access(&mut env, "len", vector_len_access) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match using_unit.register_immutable_native_function(&mut env, "sqrt", native_sqrt) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let unit = match env.load_program_from_source_using(source, &using_unit, 0) {
        Ok(unit) => unit,
        Err(e) => return Err(e),
    };

    let init = match unit.require_init() {
        Ok(init) => init,
        Err(e) => return Err(e),
    };

    match env.run(init, &unit) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    env.run_global_function("main", &unit, Vec::new())
}

fn assert_cell(result: &PengBindedCell, expected: PengCell) {
    assert!(result.value().equals(&expected));
}

#[test]
fn loop_index_survives_function_call_assignment() {
    let result = run_main(
        r#"
        func value() {
            return 1.0
        }

        func main() {
            var a = 0.0
            var sum = 0

            for (var i = 0; i < 3; i = i + 1) {
                a = value()
                sum = sum + i
            }

            return sum
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(3)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn several_locals_in_same_scope_keep_distinct_slots() {
    let result = run_main(
        r#"
        func value() {
            return 5
        }

        func main() {
            var a = 1
            var b = 10
            var c = value()
            var d = 100

            return a + b + c + d
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(116)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn nested_blocks_and_function_calls_keep_outer_locals() {
    let result = run_main(
        r#"
        func value() {
            return 7
        }

        func main() {
            var a = 1
            {
                var b = value()
                {
                    var c = value()
                    a = a + b + c
                }
                a = a + b
            }

            return a
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(22)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn sequential_loops_do_not_collide() {
    let result = run_main(
        r#"
        func value() {
            return 2
        }

        func main() {
            var sum = 0

            for (var i = 0; i < 3; i = i + 1) {
                var inside = value()
                sum = sum + i + inside
            }

            for (var j = 0; j < 2; j = j + 1) {
                var inside = value()
                sum = sum + j + inside
            }

            return sum
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(14)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn nested_loops_keep_indices_and_inner_locals_distinct() {
    let result = run_main(
        r#"
        func value() {
            return 1
        }

        func main() {
            var sum = 0

            for (var i = 0; i < 3; i = i + 1) {
                for (var j = 0; j < 2; j = j + 1) {
                    var inside = value()
                    sum = sum + i + j + inside
                }
            }

            return sum
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(15)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn parameters_and_locals_keep_distinct_slots() {
    let result = run_main(
        r#"
        func value(x) {
            var local = 10
            {
                var nested = x + local
                return nested
            }
        }

        func main() {
            var a = 4
            var b = value(a)
            return a + b
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Int(18)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn indexed_value_passed_to_normal_call_preserves_loop_index() {
    let result = run_main(
        r#"
        func get_length(value) {
            return 1.0
        }

        func test(values) {
            for (var i = 0; i < 3; i = i + 1) {
                var result = get_length(values[i])

                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([1, 2, 3])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn indexed_receiver_method_call_preserves_loop_index() {
    let result = run_main(
        r#"
        type V {
            var value

            func length(self) {
                return 1.0
            }
        }

        func test() {
            var values = [
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ]

            for (var i = 0; i < 3; i = i + 1) {
                var result = values[i].length()

                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test()
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn indexed_receiver_method_call_as_nested_argument_preserves_loop_index() {
    let result = run_main(
        r#"
        func scalar_close(left, right, epsilon) {
            var diff = left - right

            if diff < 0 {
                diff = -diff
            }

            return diff <= epsilon
        }

        type V {
            var value

            func length(self) {
                return 1.0
            }
        }

        func test() {
            var values = [
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ]

            for (var i = 0; i < 3; i = i + 1) {
                if !scalar_close(
                    values[i].length(),
                    1.0,
                    0.000001
                ) {
                    return false
                }
            }

            return true
        }

        func main() {
            return test()
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn native_len_in_for_condition_preserves_loop_index() {
    let result = run_main_with_native_len_and_sqrt(
        r#"
        func test(values) {
            for (var i = 0; i < values.len(); i = i + 1) {
                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([1, 2, 3])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn native_len_and_simple_bytecode_method_preserve_loop_index() {
    let result = run_main_with_native_len_and_sqrt(
        r#"
        type V {
            var value

            func length(self) {
                return 1.0
            }
        }

        func test(values) {
            for (var i = 0; i < values.len(); i = i + 1) {
                var x = values[i].length()

                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn native_len_and_nested_bytecode_methods_preserve_loop_index() {
    let result = run_main_with_native_len_and_sqrt(
        r#"
        type V {
            var value

            func dot(self, other) {
                var result = 0.0

                for (var j = 0; j < 3; j = j + 1) {
                    result = result + 0.33333334
                }

                return result
            }

            func length_squared(self) {
                return self.dot(self)
            }

            func length(self) {
                return self.length_squared()
            }
        }

        func test(values) {
            for (var i = 0; i < values.len(); i = i + 1) {
                var x = values[i].length()

                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn native_return_inside_bytecode_method_preserves_loop_index() {
    let result = run_main_with_native_len_and_sqrt(
        r#"
        type V {
            var value

            func dot(self, other) {
                var result = 0.0

                for (var j = 0; j < 3; j = j + 1) {
                    result = result + 0.33333334
                }

                return result
            }

            func length_squared(self) {
                return self.dot(self)
            }

            func length(self) {
                return sqrt(self.length_squared())
            }
        }

        func test(values) {
            for (var i = 0; i < values.len(); i = i + 1) {
                var x = values[i].length()

                if i != 0 && i != 1 && i != 2 {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn native_len_nested_methods_native_return_and_scalar_close_preserve_loop_index() {
    let result = run_main_with_native_len_and_sqrt(
        r#"
        func scalar_close(left, right, epsilon) {
            var difference = left - right

            if difference < 0 {
                difference = -difference
            }

            return difference <= epsilon
        }

        type V {
            var value

            func dot(self, other) {
                var result = 0.0

                for (var j = 0; j < 3; j = j + 1) {
                    result = result + 0.33333334
                }

                return result
            }

            func length_squared(self) {
                return self.dot(self)
            }

            func length(self) {
                return sqrt(self.length_squared())
            }
        }

        func test(values) {
            for (var i = 0; i < values.len(); i = i + 1) {
                if !scalar_close(
                    values[i].length(),
                    1.0,
                    0.000001
                ) {
                    return false
                }
            }

            return true
        }

        func main() {
            return test([
                V:{ value = 1 },
                V:{ value = 2 },
                V:{ value = 3 }
            ])
        }
        "#,
    );

    match result {
        Ok(cell) => assert_cell(&cell, PengCell::Bool(true)),
        Err(e) => panic!("main failed: {:?}", e),
    }
}

#[test]
fn slots_can_be_reused_after_scope_ends() {
    let mut env = PengEnv::new();
    let unit = match env.load_program_from_source(
        r#"
        func main() {
            {
                var a = 1
                var b = 2
            }

            {
                var c = 3
                var d = 4
                return c + d
            }
        }
        "#,
        0,
    ) {
        Ok(unit) => unit,
        Err(e) => panic!("program failed to load: {:?}", e),
    };

    let init = match unit.require_init() {
        Ok(init) => init,
        Err(e) => panic!("program init missing: {:?}", e),
    };

    match env.run(init, &unit) {
        Ok(_) => {}
        Err(e) => panic!("program init failed: {:?}", e),
    }

    let result = match env.run_global_function("main", &unit, Vec::new()) {
        Ok(result) => result,
        Err(e) => panic!("main failed: {:?}", e),
    };
    assert_cell(&result, PengCell::Int(7));

    let name = env.ensure_pooled_name_ptr("main".to_string());
    let function_ptr = match unit.get_global(name) {
        Some(value) => *value.value(),
        None => panic!("main global missing"),
    };
    let function = match env.get_heap(function_ptr) {
        Some(PengValue::Box(PengBox::Function(PengFunction::Bytecode(function)))) => function,
        Some(_) => panic!("main is not a bytecode function"),
        None => panic!("main heap value missing"),
    };

    let mut highest_store = 0;
    for instruction in &function.bytecode {
        match instruction {
            PengInstruction::StoreLocal(local) => {
                if *local > highest_store {
                    highest_store = *local;
                }
            }
            _ => {}
        }
    }

    assert_eq!(highest_store, 1);
}
