fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            for tk in &tkns {
                println!("{}", tk.token_display())
            }
            println!("============================");
            match penguin::parse_program(tkns) {
                Ok(ast) => {
                    println!("{:#?}", ast);
                    println!("============================");
                    ast.pretty_print();
                    println!("============================");
                    let mut env = penguin::PengEnv::new();
                    match penguin::generate_program(&mut env, &ast) {
                        Ok(init_ptr) => {
                            if let Some(init) = env.get_value(init_ptr).cloned() {
                                if let penguin::PengValue::Function(init_f) = init {
                                    if let penguin::PengFunction::Bytecode(init_btc) = init_f {
                                        println!("{:#?}", init_btc.consts);
                                        println!("- - - - - - - - - - ");
                                        for i in init_btc.bytecode {
                                            println!("{:?}", i);
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
