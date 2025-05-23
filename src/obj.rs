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
    pub fn read(reader: &mut BufReader<File>) -> Self {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        #[inline(always)]
        fn parse_index(word: &str) -> usize {
            word.split('/').nth(0).unwrap().parse().unwrap()
        }

        for (i, line) in reader.lines().map_while(Result::ok).enumerate() {
            let mut chars = line.chars();
            let c = chars.next();

            match c {
                Some('v') => match chars.next() {
                    Some(' ') => {
                        let vals = line
                            .split(' ')
                            .skip(1)
                            .filter(|x| !x.is_empty())
                            .map(|w| w.parse().unwrap())
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap();
                        vertices.push(vals);
                    }
                    _ => continue,
                },
                Some('f') => {
                    let vals = line
                        .split(' ')
                        .skip(1)
                        .filter(|x| !x.is_empty())
                        .map(parse_index)
                        .collect::<Vec<_>>();

                    let n = vals.len();
                    if n == 3 {
                        faces.push(vals.try_into().unwrap());
                        continue;
                    }

                    for i in 1..=n - 2 {
                        faces.push([vals[0], vals[i], vals[i + 1]]);
                    }
                }
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
