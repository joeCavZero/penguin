use penguin::prelude::*;
use std::time::Instant;

fn main() {
    let total_start = Instant::now();

    let mut peng = PengEnv::new();
    let mut pengstd = PengUnit::library();

    pengstd
        .register_native_function(&mut peng, "print", move |args, env| {
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
            Ok(PengBindedCell::Mutable(PengCell::Nil))
        })
        .unwrap();

    pengstd
        .register_native_function(&mut peng, "len", move |args, env| {
            for arg in args {
                match arg.value() {
                    PengCell::Reference(ptr) => {
                        if let Some(h) = env.get_heap(*ptr) {
                            match h {
                                PengValue::Box(PengBox::String(s)) => {
                                    return Ok(PengBindedCell::Mutable(PengCell::Uint(s.len())));
                                }
                                PengValue::Box(PengBox::Vector(v)) => {
                                    return Ok(PengBindedCell::Mutable(PengCell::Uint(v.len())));
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

            Ok(PengBindedCell::Mutable(PengCell::Nil))
        })
        .unwrap();

    pengstd
        .register_native_operation(&mut peng, "test_op", move |(a, b), _env| {
            println!("{:?} doing things on {:?}", a, b);
            Ok(PengBindedCell::Mutable(PengCell::Nil))
        })
        .unwrap();

    let load_start = Instant::now();

    let unit = peng
        .load_program_from_file_using("main.peng", &pengstd)
        .unwrap();

    let load_time = load_start.elapsed();

    let init_start = Instant::now();

    peng.run(unit.require_init().unwrap()).unwrap();

    let init_time = init_start.elapsed();

    let main_start = Instant::now();

    peng.run_function(&unit, "main", Vec::new()).unwrap();

    let main_time = main_start.elapsed();
    let total_time = total_start.elapsed();

    println!("==== Penguin benchmark ====");
    println!("load/compile: {:?}", load_time);
    println!("init:         {:?}", init_time);
    println!("main:         {:?}", main_time);
    println!("total:        {:?}", total_time);
}