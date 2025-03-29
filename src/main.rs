use std::{env::args, fs::File, io::BufReader};

use log::{info, LevelFilter};
use obj::Obj;
use debug::Logger;

mod obj;
mod debug;

static LOGGER: Logger = Logger;
static LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() -> std::io::Result<()> {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LOG_LEVEL));
    let mut reader = BufReader::new(File::open(args().nth(1).unwrap())?);
    info!("Parsed indices: {:?}", Obj::read(&mut reader).faces);
    Ok(())
}
