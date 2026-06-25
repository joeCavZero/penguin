use std::collections::HashMap;

fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            for tk in &tkns {
                println!("{}", tk.token_display())
            }
            println!("============================");
            match penguin::parse_script(tkns) {
                Ok(ast) => {
                    println!("============================");
                    let mut env = penguin::PengEnv::new();
                    match penguin::generate_ast(&mut env, &ast, &HashMap::new()) {
                        Ok((globals, init_ptr)) => {
                            if let Some(cbinit) = env.get_heap(init_ptr).cloned() {
                                if let penguin::PengBinded::Mutable(stated_init) = cbinit.value {
                                    if let penguin::PengStated::Initialized(init) = stated_init {
                                        if let penguin::PengHeapValue::Function(init_f) = init {
                                            if let penguin::PengFunction::Bytecode(init_btc) = init_f {
                                                println!("----- conts do init:\n{:#?}", init_btc.consts);
                                                println!("- - - - - - init.bytecode - - - - ");
                                                for i in init_btc.bytecode {
                                                    println!("{:?}", i);
                                                }
                                                println!("----- env.values antes de rodar o init:\n{:#?}", env.heap);
                                                println!("- - - - - - globals - - - - ");
                                                for (gn, gvp) in globals {
                                                    println!("{} - {:?}", env.get_pooled_name(gn).cloned().unwrap(), env.heap.get(&gvp));
                                                }
                                                
                                            }
                                        }
                                    }
                                }
                            }
                            let thread_ptr = env.create_thread(init_ptr, 0, Vec::new());
                            loop {
                                match penguin::step_thread(&mut env, thread_ptr) {
                                    Ok(possible_result) => {
                                        match possible_result {
                                            Some(value) => {
                                                println!(" O RESULTADO É...\n   {:?}", value);
                                                return;
                                            }
                                            None => {},
                                        }
                                    }
                                    Err(e) => {
                                        println!("ERRO::::: {:?}", e);
                                        println!("----- env.values depois de rodar o init e dar erro:\n{:#?}", env.heap);

                                        return;
                                    }
                                }
                                if let penguin::PengStated::Initialized(t) = env.get_heap(thread_ptr).unwrap().value.value() {
                                    if let penguin::PengHeapValue::Thread(tt) = t {
                                        println!("stack nesse momento: \n {:?}\n-------------", tt.stack)
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
