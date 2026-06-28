use penguin::prelude::*;

fn main() {
    let mut peng = PengEnv::new();
    let mut core = PengUnit::library();

    let mut io = PengUnit::library();
    let mut thread_module = PengUnit::library();

    io.register_native_function(&mut peng, "println", move |ctx| {
        let mut index = 0usize;

        loop {
            let arg = match ctx.get_arg_cell(index) {
                Some(arg) => arg,
                None => break,
            };

            if index > 0 {
                print!(" ");
            }

            print_binded_cell(ctx, arg);

            index += 1;
        }

        println!();

        Ok(PengBindedCell::Mutable(PengCell::Nil))
    })
    .unwrap();
    core.register_native_function(&mut peng, "len", move |ctx| {
        let arg = match ctx.get_arg_cell(0) {
            Some(arg) => arg,
            None => {
                return Err(PengError::NotImplemented(
                    "len expected string or vector".to_string(),
                ));
            }
        };

        match arg.value() {
            PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
                Some(PengValue::Box(PengBox::String(s))) => {
                    Ok(PengBindedCell::Mutable(PengCell::Uint(s.len())))
                }

                Some(PengValue::Box(PengBox::Vector(v))) => {
                    Ok(PengBindedCell::Mutable(PengCell::Uint(v.len())))
                }

                _ => Err(PengError::NotImplemented(
                    "len expected string or vector".to_string(),
                )),
            },

            _ => Err(PengError::NotImplemented(
                "len expected string or vector".to_string(),
            )),
        }
    })
    .unwrap();

    core.register_custom_access(&mut peng, "len", |ctx| {
        if let Some(value) = ctx.get_arg_value(0) {
            let len = match value.value() {
                PengValue::Box(PengBox::Vector(v)) => v.values.len(),

                PengValue::Box(PengBox::Object(o)) => o.fields.len(),

                PengValue::Box(PengBox::Module(m)) => m.members.len(),

                PengValue::Box(PengBox::Type(PengType::Custom(t))) => t.fields.len(),

                PengValue::Box(PengBox::String(s)) => s.chars().count(),

                _ => {
                    return Err(PengError::CannotCallValue(
                        "len() not supported for this value".into(),
                    ));
                }
            };

            return Ok(PengBinded::Mutable(PengCell::Uint(len)));
        } else {
        }
        Err(PengError::CannotCallValue(
            "len() not supported for this value".into(),
        ))
    })
    .unwrap();

    core.register_custom_access(&mut peng, "sum", |ctx| {
        let value = match ctx.get_arg_value(0) {
            Some(value) => value,
            None => {
                return Err(PengError::CannotCallValue("sum() expected receiver".into()));
            }
        };

        let vector = match value.value() {
            PengValue::Box(PengBox::Vector(v)) => v,
            _ => {
                return Err(PengError::CannotCallValue("sum() expected vector".into()));
            }
        };

        let mut sum = PengCell::Int(0);

        for item in &vector.values {
            sum = match (&sum, item.value()) {
                (PengCell::Int(left), PengCell::Int(right)) => PengCell::Int(left + right),

                (PengCell::Uint(left), PengCell::Uint(right)) => PengCell::Uint(left + right),

                (PengCell::Byte(left), PengCell::Byte(right)) => PengCell::Byte(left + right),

                (PengCell::Float32(left), PengCell::Float32(right)) => {
                    PengCell::Float32(left + right)
                }

                (PengCell::Float64(left), PengCell::Float64(right)) => {
                    PengCell::Float64(left + right)
                }

                // primeiro item define o tipo real da soma
                (PengCell::Int(0), PengCell::Uint(right)) => PengCell::Uint(*right),

                (PengCell::Int(0), PengCell::Byte(right)) => PengCell::Byte(*right),

                (PengCell::Int(0), PengCell::Float32(right)) => PengCell::Float32(*right),

                (PengCell::Int(0), PengCell::Float64(right)) => PengCell::Float64(*right),

                _ => {
                    return Err(PengError::CannotCallValue(
                        "sum() expected vector with numbers of the same type".into(),
                    ));
                }
            };
        }

        Ok(PengBinded::Mutable(sum))
    })
    .unwrap();

    core.register_custom_access(&mut peng, "push", |ctx| {
        let item = match ctx.get_arg_cell(1) {
            Some(item) => item.clone(),
            None => {
                return Err(PengError::CannotCallValue(
                    "push() expected one argument".into(),
                ));
            }
        };

        let mut value = match ctx.get_arg_value_mut(0) {
            Some(value) => value,
            None => {
                return Err(PengError::CannotCallValue(
                    "push() expected mutable vector".into(),
                ));
            }
        };

        let vector = match value.value_mut() {
            PengValue::Box(PengBox::Vector(v)) => v,
            _ => {
                return Err(PengError::CannotCallValue(
                    "push() expected mutable vector".into(),
                ));
            }
        };

        vector.values.push(item);

        Ok(PengBinded::Mutable(PengCell::Nil))
    })
    .unwrap();

    core.register_custom_access(&mut peng, "keys", |ctx| {
        let names: Vec<String> = match ctx.get_arg_value(0) {
            Some(value) => match value.value() {
                PengValue::Box(PengBox::Object(obj)) => obj
                    .fields
                    .keys()
                    .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                    .collect(),

                PengValue::Box(PengBox::Module(module)) => module
                    .members
                    .keys()
                    .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                    .collect(),

                PengValue::Box(PengBox::Type(PengType::Custom(ty))) => ty
                    .fields
                    .keys()
                    .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                    .collect(),

                _ => {
                    return Err(PengError::CannotCallValue(
                        "keys() expected object, module or type".into(),
                    ));
                }
            },

            None => {
                return Err(PengError::CannotCallValue(
                    "keys() expected receiver".into(),
                ));
            }
        };

        let values = names
            .into_iter()
            .map(|name| {
                let ptr = ctx
                    .env_mut()
                    .create_heap_value(PengValue::Box(PengBox::String(name)));

                PengBinded::Mutable(PengCell::Reference(ptr))
            })
            .collect();

        let ptr = ctx
            .env_mut()
            .create_heap_value(PengValue::Box(PengBox::Vector(PengVector::new(values))));

        Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
    })
    .unwrap();

    thread_module
        .register_native_function(&mut peng, "spawn", |ctx| {
            let function_ptr = match ctx.get_arg_cell(0) {
                Some(arg) => match arg.value() {
                    PengCell::Reference(ptr) => *ptr,
                    _ => {
                        return Err(PengError::CannotCallValue(
                            "Thread:spawn() expected function as first argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "Thread:spawn() expected function".into(),
                    ));
                }
            };

            match ctx.get_value(function_ptr) {
                Some(PengValue::Box(PengBox::Function(_))) => {}
                _ => {
                    return Err(PengError::CannotCallValue(
                        "Thread:spawn() expected function as first argument".into(),
                    ));
                }
            }

            let mut params = Vec::new();
            let mut index = 1usize;

            loop {
                match ctx.get_arg_cell(index) {
                    Some(arg) => params.push(arg.clone()),
                    None => break,
                }

                index += 1;
            }

            let thread = PengThread::new(function_ptr, 0, params, PengThreadState::Running);

            let ptr = ctx.create_box(PengBox::Thread(thread));

            ctx.env_mut().activate_thread(ptr);

            Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
        })
        .unwrap();

    core.register_custom_access(&mut peng, "resume", |ctx| {
        let mut value = match ctx.get_arg_value_mut(0) {
            Some(value) => value,
            None => {
                return Err(PengError::CannotCallValue(
                    "resume() expected thread".into(),
                ));
            }
        };

        match value.value_mut() {
            PengValue::Box(PengBox::Thread(t)) => {
                t.state = PengThreadState::Running;
                Ok(PengBinded::Mutable(PengCell::Nil))
            }

            _ => Err(PengError::CannotCallValue(
                "resume() expected thread".into(),
            )),
        }
    })
    .unwrap();

    core.register_custom_access(&mut peng, "pause", |ctx| {
        let mut value = match ctx.get_arg_value_mut(0) {
            Some(value) => value,
            None => {
                return Err(PengError::CannotCallValue("pause() expected thread".into()));
            }
        };

        match value.value_mut() {
            PengValue::Box(PengBox::Thread(t)) => {
                t.state = PengThreadState::Paused;
                Ok(PengBinded::Mutable(PengCell::Nil))
            }

            _ => Err(PengError::CannotCallValue("pause() expected thread".into())),
        }
    })
    .unwrap();

    core.register_custom_access(&mut peng, "cancel", |ctx| {
        let mut value = match ctx.get_arg_value_mut(0) {
            Some(value) => value,
            None => {
                return Err(PengError::CannotCallValue(
                    "cancel() expected thread".into(),
                ));
            }
        };

        match value.value_mut() {
            PengValue::Box(PengBox::Thread(t)) => {
                t.state = PengThreadState::Cancelled;
                Ok(PengBinded::Mutable(PengCell::Nil))
            }

            _ => Err(PengError::CannotCallValue(
                "cancel() expected thread".into(),
            )),
        }
    })
    .unwrap();

    core.register_custom_access(&mut peng, "state", |ctx| {
        let state = match ctx.get_arg_value(0) {
            Some(value) => match value.value() {
                PengValue::Box(PengBox::Thread(t)) => match t.state {
                    PengThreadState::Running => "running",
                    PengThreadState::Finished => "finished",
                    PengThreadState::Paused => "paused",
                    PengThreadState::Waiting => "waiting",
                    PengThreadState::Cancelled => "cancelled",
                    PengThreadState::Failed => "failed",
                },

                _ => {
                    return Err(PengError::CannotCallValue("state() expected thread".into()));
                }
            },

            None => {
                return Err(PengError::CannotCallValue("state() expected thread".into()));
            }
        };

        let ptr = ctx
            .env_mut()
            .create_heap_value(PengValue::Box(PengBox::String(state.to_string())));

        Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
    })
    .unwrap();

    thread_module
        .register_native_function(&mut peng, "yield", |ctx| {
            match ctx.yield_now() {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
            Ok(PengBinded::Mutable(PengCell::Nil))
        })
        .unwrap();

    core.register_module(&mut peng, "io", &io).unwrap();
    core.register_module(&mut peng, "Thread", &thread_module)
        .unwrap();

    let core_for_import = core.clone();

    core.register_native_function(&mut peng, "import", move |ctx| {
        let path = match ctx.get_arg_value(0) {
            Some(value) => match value.value() {
                PengValue::Box(PengBox::String(s)) => s.clone(),
                _ => {
                    return Err(PengError::CannotCallValue(
                        "import() expected string path".into(),
                    ));
                }
            },

            None => {
                return Err(PengError::CannotCallValue("import() expected path".into()));
            }
        };

        let unit = match ctx
            .env_mut()
            .load_program_from_file_using(&path, &core_for_import, 0)
        {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let init = match unit.require_init() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match ctx.env_mut().run_isolated(init, &unit) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let module_ptr = unit.create_module_heap(ctx.env_mut());

        Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)))
    })
    .unwrap();

    let unit = match peng.load_program_from_file_using("main.peng", &core, 0) {
        Ok(unit) => unit,
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    };

    let init = match unit.require_init() {
        Ok(init) => init,
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    };

    match peng.run(init, &unit) {
        Ok(_) => {}
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }

    match peng.run_global_function("main", &unit, Vec::new()) {
        Ok(_) => {}
        Err(e) => println!("{:#?}", e),
    }
}

