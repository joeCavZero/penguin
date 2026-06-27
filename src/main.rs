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
                            PengBox::Function(_) => print!("<function>"),
                            PengBox::Module(_) => print!("<module>"),
                            PengBox::Object(_) => print!("<object>"),
                            PengBox::Operation(_) => print!("<operation>"),
                            PengBox::String(s) => print!("{}", s),
                            PengBox::Thread(_) => print!("<thread>"),
                            PengBox::Type(_) => print!("<type>"),
                            PengBox::Vector(_) => print!("<vector>"),
                            PengBox::Union(_) => print!("<union>"),
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
                            PengBox::String(s) => {
                                return Ok(PengBinded::Mutable(PengCell::Uint(s.len())));
                            }
                            PengBox::Vector(v) => {
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
    peng.run(init).unwrap();
}
