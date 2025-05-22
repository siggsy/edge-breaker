#![allow(non_snake_case)]

mod common;
mod compression;
mod decompression;

use crate::obj::Obj;
use common::{Id, Op};
use compression::{HalfEdges, compress};
use decompression::decompress;
use log::debug;
use std::fmt::Debug;

// .--------------------------------------------------------------------------.
// | Struct: EdgeBreaker                                                      |
// '--------------------------------------------------------------------------'

#[derive(Debug)]
pub struct EdgeBreaker {
    history: Vec<Op>,
    previous: Vec<Id>,
    lengths: Vec<usize>,
    m_table: Vec<(usize, usize, usize)>,
}

// .--------------------------------------------------------------------------.
// | Public functions                                                         |
// '--------------------------------------------------------------------------'

pub fn compress_obj(obj: &Obj) -> EdgeBreaker {
    let mut he = HalfEdges::init(obj);
    let eb = compress(&mut he);
    debug!("History: {:?}", eb.history);
    debug!("Previous: {:?}", eb.previous);
    debug!("Lengths: {:?}", eb.lengths);
    eb
}

pub fn decompress_obj(eb: &EdgeBreaker, vertices: Vec<[f32; 3]>) -> Obj {
    let faces = decompress(eb);
    debug!("Faces: {:?}", faces);
    debug!("Faces len: {:?}", faces.len());

    Obj { faces, vertices }
}
