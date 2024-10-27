mod logger;

use eyre::Result;

fn main() -> Result<()> {
    logger::init();

    Ok(())
}
