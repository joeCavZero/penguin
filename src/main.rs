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
    return Ok(penguin::PengBindedCell::Mutable(penguin::PengCell::Nil))
}

fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            match penguin::parse_script(tkns) {
                Ok(ast) => {
                    let mut env = penguin::PengEnv::new();
                    let f = penguin::PengCell::Reference(env.create_heap_value(penguin::PengHeapValue::Function(penguin::PengFunction::new_native(peng_print))));
                    let ff = penguin::PengBinded::Immutable(penguin::PengStated::Initialized(f));
                    env.set_global("print".to_string(), ff);
                    match penguin::generate_ast(&mut env, &ast, &HashMap::new()) {
                        Ok((_, init_ptr)) => {
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
                                        println!("ERROR:\n{:#?}", e);
                                        return;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                        }
                    }
                }
                Err(_) => {
                }
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