fn print_cell(ctx: &PengNativeFunctionCallContext, cell: &PengCell) {
    match cell {
        PengCell::Bool(v) => print!("{}", v),
        PengCell::Byte(v) => print!("{}", v),
        PengCell::Float32(v) => print!("{}", v),
        PengCell::Float64(v) => print!("{}", v),
        PengCell::Int(v) => print!("{}", v),
        PengCell::Nil => print!("nil"),
        PengCell::Uint(v) => print!("{}", v),

        PengCell::Reference(ptr) => print_value_ref(ctx, *ptr),
    }
}

fn print_binded_cell(ctx: &PengNativeFunctionCallContext, cell: &PengBindedCell) {
    print_cell(ctx, cell.value());
}

fn print_value_ref(ctx: &PengNativeFunctionCallContext, ptr: PengHeapPtr) {
    match ctx.get_value(ptr) {
        Some(value) => print_value(ctx, value),
        None => print!("<missing heap value>"),
    }
}

fn print_value(ctx: &PengNativeFunctionCallContext, value: &PengValue) {
    match value {
        PengValue::Cell(cell) => print_cell(ctx, cell),

        PengValue::Box(PengBox::Function(_)) => print!("<function>"),
        PengValue::Box(PengBox::Operation(_)) => print!("<operation>"),
        PengValue::Box(PengBox::Thread(_)) => print!("<thread>"),
        PengValue::Box(PengBox::Type(_)) => print!("<type>"),
        PengValue::Box(PengBox::Union(_)) => print!("<union>"),

        PengValue::Box(PengBox::String(s)) => print!("{}", s),

        PengValue::Box(PengBox::Vector(v)) => {
            print!("[");
            for (i, value) in v.values.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                print_binded_cell(ctx, value);
            }
            print!("]");
        }

        PengValue::Box(PengBox::Object(o)) => {
            print!("{{");
            for (i, (name, value)) in o.fields.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                print_binded_cell(ctx, value);
            }
            print!("}}");
        }

        PengValue::Box(PengBox::Module(m)) => {
            print!("module {{");
            for (i, (name, value)) in m.members.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                print_binded_cell(ctx, value);
            }
            print!("}}");
        }
    }
}
