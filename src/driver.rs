use std::{
    fs::File,
    io::{self, Read, Write},
};

use atty::Stream;

use crate::{
    lexer::{Token, TokenStream},
    parser::{handle_extern, handle_func, handle_top_lvl_expr},
};

pub fn main_loop() -> io::Result<()> {
    if atty::is(Stream::Stdin) {
        // if this is a human, be interactive
        repl()
    } else {
        // otherwise, don't
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let mut tokens = TokenStream::new(&input);

        while handle_top(&mut tokens) {}

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
            let mut args = cmd.split_ascii_whitespace();

            if let Some(cmd) = args.next() {
                match cmd {
                    "exit" | "quit" => return Ok(()),
                    "save" => {
                        if let Some(path) = args.next() {
                            let mut f = File::create(path)?;
                            f.write_all(&buf)?;
                        } else {
                            eprintln!("[kip::driver] error: the `save` command requires a file argument");
                        }
                    }
                    _ => eprintln!(
                        "[kip::driver] error: unknown repl command `{}`",
                        input
                    ),
                }
            }
        } else {
            let mut tokens = TokenStream::new(&input);

            if !handle_top(&mut tokens) {
                break;
            }

            for b in input.as_bytes() {
                buf.push(*b);
            }
        }
    }

    Ok(())
}

fn handle_top(tokens: &mut TokenStream) -> bool {
    match tokens.peek() {
        Token::Eof => return false,
        Token::Semicolon => {
            tokens.eat();
        }
        Token::Func => {
            tokens.eat();
            handle_func(tokens);
        }
        Token::Extern => {
            tokens.eat();
            handle_extern(tokens);
        }
        _ => handle_top_lvl_expr(tokens),
    }

    true
}
