mod sourcepos;
mod token;

use token::{Tokenizer, Type};

const text: &'static str = "(abc def)";

fn main() {
    let mut toks = Tokenizer::new(text.as_bytes());
    loop {
        let tok = toks.next();
        println!("tok: {:?}", tok);
        if tok.ty == Type::End {
            break;
        }
    }
}
