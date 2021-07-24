use kip::{driver, logger};
use log::info;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    logger::init()?;
    info!(target: "compiler_info", "kip version {version} (kip v{version})", version = env!("CARGO_PKG_VERSION"));

    let mut args = env::args();
    // skip program name
    args.next();

    driver::run(args)?;

    Ok(())
}
