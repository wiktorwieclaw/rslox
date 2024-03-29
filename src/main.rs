use std::{
    cmp::Ordering,
    env, fs,
    io::{self, BufRead, Write},
    process,
};

use lox::{interpret, lex, parse};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: jlox [script]");
            process::exit(64);
        }
        Ordering::Equal => run_file(&args[1]).unwrap(),
        Ordering::Less => run_prompt().unwrap(),
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let code = fs::read_to_string(path)?;
    run(&code);
    if lox::error::HAD_ERROR.with(|e| e.get()) {
        process::exit(65);
    }
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let stdout = io::stdin();
    let mut lines = stdout.lock().lines();
    loop {
        print!("> ");
        io::stdout().flush()?;
        match lines.next() {
            Some(line) => {
                run(&line?);
                lox::error::HAD_ERROR.with(|e| e.set(false))
            }
            None => break,
        }
    }
    Ok(())
}

fn run(code: &str) {
    let scanner = lex::Scanner::new(code);
    let mut parser = parse::Parser::new(scanner);
    let ast = parser.parse();
    if lox::error::HAD_ERROR.with(|e| e.get()) {
        return;
    }
    println!("{:?}", interpret::interpret(ast.unwrap()));
}
