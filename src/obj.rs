use colored::Colorize;
use log::warn;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
pub struct Obj {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
}

impl Obj {
    #[inline(always)]
    fn parse_index(word: &str) -> u32 {
        word.split('/').nth(0).unwrap().parse().unwrap()
    }

    #[inline(always)]
    fn values<T: Debug, const N: usize>(line: &str, conv: fn(&str) -> T) -> [T; N] {
        line.split(' ')
            .skip(1)
            .take(N)
            .map(conv)
            .collect::<Vec<T>>()
            .try_into()
            .unwrap()
    }

    pub fn read(reader: &mut BufReader<File>) -> Obj {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (i, line) in reader.lines().map_while(Result::ok).enumerate() {
            let mut chars = line.chars();
            let c = chars.next();

            match c {
                Some('v') => match chars.next() {
                    Some(' ') => vertices.push(Obj::values(&line, |w| w.parse().unwrap())),
                    _ => continue,
                },
                Some('f') => faces.push(Obj::values(&line, Obj::parse_index)),
                Some('#') => continue,
                _ => warn!("Failed to parse line: {line}"),
            }
        }

        Obj { vertices, faces }
    }
}
