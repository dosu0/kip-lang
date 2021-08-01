//! Kip driver

use std::error::Error;
use std::fs;
use std::io::{self, Read};

use atty::Stream;
use log::error;

use crate::cli;
use crate::lexer::TokenStream;
use crate::parser::Parser;
use crate::source::Source;

pub fn run(options: cli::Options) -> Result<(), Box<dyn Error>> {
    let source = if let Some(source_file_path) = options.input {
        // user provided a path to a source file
        Source {
            contents: fs::read_to_string(&source_file_path)?,
            name: source_file_path.to_string_lossy().into_owned(),
        }
    } else if atty::isnt(Stream::Stdin) {
        // the user (presumably) passed a source file through stdin
        let mut source = Source {
            name: String::from("<stdin>"),
            contents: String::new(),
        };
        io::stdin().read_to_string(&mut source.contents)?;
        source
    } else {
        error!(target: "driver", "no input file");
        return Ok(());
    };

    let tokens = TokenStream::new(&source);
    let mut parser = Parser::new(tokens);
    parser.parse();
    Ok(())
}
