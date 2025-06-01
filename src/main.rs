use std::{
    env::{Args, args},
    fs::File,
    io::{self, BufRead, BufReader, LineWriter, Write},
    process::exit,
};

use colored::Colorize;
use debug::Logger;
use log::{LevelFilter, error};
use obj::Obj;

mod debug;
mod edgebreaker;
mod obj;

static LOGGER: Logger = Logger;

enum Operation {
    Compress,
    Decompress,
}

struct CLI {
    verbose: bool,
    input: Option<String>,
    output: Option<String>,
    operation: Option<Operation>,
}

impl CLI {
    fn open_input(&self) -> Box<dyn BufRead> {
        match &self.input {
            Some(path) => Box::new(BufReader::new(File::open(path).unwrap_or_else(|_| {
                error!("Input file does not exist");
                exit(1);
            }))),
            None => Box::new(BufReader::new(io::stdin())),
        }
    }

    fn open_output(&self) -> Box<dyn Write> {
        match &self.output {
            Some(path) => Box::new(LineWriter::new(File::create(path).unwrap_or_else(|_| {
                error!("Output file does not exist");
                exit(1);
            }))),
            None => Box::new(LineWriter::new(io::stdout())),
        }
    }
}

fn print_help() {
    eprintln!(
        "usage: {} <{}> [{}]",
        args().nth(0).unwrap().yellow(),
        "OPERATIONS".green(),
        "FLAGS".blue()
    );
    eprintln!();
    eprintln!("{}:", "OPERATIONS".green());
    eprintln!("  compress       Compress input and write it to output");
    eprintln!("  decompress     Decompress input and write it to output");
    eprintln!();
    eprintln!("{}:", "FLAGS".blue());
    eprintln!("  -i <file>      Input file. Defaults to stdin");
    eprintln!("  -o <file>      Output file. Defaults to stdout");
    eprintln!("  -v             Increase verbosity");
    eprintln!();
}

fn parse_args(args: &mut Args) -> CLI {
    let mut cli = CLI {
        verbose: false,
        input: None,
        output: None,
        operation: None,
    };

    while let Some(arg) = args.next() {
        let mut arg_chars = arg.chars();
        match arg_chars.next() {
            Some('-') => {
                while let Some(ch) = arg_chars.next() {
                    match ch {
                        'v' => cli.verbose = true,
                        'i' => {
                            if let Some(path) = args.next() {
                                cli.input = Some(path);
                            } else {
                                error!("-i: missing file path")
                            }
                        }
                        'o' => {
                            if let Some(path) = args.next() {
                                cli.output = Some(path);
                            } else {
                                error!("-o: missing file path")
                            }
                        }
                        _ => error!("Unknown flag '{}'", ch),
                    }
                }
            }

            Some('c') => {
                if "compression".starts_with(&arg) {
                    cli.operation = Some(Operation::Compress)
                } else {
                    error!("Unknown operation '{}'", arg);
                }
            }

            Some('d') => {
                if "decompression".starts_with(&arg) {
                    cli.operation = Some(Operation::Decompress)
                } else {
                    error!("Unknown operation '{}'", arg);
                }
            }

            _ => error!("Failed to parse argument '{}'", arg),
        }
    }

    cli
}

fn main() -> std::io::Result<()> {
    let cli = parse_args(&mut args());

    let _ = log::set_logger(&LOGGER).map(|()| {
        log::set_max_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
    });

    match cli.operation {
        Some(Operation::Compress) => {
            let mut obj = Obj::read(&mut cli.open_input());
            edgebreaker::compress_obj(&mut obj);
            obj.write(&mut cli.open_output());
        }
        Some(Operation::Decompress) => {
            let mut obj = Obj::read(&mut cli.open_input());
            edgebreaker::decompress_obj(&mut obj);
            obj.write(&mut cli.open_output());
        }
        None => print_help(),
    };

    Ok(())
}
