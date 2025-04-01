use std::{env::args, fs::File, io::BufReader};

use edgebreaker::EdgeBreaker;
use log::{info, LevelFilter};
use obj::Obj;
use debug::Logger;

mod obj;
mod debug;
mod edgebreaker;

static LOGGER: Logger = Logger;
static LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() -> std::io::Result<()> {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LOG_LEVEL));
    let mut reader = BufReader::new(File::open(args().nth(1).unwrap_or(String::from("./assets/cube.obj")))?);
    let obj = Obj::read(&mut reader);
    let eb = EdgeBreaker::compress(&obj);
    Ok(())
}
