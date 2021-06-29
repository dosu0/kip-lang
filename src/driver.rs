//! Kip driver

// TODO: add broad compilation
use std::{
    fs::File,
    io::{self, Read, Write},
};

use atty::Stream;

use crate::lexer::TokenStream;
use crate::parser::Parser;

pub fn main_loop() -> io::Result<()> {
    if atty::is(Stream::Stdin) {
        // if this is a human, be interactive
        repl()
    } else {
        // otherwise, don't
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let tokens = TokenStream::new(&input);
        let mut parser = Parser::new(tokens);
        parser.parse();
        Ok(())
    }
}

fn repl() -> io::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        eprint!("[kip:ready]> ");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Some(cmd) = input.trim().strip_prefix('.') {
            let mut args = cmd.split_whitespace();

            if let Some(cmd) = args.next() {
                match cmd {
                    "exit" | "quit" => break,
                    "save" => {
                        if let Some(path) = args.next() {
                            let mut f = File::create(path)?;
                            f.write_all(&buf)?;
                        } else {
                            eprintln!(
                                "[kip::driver] error: the `save` command requires a file argument"
                            );
                        }
                    }
                    "help" => {
                        eprintln!("This is the kip repl, commands start with a `.`");
                        eprintln!(".exit => leave the repl");
                        eprintln!(".quit => leave the repl");
                        eprintln!(".save <file> => save all code into a file");
                        eprintln!(".help => display this help message");
                    }
                    _ => eprintln!("[kip::driver] error: unknown repl command `{}`", input),
                }
            }
        } else {
            let tokens = TokenStream::new(&input);
            let mut parser = Parser::new(tokens);
            parser.parse();
            buf.write(input.as_bytes())?;
        }
    }

    Ok(())
}
