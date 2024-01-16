use kip::cli::Options;
use kip::driver;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let options = Options::parse();

    driver::run(options)?;

    Ok(())
}
