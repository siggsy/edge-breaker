#![allow(non_snake_case)]

mod edge;
mod id;
mod op;

use crate::obj::Obj;
use edge::Edge;
use id::Id;
use id::NULL;
use log::debug;
use op::Op;
use std::collections::HashMap;
use std::fmt::Debug;

// ,---------------------------------------------------------------------------
// | Structs
// `---------------------------------------------------------------------------

#[derive(Debug)]
pub struct EdgeBreaker {
    gate: Id,
    history: Vec<Op>,
    previous: Vec<Id>,
}

#[derive(Debug)]
struct HalfEdges {
    vertex_count: usize,
    triangle_count: usize,
    s: Vec<Id>,
    e: Vec<Id>,
    n: Vec<Id>,
    o: Vec<Id>,
    p: Vec<Id>,
}

impl HalfEdges {
    fn init(obj: &Obj) -> Self {
        let capacity = obj.faces.len() * 3;
        let mut s: Vec<Id> = vec![NULL; capacity];
        let mut e: Vec<Id> = vec![NULL; capacity];
        let mut n: Vec<Id> = vec![NULL; capacity];
        let mut p: Vec<Id> = vec![NULL; capacity];
        let mut o: Vec<Id> = vec![NULL; capacity];

        let mut edge_map: HashMap<Edge, Id> = HashMap::new();
        for (t, face) in obj.faces.iter().enumerate() {
            let offset = t * 3;

            // Construct half-edges from triangle
            for i in 0..3 {
                let h = Id::from_offset(i + offset);

                s[h] = Id::new(face[i]);
                e[h] = Id::new(face[(i + 1) % 3]);
                n[h] = Id::from_offset((i + 1) % 3 + offset);
                p[h] = Id::from_offset((i + 2) % 3 + offset);
            }

            // Check for collisions and fix boundary
            for i in 0..3 {
                let h = Id::from_offset(i + offset);
                let edge = Edge::new(face[i], face[(i + 1) % 3]);

                if let Some(g) = edge_map.insert(edge, h) {
                    // Fix next and previous for triangles
                    let gN = n[g];
                    let gP = p[g];
                    let hN = n[h];
                    let hP = p[h];

                    // TODO handle non-orientable and non-manifold meshes
                    if gN == NULL || gP == NULL || hN == NULL || hP == NULL {
                        panic!("Surface is not a 2-manifold");
                    }

                    n[hP] = gN;
                    p[gN] = hP;
                    n[gP] = hN;
                    p[hN] = gP;

                    // Reset next and previous half edges
                    n[g] = NULL;
                    p[g] = NULL;
                    n[h] = NULL;
                    p[h] = NULL;

                    o[h] = g;
                    o[g] = h;
                }
            }
        }

        Self {
            vertex_count: obj.vertices.len(),
            triangle_count: obj.faces.len(),
            s,
            e,
            n,
            o,
            p,
        }
    }

    fn v(&self, id: Id) -> Id {
        self.e[Self::n(id)]
    }

    fn n(id: Id) -> Id {
        assert!(id != NULL);
        let offset = id.offset();
        let i = offset % 3;
        let t = offset - i;
        Id::from_offset((i + 1) % 3 + t)
    }

    fn p(id: Id) -> Id {
        assert!(id != NULL);
        let offset = id.offset();
        let i = offset % 3;
        let t = offset - i;
        Id::from_offset((i + 2) % 3 + t)
    }
}

// ,---------------------------------------------------------------------------
// | Public functions
// `---------------------------------------------------------------------------

pub fn compress_obj(obj: &Obj) -> EdgeBreaker {
    let mut he = HalfEdges::init(obj);
    debug!("faces.len: {:?}", obj.faces.len());
    debug!("vertices.len: {:?}", obj.vertices.len());
    let eb = compress(&mut he);
    debug!("gate: {:?}", eb.gate);
    debug!("history: {:?}", eb.history);
    debug!("history.len: {:?}", eb.history.len());
    debug!("previous: {:?}", eb.previous);
    debug!("previous.len: {:?}", eb.previous.len());
    eb
}

// ,---------------------------------------------------------------------------
// | Internal functions
// `---------------------------------------------------------------------------

