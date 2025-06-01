use log::warn;
use std::{
    fmt::Debug,
    io::{BufRead, Write},
};

use crate::edgebreaker::public::Op;

#[derive(Debug, Clone, Copy)]
pub enum Table {
    Hole(usize, usize),
    Merge(usize, usize, usize, usize),
}

#[derive(Debug)]
pub struct Obj {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
    pub eb_history: Vec<Op>,
    pub eb_table: Vec<Table>,
    pub eb_dup: Vec<(usize, usize)>,
}

impl Obj {
    pub fn read<T: BufRead>(reader: &mut T) -> Self {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut history = Vec::new();
        let mut table = Vec::new();
        let mut dup = Vec::new();

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

                Some('e') => match line.split(' ').nth(0).unwrap() {
                    "ebh" => {
                        let [base64, pad_char] = line.split(' ').skip(1).collect::<Vec<_>>()[..]
                        else {
                            warn!("Failed decoding base64 at line {i}");
                            continue;
                        };
                        history.extend(Op::decode_history(base64, pad_char.parse().unwrap()));
                    }
                    "ebt" => {
                        let entries = line
                            .split(' ')
                            .skip(1)
                            .filter(|x| !x.is_empty())
                            .collect::<Vec<_>>();

                        for entry in entries {
                            let vals = entry.split('/').collect::<Vec<_>>();
                            if vals.len() == 2 {
                                table.push(Table::Hole(
                                    vals[0].parse().unwrap(),
                                    vals[1].parse().unwrap(),
                                ));
                            } else {
                                table.push(Table::Merge(
                                    vals[0].parse().unwrap(),
                                    vals[1].parse().unwrap(),
                                    vals[2].parse().unwrap(),
                                    vals[3].parse().unwrap(),
                                ));
                            }
                        }
                    }
                    "ebd" => {
                        let entries = line
                            .split(' ')
                            .skip(1)
                            .filter(|x| !x.is_empty())
                            .collect::<Vec<_>>();

                        for entry in entries {
                            let [pos_word, idx_word] = entry.split('/').collect::<Vec<_>>()[..]
                            else {
                                warn!(
                                    "Failed to parse edge breaker duplicated at line {i}: {line}"
                                );
                                continue;
                            };

                            let pos = pos_word.parse().unwrap();
                            let idx = idx_word.parse().unwrap();
                            dup.push((pos, idx));
                        }
                    }
                    _ => warn!("Failed to parse line {i}: {line}"),
                },

                Some('#') => continue,
                _ => warn!("Failed to parse line {i}: {line}"),
            }
        }

        Obj {
            vertices,
            faces,
            eb_history: history,
            eb_table: table,
            eb_dup: dup,
        }
    }

    pub fn write<T: Write>(&self, writer: &mut T) {
        for v in &self.vertices {
            let _ = write!(writer, "v {} {} {}\n", v[0], v[1], v[2]);
        }
        for f in &self.faces {
            let _ = write!(writer, "f {} {} {}\n", f[0], f[1], f[2]);
        }

        if !self.eb_history.is_empty() {
            let (base64, pad) = Op::encode_history(&self.eb_history);
            let _ = write!(writer, "ebh {} {}\n", base64, pad);
        }

        if !self.eb_table.is_empty() {
            let _ = writer.write(b"ebt");
            for entry in &self.eb_table {
                let _ = match entry {
                    Table::Hole(i1, i2) => write!(writer, " {i1}/{i2}"),
                    Table::Merge(i1, i2, i3, i4) => write!(writer, " {i1}/{i2}/{i3}/{i4}"),
                };
            }
            let _ = writer.write(b"\n");
        }

        if !self.eb_dup.is_empty() {
            let _ = writer.write(b"ebd");
            for (pos, idx) in &self.eb_dup {
                let _ = write!(writer, " {pos}/{idx}");
            }
            let _ = writer.write(b"\n");
        }
    }
}
