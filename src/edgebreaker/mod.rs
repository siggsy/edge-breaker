#![allow(non_snake_case)]

mod id;
mod op;

use crate::obj::Obj;
use id::Id;
use id::NULL;
use log::debug;
use op::Op;
use std::collections::HashMap;
use std::fmt::Debug;

// .--------------------------------------------------------------------------.
// | Struct: EdgeBreaker                                                      |
// '--------------------------------------------------------------------------'

#[derive(Debug)]
pub struct EdgeBreaker {
    history: Vec<Op>,
    previous: Vec<Id>,
    duplicated: Vec<Id>,
    lengths: Vec<usize>,
}

// .--------------------------------------------------------------------------.
// | Struct: HalfEdges                                                        |
// '--------------------------------------------------------------------------'

#[derive(Debug)]
struct HalfEdges {
    vertex_count: usize,
    triangle_count: usize,
    duplicated: Vec<Id>,
    s: Vec<Id>,
    e: Vec<Id>,
    n: Vec<Id>,
    o: Vec<Id>,
    p: Vec<Id>,
}

impl HalfEdges {
    fn init(obj: &Obj) -> Self {
        let capacity = obj.faces.len() * 3;
        let vertex_count = obj.vertices.len();
        let mut dup_vertices = Vec::new();
        let mut s: Vec<Id> = vec![NULL; capacity];
        let mut e: Vec<Id> = vec![NULL; capacity];
        let mut n: Vec<Id> = vec![NULL; capacity];
        let mut p: Vec<Id> = vec![NULL; capacity];
        let mut o: Vec<Id> = vec![NULL; capacity];

        let mut edge_map: HashMap<(usize, usize), Id> = HashMap::new();
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
                let a = face[i];
                let b = face[(i + 1) % 3];

                if let Some(&g) = edge_map.get(&(b, a)) {
                    // Fix next and previous for triangles
                    let gN = n[g];
                    let gP = p[g];
                    let hN = n[h];
                    let hP = p[h];

                    if gN == NULL || gP == NULL || hN == NULL || hP == NULL {
                        // non-surface edge: colided edge is already internal
                        // Detach by duplicating vertices
                        dup_vertices.push(s[h]);
                        s[h] = Id::new(dup_vertices.len() + vertex_count);
                        dup_vertices.push(e[h]);
                        e[h] = Id::new(dup_vertices.len() + vertex_count);
                        edge_map.insert((s[h].offset(), e[h].offset()), h);
                    } else {
                        // First collision: make half edges internal

                        // Connect border loops
                        n[hP] = gN;
                        p[gN] = hP;
                        n[gP] = hN;
                        p[hN] = gP;

                        // Remove border loop for colided half edges
                        n[g] = NULL;
                        p[g] = NULL;
                        n[h] = NULL;
                        p[h] = NULL;

                        // h and g are opposites
                        o[h] = g;
                        o[g] = h;
                    }
                } else if let Some(_) = edge_map.get(&(a, b)) {
                    // non-orientable edge: duplicate vertices
                    dup_vertices.push(s[h]);
                    s[h] = Id::new(dup_vertices.len() + vertex_count);
                    dup_vertices.push(e[h]);
                    e[h] = Id::new(dup_vertices.len() + vertex_count);
                    edge_map.insert((s[h].offset(), e[h].offset()), h);
                } else {
                    edge_map.insert((a, b), h);
                }
            }
        }

        Self {
            vertex_count: vertex_count + dup_vertices.len(),
            triangle_count: obj.faces.len(),
            duplicated: dup_vertices,
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

// .--------------------------------------------------------------------------.
// | Enum: Mark                                                               |
// '--------------------------------------------------------------------------'

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unmarked,
    External1,
    External2,
}

// .--------------------------------------------------------------------------.
// | Public functions                                                         |
// '--------------------------------------------------------------------------'

pub fn compress_obj(obj: &Obj) -> EdgeBreaker {
    let mut he = HalfEdges::init(obj);
    let eb = compress(&mut he);
    debug!("History: {:?}", eb.history);
    eb
}

pub fn decompress_obj(eb: &EdgeBreaker, vertices: Vec<[f32; 3]>) -> Obj {
    let faces = decompress(eb);
    debug!("Faces: {:?}", faces);

    Obj {
        faces,
        vertices: eb.previous.iter().map(|&x| vertices[x]).collect(),
    }
}

// .--------------------------------------------------------------------------.
// | Internal functions                                                       |
// '--------------------------------------------------------------------------'