fn compress(he: &mut HalfEdges) -> EdgeBreaker {
    let mut boundary = Vec::new();
    let mut history = Vec::new();
    let mut previous = Vec::new();
    let mut stack = Vec::new();

    let mut vm = vec![false; he.vertex_count];
    let mut hm = vec![false; he.triangle_count * 3];

    let gate = match he.n.iter().find(|&x| *x != NULL) {
        Some(&gate) => gate,
        None => Id::new(1),
    };

    // Find boundary
    let mut g = gate;
    while g != NULL && he.e[g] != he.s[gate] {
        let ev = he.e[g];
        boundary.push(ev);
        vm[ev] = true;
        hm[g] = true;
        g = he.n[g];
    }
    boundary.push(he.s[gate]);
    vm[he.s[gate]] = true;

    if he.n[gate] == NULL {
        he.n[gate] = he.o[gate];
        he.p[gate] = he.o[gate];
        he.n[he.o[gate]] = gate;
        he.p[he.o[gate]] = gate;
        hm[he.o[gate]] = true;
    }

    // Main algorithm loop
    stack.push(gate);
    while let Some(g) = stack.pop() {
        if !vm[he.v(g)] {
            // Case C
            history.push(Op::C);
            previous.push(he.v(g));

            let gpo = he.o[HalfEdges::p(g)];
            let gno = he.o[HalfEdges::n(g)];
            let gv = he.v(g);
            let gN = he.n[g];
            let gP = he.p[g];

            // Fix flags
            hm[g] = false;
            hm[gpo] = true;
            hm[gno] = true;
            vm[gv] = true;

            // Link 1
            he.p[gpo] = he.p[g];
            he.n[gP] = gpo;

            // Link 2
            he.n[gpo] = gno;
            he.p[gno] = gpo;

            // Link 3
            he.n[gno] = gN;
            he.p[gN] = gno;

            stack.push(gno);
        } else {
            if HalfEdges::p(g) == he.p[g] {
                if HalfEdges::n(g) == he.n[g] {
                    // Case E
                    history.push(Op::E);

                    let gn = HalfEdges::n(g);
                    let gp = HalfEdges::p(g);
                    hm[g] = false;
                    hm[gn] = false;
                    hm[gp] = false;
                } else {
                    // Case L
                    history.push(Op::L);

                    let gP = he.p[g];
                    let gPP = he.p[gP];
                    let gno = he.o[HalfEdges::n(g)];
                    let gN = he.n[g];

                    // Flags
                    hm[g] = false;
                    hm[gP] = false;
                    hm[gno] = true;

                    // Link 1
                    he.n[gPP] = gno;
                    he.p[gno] = gPP;

                    // Link 2
                    he.n[gno] = gN;
                    he.p[gN] = gno;

                    stack.push(gno);
                }
            } else {
                if HalfEdges::n(g) == he.n[g] {
                    // Case R
                    history.push(Op::R);

                    let gN = he.n[g];
                    let gNN = he.n[gN];
                    let gpo = he.o[HalfEdges::p(g)];
                    let gP = he.p[g];

                    // Flags
                    hm[g] = false;
                    hm[gN] = false;
                    hm[gpo] = true;

                    // Link 1
                    he.p[gNN] = gpo;
                    he.n[gpo] = gNN;

                    // Link 2
                    he.p[gpo] = gP;
                    he.n[gP] = gpo;

                    stack.push(gpo);
                } else {
                    // Case S
                    history.push(Op::S);

                    let gno = he.o[HalfEdges::n(g)];
                    let gpo = he.o[HalfEdges::p(g)];
                    let gN = he.n[g];
                    let gP = he.p[g];

                    // Flags
                    hm[g] = false;
                    hm[gpo] = true;
                    hm[gno] = true;

                    // Find b by rotating around v
                    let mut b = HalfEdges::n(g);
                    while !hm[b] {
                        b = HalfEdges::p(he.o[b]);
                    }

                    // Link 1
                    he.n[gP] = gpo;
                    he.p[gpo] = gP;

                    // Link 2
                    let bN = he.n[b];
                    he.n[gpo] = bN;
                    he.p[bN] = gpo;

                    // Link 3
                    he.n[b] = gno;
                    he.p[gno] = b;

                    // Link 4
                    he.n[gno] = gN;
                    he.p[gN] = gno;

                    stack.push(gpo);
                    stack.push(gno);
                }
            }
        }
    }

    EdgeBreaker {
        gate,
        history,
        previous,
    }
}
