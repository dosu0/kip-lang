use clap::Parser;

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Options {
    /// The input source file, stdin if not specified
    pub input: Option<PathBuf>,
    /// Where to output the intermediate; stdout if not specified
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}
