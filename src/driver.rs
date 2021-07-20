//! Kip driver

use std::env::Args;
use std::error::Error;
use std::fs;
use std::io::{self, Read};

use atty::Stream;
use log::error;

use crate::lexer::TokenStream;
use crate::parser::Parser;
use crate::source::Source;

pub fn run(mut args: Args) -> Result<(), Box<dyn Error>> {
    let mut source = Source {
        name: String::new(),
        contents: String::new(),
    };

    if let Some(source_file_path) = args.next() {
        // user provided a path to a source file
        source.contents = fs::read_to_string(&source_file_path)?;
        source.name = source_file_path;
    } else if atty::isnt(Stream::Stdin) {
        // the user (presumably) passed a source file through stdin
        source.name = String::from("<stdin>");
        io::stdin().read_to_string(&mut source.contents)?;
    } else {
        error!(target: "driver", "no input file");
    }

    let tokens = TokenStream::new(&source);
    let mut parser = Parser::new(tokens);
    parser.parse();
    Ok(())
}
