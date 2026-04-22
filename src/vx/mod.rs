mod lexer;
mod ast;
mod parser;
mod interpreter;

pub use lexer::{Token, tokenize};
pub use parser::Parser;
pub use interpreter::Interpreter;

use crate::scene::Scene;

pub fn execute_script(source: &str, scene: &mut Scene) -> Result<(), String> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program, scene)?;
    Ok(())
}
