fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            for tk in &tkns {
                println!("{}", tk.token_display())
            }
            println!("============================");
            match penguin::parse_program(tkns) {
                Ok(ast) => {
                    println!("============================");
                    let mut env = penguin::PengEnv::new();
                    env.create_global("g".to_string());
                    match penguin::generate_program(&mut env, &ast) {
                        Ok((globals, init_ptr)) => {
                            if let Some(init) = env.get_value(init_ptr).cloned() {
                                if let penguin::PengValue::Function(init_f) = init {
                                    if let penguin::PengFunction::Bytecode(init_btc) = init_f {
                                        println!("----- conts do init:\n{:#?}", init_btc.consts);
                                        println!("- - - - - - init.bytecode - - - - ");
                                        for i in init_btc.bytecode {
                                            println!("{:?}", i);
                                        }
                                        println!("----- env.values:\n{:#?}", env.values);
                                        println!("- - - - - - globals - - - - ");
                                        for (gn, gvp) in globals {
                                            println!("{} - {:?}", env.get_name(gn).cloned().unwrap(), env.values.get(&gvp));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
