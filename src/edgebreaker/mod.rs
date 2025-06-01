#![allow(non_snake_case)]

mod common;
mod compression;
mod decompression;
pub mod public;

use crate::obj::{Obj, Table};
use common::{EdgeBreaker, Id, NULL};
use compression::{HalfEdges, compress};
use decompression::decompress;
use log::debug;
use public::Op;

fn table_scount(entry: Table) -> isize {
    match entry {
        Table::Hole(_s, _) => _s as isize,
        Table::Merge(_s, _, _, _) => _s as isize,
    }
}

// ,---------------------------------------------------------------------------
// | Public functions
// '---------------------------------------------------------------------------

pub fn compress_obj(obj: &mut Obj) {
    let mut he = HalfEdges::init(obj);
    let eb = compress(&mut he);
    debug!("eb: {:?}", eb);
    debug!("History: {:?}", eb.history);
    debug!("Previous: {:?}", eb.previous);
    debug!("Lengths: {:?}", eb.lengths);

    let mut perm_vertices = Vec::with_capacity(obj.vertices.len());
    let mut dup = Vec::new();
    let mut inserted = vec![NULL; obj.vertices.len()];
    let mut c = 0;
    for p in eb.previous {
        if inserted[p] == NULL {
            inserted[p] = Id::from_offset(c);
            perm_vertices.push(obj.vertices[p]);
        } else {
            dup.push((c, inserted[p].offset()));
        }
        c += 1;
    }

    // eb_table is more involved
    let mut s_count = 0;
    let mut h = 0;
    let mut m = 0;
    let mut eb_table = Vec::new();

    for op in &eb.history {
        match op {
            Op::S => s_count += 1,
            Op::H => {
                let l = eb.lengths[h];
                h += 1;
                eb_table.push(Table::Hole(s_count, l));
                s_count = 0;
            }
            Op::M => {
                let (i1, i2, i3) = eb.m_table[m];
                m += 1;
                eb_table.push(Table::Merge(s_count, i1, i2, i3));
                s_count = 0;
            }
            _ => {} // Do nothing
        }
    }

    obj.vertices = perm_vertices;
    obj.faces = Vec::new();
    obj.eb_history = eb.history;
    obj.eb_table = eb_table;
    obj.eb_dup = dup;
}

pub fn decompress_obj(obj: &mut Obj) {
    let mut history = Vec::with_capacity(obj.eb_history.len());
    let mut lengths = Vec::new();
    let mut m_table = Vec::new();

    let mut t = 0;
    let mut s = obj.eb_table.first().map_or(-1, |&x| table_scount(x));

    for op in &obj.eb_history {
        match op {
            Op::S | Op::M | Op::H => {
                if s == 0 {
                    match obj.eb_table[t] {
                        Table::Hole(_, l) => {
                            history.push(Op::H);
                            lengths.push(l);
                        }
                        Table::Merge(_, i1, i2, i3) => {
                            history.push(Op::M);
                            m_table.push((i1, i2, i3));
                        }
                    }
                    t += 1;
                    s = obj.eb_table.get(t).map_or(-1, |&x| table_scount(x));
                } else {
                    history.push(*op);
                    s -= 1;
                }
            }
            _ => {
                history.push(*op);
            }
        }
    }

    let mut previous = Vec::new();
    let mut i = 0;
    for (pos, idx) in &obj.eb_dup {
        while previous.len() < *pos {
            previous.push(Id::from_offset(i));
            i += 1;
        }
        previous.push(Id::from_offset(*idx));
    }

    for _ in 0..obj.vertices.len() - (previous.len() - obj.eb_dup.len()) {
        previous.push(Id::from_offset(i));
        i += 1;
    }

    let eb = EdgeBreaker {
        history: history,
        previous: previous,
        lengths: lengths,
        m_table: m_table,
    };
    debug!("eb: {:?}", eb);
    let faces = decompress(&eb);
    debug!("Faces: {:?}", faces);
    debug!("Faces len: {:?}", faces.len());
    obj.eb_history = Vec::new();
    obj.eb_table = Vec::new();
    obj.eb_dup = Vec::new();
    obj.faces = faces;
}
