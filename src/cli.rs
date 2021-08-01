use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    /// the input source file, stdin if not specified
    pub input: Option<PathBuf>,
    /// where to output the intermediate; stdout if not specified
    #[structopt(short)]
    pub output: Option<PathBuf>,
}

