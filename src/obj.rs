use colored::Colorize;
use log::warn;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, LineWriter, Write},
};

#[derive(Debug)]
pub struct Obj {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
}

impl Obj {
    #[inline(always)]
    fn parse_index(word: &str) -> usize {
        word.split('/').nth(0).unwrap().parse().unwrap()
    }

    #[inline(always)]
    fn values<T: Debug, const N: usize>(line: &str, conv: fn(&str) -> T) -> [T; N] {
        line.split(' ')
            .skip(1)
            .filter(|x| !x.is_empty())
            .take(N)
            .map(conv)
            .collect::<Vec<T>>()
            .try_into()
            .unwrap()
    }

    pub fn read(reader: &mut BufReader<File>) -> Self {
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
                _ => warn!("Failed to parse line {i}: {line}"),
            }
        }

        Obj { vertices, faces }
    }

    pub fn write(&self, file: &File) {
        let mut writer = LineWriter::new(file);
        for v in &self.vertices {
            let _ = writer.write_all(&format!("v {} {} {}\n", v[0], v[1], v[2]).into_bytes());
        }
        for f in &self.faces {
            let _ = writer.write_all(&format!("f {} {} {}\n", f[0], f[1], f[2]).into_bytes());
        }
    }
}
