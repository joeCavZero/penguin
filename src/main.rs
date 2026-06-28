use penguin::prelude::*;

fn main() {
    let mut peng = PengEnv::new();
    let mut pengstd = PengUnit::library();

    match pengstd.register_native_function(&mut peng, "print", move |ctx| {
        let mut index = 0usize;

        loop {
            let arg = match ctx.get_arg_cell(index) {
                Some(arg) => arg,
                None => break,
            };

            match arg.value() {
                PengCell::Bool(v) => print!("{}", v),
                PengCell::Byte(v) => print!("{}", v),
                PengCell::Float32(v) => print!("{}", v),
                PengCell::Float64(v) => print!("{}", v),
                PengCell::Int(v) => print!("{}", v),
                PengCell::Nil => print!("nil"),
                PengCell::Uint(v) => print!("{}", v),

                PengCell::Reference(ptr) => {
                    match ctx.get_value(*ptr) {
                        Some(PengValue::Box(PengBox::Function(_))) => print!("<function>"),
                        Some(PengValue::Box(PengBox::Module(_))) => print!("<module>"),
                        Some(PengValue::Box(PengBox::Object(_))) => print!("<object>"),
                        Some(PengValue::Box(PengBox::Operation(_))) => print!("<operation>"),
                        Some(PengValue::Box(PengBox::String(s))) => print!("{}", s),
                        Some(PengValue::Box(PengBox::Thread(_))) => print!("<thread>"),
                        Some(PengValue::Box(PengBox::Type(_))) => print!("<type>"),
                        Some(PengValue::Box(PengBox::Vector(_))) => print!("<vector>"),
                        Some(PengValue::Box(PengBox::Union(_))) => print!("<union>"),
                        Some(PengValue::Cell(cell)) => print!("{:?}", cell),
                        None => print!("<missing heap value>"),
                    }
                }
            }

            index += 1;
        }

        println!();

        Ok(PengBindedCell::Mutable(PengCell::Nil))
    }) {
        Ok(_) => {}
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }

    match pengstd.register_native_function(&mut peng, "len", move |ctx| {
        let arg = match ctx.get_arg_cell(0) {
            Some(arg) => arg,
            None => {
                return Err(PengError::NotImplemented(
                    "len expected string or vector".to_string(),
                ));
            }
        };

        match arg.value() {
            PengCell::Reference(ptr) => {
                match ctx.get_value(*ptr) {
                    Some(PengValue::Box(PengBox::String(s))) => {
                        Ok(PengBindedCell::Mutable(PengCell::Uint(s.len())))
                    }

                    Some(PengValue::Box(PengBox::Vector(v))) => {
                        Ok(PengBindedCell::Mutable(PengCell::Uint(v.len())))
                    }

                    _ => {
                        Err(PengError::NotImplemented(
                            "len expected string or vector".to_string(),
                        ))
                    }
                }
            }

            _ => {
                Err(PengError::NotImplemented(
                    "len expected string or vector".to_string(),
                ))
            }
        }
    }) {
        Ok(_) => {}
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }

    match pengstd.register_native_operation(&mut peng, "test_op", move |ctx| {
        let left = ctx.get_left_cell().clone();
        let right = ctx.get_right_cell().clone();

        println!("{:?} doing things on {:?}", left, right);

        Ok(PengBindedCell::Mutable(PengCell::Nil))
    }) {
        Ok(_) => {}
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }
    let unit = match peng.load_program_from_file_using("main.peng", &pengstd, 0) {
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

    match peng.run(init) {
        Ok(_) => {}
        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }

    match peng.run_function(&unit, "main", Vec::new()) {
        Ok(_) => {}
        Err(e) => println!("{:#?}", e),
    }
}