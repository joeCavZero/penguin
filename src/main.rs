fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            for tk in &tkns {
                println!("{}", tk.token_display())
            }
            println!("============================");
            match penguin::parse(tkns) {
                Ok(ast) => {
                    println!("{:#?}", ast);
                    println!("============================");
                    ast.pretty_print();
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