fn compress(he: &mut HalfEdges) -> EdgeBreaker {
    // TODO: handle detached components
    let mut history = Vec::new();
    let mut previous = Vec::new();
    let mut lengths = Vec::new();
    let mut stack = Vec::new();

    let mut vm = vec![Mark::Unmarked; he.vertex_count];
    let mut hm = vec![Mark::Unmarked; he.triangle_count * 3];

    // Find the first gate
    let gate = match he.n.iter().position(|&x| x != NULL) {
        Some(i) => Id::from_offset(i),
        None => Id::new(1),
    };

    debug!("gate: {:?} ({:?}, {:?})", gate, he.n[gate], he.p[gate]);

    fn markEdges(
        mark: Mark,
        gate: Id,
        he: &mut HalfEdges,
        previous: &mut Vec<Id>,
        vm: &mut Vec<Mark>,
        hm: &mut Vec<Mark>,
    ) {
        let mut g = gate;
        loop {
            let ev = he.e[g];
            previous.push(ev);
            vm[ev] = mark;
            hm[g] = mark;
            g = he.n[g];
            if g == NULL || g == gate {
                break;
            }
        }
    }

    // Mark first boundary
    markEdges(Mark::External1, gate, he, &mut previous, &mut vm, &mut hm);

    if he.n[gate] == NULL {
        // Triangulation has no edges. Make one
        he.n[gate] = he.o[gate];
        he.p[gate] = he.o[gate];
        he.n[he.o[gate]] = gate;
        he.p[he.o[gate]] = gate;
        hm[he.o[gate]] = Mark::External1;
        vm[he.s[gate]] = Mark::External1;
        previous.push(he.s[gate]);
    } else {
        // Find other external edges (Holes)
        while let Some(i) =
            he.n.iter()
                .zip(&hm)
                .position(|(&n, &m)| n != NULL && m == Mark::Unmarked)
        {
            markEdges(
                Mark::External2,
                Id::from_offset(i),
                he,
                &mut previous,
                &mut vm,
                &mut hm,
            );
        }
    }

    // Main algorithm loop
    stack.push(gate);
    while let Some(g) = stack.pop() {
        match vm[he.v(g)] {
            Mark::Unmarked => {
                // Case C
                history.push(Op::C);
                previous.push(he.v(g));

                let gpo = he.o[HalfEdges::p(g)];
                let gno = he.o[HalfEdges::n(g)];
                let gv = he.v(g);
                let gN = he.n[g];
                let gP = he.p[g];

                // Fix flags
                hm[g] = Mark::Unmarked;
                hm[gpo] = Mark::External1;
                hm[gno] = Mark::External1;
                vm[gv] = Mark::External1;

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
            }

            Mark::External2 => {
                // Case M
                history.push(Op::H);

                let gpo = he.o[HalfEdges::p(g)];
                let gno = he.o[HalfEdges::n(g)];
                let gN = he.n[g];
                let gP = he.p[g];

                hm[g] = Mark::Unmarked;
                hm[gpo] = Mark::External1;
                hm[gno] = Mark::External1;

                let mut b = HalfEdges::n(g);
                while hm[b] != Mark::External2 {
                    b = HalfEdges::p(he.o[b]);
                }

                // Hole traversal
                let mut len = 0;
                loop {
                    let bs = he.s[b];
                    hm[b] = Mark::External1;
                    vm[bs] = Mark::External1;
                    len += 1;
                    previous.push(bs);
                    b = he.n[b];
                    if he.e[b] == he.s[gno] {
                        break;
                    }
                }
                lengths.push(len);

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

                stack.push(gno);
            }

            Mark::External1 => {
                if HalfEdges::p(g) == he.p[g] {
                    if HalfEdges::n(g) == he.n[g] {
                        // Case E
                        history.push(Op::E);

                        let gn = HalfEdges::n(g);
                        let gp = HalfEdges::p(g);
                        hm[g] = Mark::Unmarked;
                        hm[gn] = Mark::Unmarked;
                        hm[gp] = Mark::Unmarked;
                    } else {
                        // Case L
                        history.push(Op::L);

                        let gP = he.p[g];
                        let gPP = he.p[gP];
                        let gno = he.o[HalfEdges::n(g)];
                        let gN = he.n[g];

                        // Flags
                        hm[g] = Mark::Unmarked;
                        hm[gP] = Mark::Unmarked;
                        hm[gno] = Mark::External1;

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
                        hm[g] = Mark::Unmarked;
                        hm[gN] = Mark::Unmarked;
                        hm[gpo] = Mark::External1;

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
                        hm[g] = Mark::Unmarked;
                        hm[gpo] = Mark::External1;
                        hm[gno] = Mark::External1;

                        // Find b by rotating around v
                        let mut b = HalfEdges::n(g);
                        while hm[b] == Mark::Unmarked {
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
    }

    EdgeBreaker {
        history,
        previous,
        lengths,
        duplicated: he.duplicated.clone(),
    }
}

fn decompress(eb: &EdgeBreaker) -> Vec<[usize; 3]> {
    let t = eb.history.len();
    let mut d: i32 = 0; // |S| - |E|
    let mut c: usize = 0; // |C| = |V_i|
    let mut e: i32 = 0; // 3|E| + |L| + |R| - |C| - |S| = |V_e|
    let mut s: usize = 0; // |S|
    let mut stack: Vec<(i32, usize)> = Vec::new();
    let mut offsets: Vec<usize> = vec![0; eb.history.iter().filter(|&o| *o == Op::S).count()];
    let mut edge_count = 0;
    let mut li = 0;

    // .----------------------------------------
    // | Preprocessing phase

    for op in eb.history.iter() {
        match op {
            Op::S => {
                e -= 1;
                stack.push((e, s));
                s += 1;
                d += 1;
                edge_count += 1;
            }

            Op::E => {
                e += 3;
                edge_count += 3;
                if d <= 0 {
                    break;
                }
                let (_e, _s) = stack.pop().expect("(e,s) stack prematurely empty!");
                offsets[_s] = (e - _e - 2)
                    .try_into()
                    .expect("Encountered negative S offset!");
                d -= 1;
            }

            Op::C => {
                e -= 1;
                c += 1;
                edge_count += 1;
            }

            Op::R => {
                e += 1;
                edge_count += 2;
            }

            Op::L => {
                e += 1;
                edge_count += 2;
            }

            Op::H => {
                let l = eb.lengths[li];
                e -= l as i32 + 1;
                li += 1;
            }
        }
    }

    // '----------------------------------------

    // Sanity check
    assert!(t == eb.history.len());
    // assert!(c as i32 + e == eb.previous.len() as i32);

    // .----------------------------------------
    // | Generation phase

    let mut tv: Vec<[usize; 3]> = Vec::with_capacity(t);
    let mut vc = e as usize;
    let mut ec: usize = 0;
    s = 0;
    li = 0;

    // Create bounding loop
    let mut end = vec![NULL; edge_count];
    let mut next = vec![NULL; edge_count];
    let mut prev = vec![NULL; edge_count];

    for v in 0..vc {
        next[v] = Id::from_offset((v + 1) % vc);
        prev[v] = Id::from_offset((v + vc - 1) % vc);
        end[v] = Id::from_offset(ec);
        ec += 1;
    }

    let mut g = Id::new(1);
    let mut stack: Vec<Id> = vec![g];
    for op in eb.history.iter() {
        match op {
            Op::C => {
                let gp = prev[g];
                vc += 1;
                tv.push([end[gp].id(), end[g].id(), vc]);

                ec += 1;
                let a = Id::new(ec);

                end[a] = Id::new(vc);
                next[gp] = a;
                prev[a] = prev[g];
                next[a] = g;
                prev[g] = a;
            }

            Op::R => {
                let gp = prev[g];
                let gn = next[g];
                tv.push([end[gp].id(), end[g].id(), end[gn].id()]);
                next[gp] = gn;
                prev[gn] = gp;
                g = gn;
            }

            Op::L => {
                let gp = prev[g];
                let gpp = prev[gp];
                tv.push([end[gp].id(), end[g].id(), end[gpp].id()]);

                prev[g] = gpp;
                next[gpp] = g;
            }

            Op::E => {
                let gp = prev[g];
                let gn = next[g];
                tv.push([end[gp].id(), end[g].id(), end[gn].id()]);

                g = stack.pop().expect("Hmmmmmmm :(");
            }

            Op::S => {
                let gp = prev[g];
                let mut d = next[g];
                for _ in 0..offsets[s] {
                    d = next[d];
                }
                s += 1;

                tv.push([end[gp].id(), end[g].id(), end[d].id()]);

                ec += 1;
                let a = Id::new(ec);
                end[a] = end[d];
                next[gp] = a;
                prev[a] = gp;

                stack.push(a);
                let dn = next[d];
                next[a] = dn;
                prev[dn] = a;
                prev[g] = d;
                next[d] = g;
            }

            Op::H => {
                let gp = prev[g];
                tv.push([end[gp].id(), end[g].id(), vc + 1]);

                let mut d = gp;
                let l = eb.lengths[li];
                li += 1;
                for _ in 0..l {
                    ec += 1;
                    let a = Id::new(ec);

                    next[d] = a;
                    prev[a] = d;
                    vc += 1;
                    end[a] = Id::new(vc);
                    d = a;
                }
                ec += 1;
                let a = Id::new(ec);
                next[d] = a;
                prev[a] = d;
                end[a] = Id::new(vc + 1 - l);
                next[a] = g;
                prev[g] = a;
            }
        }
    }

    // '----------------------------------------

    dbg!(vc);
    dbg!(&eb.duplicated);
    let vertex_count = vc + 1 - eb.duplicated.len();
    dbg!(&tv);
    for t in tv.iter_mut() {
        for v in t.iter_mut() {
            if *v > vertex_count {
                *v = eb.duplicated[*v - vertex_count - 1].id();
            }
        }
    }
    tv
}
