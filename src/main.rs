use kip::cli::Options;
use kip::{driver, logger};
use log::info;
use std::env;
use std::error::Error;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init()?;
    info!(target: "compiler_info", "kip version {version} (kip v{version})", version = env!("CARGO_PKG_VERSION"));

    let options = Options::from_args();

    driver::run(options)?;

    Ok(())
}
