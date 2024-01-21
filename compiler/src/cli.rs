use clap::Parser;

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Options {
    /// The input source file, stdin if not specified
    pub input: Option<PathBuf>,
    /// Where to output the intermediate code
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    // whether or not to optimize the code
    #[clap(short = 'O', long, default_value_t = false)]
    pub optimize: bool,
}
