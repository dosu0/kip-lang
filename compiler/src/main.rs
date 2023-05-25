use kip::driver;
use kip::cli::Options;

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let options = Options::parse();

    driver::run(options)?;

    Ok(())
}
