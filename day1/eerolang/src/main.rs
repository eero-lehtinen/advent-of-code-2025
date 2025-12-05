use log::error;

use crate::program::Program;

mod ast_parser;
mod program;
mod tokenizer;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        error!("Usage: {} <source_file>", args[0]);
        return;
    }

    let source_file = &args[1];
    let source_code = std::fs::read_to_string(source_file).expect("Failed to read source file");

    let tokens = tokenizer::tokenize(&source_code);
    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let block = ast_parser::parse(&tokens);
    let mut program = Program::new(block);
    program.execute();
}
