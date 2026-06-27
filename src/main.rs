use penguin::prelude::*;

fn main() {
    let mut peng = PengEnv::new();

    peng.register_native_function("print", move |args, env| {
        for arg in args {
            match arg.value() {
                PengCell::Bool(v) => print!("{}", v),
                PengCell::Byte(v) => print!("{}", v),
                PengCell::Float32(v) => print!("{}", v),
                PengCell::Float64(v) => print!("{}", v),
                PengCell::Int(v) => print!("{}", v),
                PengCell::Nil => print!("nil"),
                PengCell::Uint(v) => print!("{}", v),
                PengCell::Reference(ptr) => {
                    if let Some(h) = env.get_heap(*ptr) {
                        match h {
                            PengValue::Box(PengBox::Function(_)) => print!("<function>"),
                            PengValue::Box(PengBox::Module(_)) => print!("<module>"),
                            PengValue::Box(PengBox::Object(_)) => print!("<object>"),
                            PengValue::Box(PengBox::Operation(_)) => print!("<operation>"),
                            PengValue::Box(PengBox::String(s)) => print!("{}", s),
                            PengValue::Box(PengBox::Thread(_)) => print!("<thread>"),
                            PengValue::Box(PengBox::Type(_)) => print!("<type>"),
                            PengValue::Box(PengBox::Vector(_)) => print!("<vector>"),
                            PengValue::Box(PengBox::Union(_)) => print!("<union>"),
                            PengValue::Cell(cell) => print!("{:?}", cell),
                        }
                    }
                }
            }
        }
        println!();
        return Ok(PengBindedCell::Mutable(PengCell::Nil));
    })
    .unwrap();
    peng.register_native_function("len", move |args, env| {
        for arg in args {
            match arg.value() {
                PengCell::Reference(ptr) => {
                    if let Some(h) = env.get_heap(*ptr) {
                        match h {
                            PengValue::Box(PengBox::String(s)) => {
                                return Ok(PengBinded::Mutable(PengCell::Uint(s.len())));
                            }
                            PengValue::Box(PengBox::Vector(v)) => {
                                return Ok(PengBinded::Mutable(PengCell::Uint(v.len())));
                            }
                            _ => {
                                return Err(PengError::NotImplemented(
                                    "Expected string or vector".to_string(),
                                ));
                            }
                        }
                    }
                }
                _ => {
                    return Err(PengError::NotImplemented(
                        "Expected string or vector".to_string(),
                    ));
                }
            }
        }
        println!();
        return Ok(PengBindedCell::Mutable(PengCell::Nil));
    })
    .unwrap();
    peng.register_native_operation("test_op", move |(a, b), _env| {
        println!("{:?} doing things on {:?}", a, b);
        return Ok(PengBindedCell::Mutable(PengCell::Nil));
    }).unwrap();



    let init = peng.load_script_from_file("main.peng").unwrap();
    peng.run(init.init()).unwrap();
}
