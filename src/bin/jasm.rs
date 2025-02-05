use std::{
    env, fs,
    io::{self, Read},
};

use jcore::assembler::{lexer::Lexer, symbols::SymbolTable};

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap();
    let args = args.collect::<Vec<_>>();

    if args.is_empty() {
        eprintln!("USAGE: {program} - (stdin) | <filename>");
    }

    let filename = &args[0];
    let mut buffer = String::new();

    if filename == "-" {
        let mut stdin = io::stdin();
        stdin.read_to_string(&mut buffer).unwrap();
    } else {
        let mut file = fs::File::open(filename).unwrap();
        file.read_to_string(&mut buffer).unwrap();
    }

    let mut symbols = SymbolTable::default();
    let tokens = Lexer::new(&buffer, &mut symbols).collect::<Vec<_>>();

    println!("{tokens:?}")
}
