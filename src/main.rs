use std::collections::HashMap;

fn peng_print(
    args: Vec<penguin::PengBindedCell>,
    env: &mut penguin::PengEnv,
) -> Result<penguin::PengBindedCell, penguin::PengError> {
    for arg in args {
        match arg.value() {
            penguin::PengCell::Bool(v) => print!("{}", v),
            penguin::PengCell::Byte(v) => print!("{}", v),
            penguin::PengCell::Float32(v) => print!("{}", v),
            penguin::PengCell::Float64(v) => print!("{}", v),
            penguin::PengCell::Int(v) => print!("{}", v),
            penguin::PengCell::Nil => print!("nil"),
            penguin::PengCell::Uint(v) => print!("{}", v),
            penguin::PengCell::Reference(ptr) => {
                if let Some(h) = env.get_heap(*ptr) {
                    match h {
                        penguin::PengHeapValue::Function(_) => print!("<function>"),
                        penguin::PengHeapValue::Module(_) => print!("<module>"),
                        penguin::PengHeapValue::Object(_) => print!("<object>"),
                        penguin::PengHeapValue::Operation(_) => print!("<operation>"),
                        penguin::PengHeapValue::String(s) => print!("{}", s),
                        penguin::PengHeapValue::Thread(_) => print!("<thread>"),
                        penguin::PengHeapValue::Type(_) => print!("<type>"),
                        penguin::PengHeapValue::Vector(_) => print!("<vector>"),
                        penguin::PengHeapValue::Union(_) => print!("<union>"),
                    }
                }
            }
        }
    }
    println!();
    return Ok(penguin::PengBindedCell::Mutable(penguin::PengCell::Nil));
}

fn peng_len(
    args: Vec<penguin::PengBindedCell>,
    env: &mut penguin::PengEnv,
) -> Result<penguin::PengBindedCell, penguin::PengError> {
    for arg in args {
        match arg.value() {
            penguin::PengCell::Reference(ptr) => {
                if let Some(h) = env.get_heap(*ptr) {
                    match h {
                        penguin::PengHeapValue::String(s) => {
                            return Ok(penguin::PengBinded::Mutable(penguin::PengCell::Uint(
                                s.len(),
                            )))
                        }
                        penguin::PengHeapValue::Vector(v) => {
                            return Ok(penguin::PengBinded::Mutable(penguin::PengCell::Uint(
                                v.len(),
                            )))
                        }
                        _ => {
                            return Err(penguin::PengError::NotImplemented(
                                "Expected string or vector".to_string(),
                            ))
                        }
                    }
                }
            }
            _ => {
                return Err(penguin::PengError::NotImplemented(
                    "Expected string or vector".to_string(),
                ))
            }
        }
    }
    println!();
    return Ok(penguin::PengBindedCell::Mutable(penguin::PengCell::Nil));
}

fn main() {
    match penguin::lex_file("main.p".to_string()) {
        Ok(tkns) => {
            match penguin::parse_script(tkns) {
                Ok(ast) => {
                    println!("ast: {:#?}", ast);
                    let mut env = penguin::PengEnv::new();
                    let f = penguin::PengBinded::Immutable(penguin::PengCell::Reference(
                        env.create_heap_value(penguin::PengHeapValue::Function(
                            penguin::PengFunction::new_native(peng_print),
                        )),
                    ));
                    env.set_global("print".to_string(), f).unwrap();
                    let f = penguin::PengBinded::Immutable(penguin::PengCell::Reference(
                        env.create_heap_value(penguin::PengHeapValue::Function(
                            penguin::PengFunction::new_native(peng_len),
                        )),
                    ));
                    env.set_global("len".to_string(), f).unwrap();
                    match penguin::generate_ast(&mut env, &ast, &HashMap::new()) {
                        Ok((_, init_ptr)) => {
                            if let penguin::PengHeapValue::Function(f) =
                                env.get_heap(init_ptr).unwrap()
                            {
                                if let penguin::PengFunction::Bytecode(b) = f {
                                    println!("bytecode de init:\n{:#?}", b.bytecode);
                                }
                            }
                            let thread_ptr = env.create_thread(init_ptr, 0, Vec::new());
                            loop {
                                match penguin::step_thread(&mut env, thread_ptr) {
                                    Ok(possible_result) => match possible_result {
                                        Some(_) => {
                                            return;
                                        }
                                        None => {}
                                    },
                                    Err(e) => {
                                        println!("execution:\n{:#?}", e);
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("ast generation:\n{:#?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("parsing:\n{:#?}", e);
                }
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
