use std::{env::args, fs::File, io::BufReader};

use debug::Logger;
use log::LevelFilter;
use obj::Obj;

mod debug;
mod edgebreaker;
mod obj;

static LOGGER: Logger = Logger;
static LOG_LEVEL: LevelFilter = LevelFilter::Debug;

fn main() -> std::io::Result<()> {
    let fallback = String::from("./assets/cube.obj");

    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Error));
    let mut reader = BufReader::new(File::open(args().nth(1).unwrap_or(fallback))?);
    let obj = Obj::read(&mut reader);
    let eb = edgebreaker::compress_obj(&obj);
    let _obj = edgebreaker::decompress_obj(&eb, obj.vertices);
    let f = File::create("out/compress-decompress.obj")
        .expect("Failed creating file compress-decompress.obj");
    _obj.write(&f);
    Ok(())
}
