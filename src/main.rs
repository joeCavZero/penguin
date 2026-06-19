fn main() {
    match penguin::lex_file("main.p".to_string(), 0) {
        Ok(tkns) => {
            for tk in tkns {
                println!("{}", tk.token_display())
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
