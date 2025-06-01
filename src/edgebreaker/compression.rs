use std::collections::HashMap;

use crate::{edgebreaker::public::Op, obj::Obj};
use log::debug;

use super::{
    EdgeBreaker,
    common::{Id, NULL},
};

// .--------------------------------------------------------------------------.
// | Struct: HalfEdges                                                        |
// '--------------------------------------------------------------------------'

#[derive(Debug)]
pub struct HalfEdges {
    vertex_count: usize,
    triangle_count: usize,
    conflicts: HashMap<(usize, usize), usize>,
    s: Vec<Id>,
    e: Vec<Id>,
    n: Vec<Id>,
    o: Vec<Id>,
    p: Vec<Id>,
}

impl HalfEdges {
    pub fn init(obj: &Obj) -> Self {
        let capacity = obj.faces.len() * 3;
        let vertex_count = obj.vertices.len();
        let mut conflicts: HashMap<(usize, usize), usize> = HashMap::new();
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
                let a = s[h];
                let b = e[h];

                if let Some(&g) = edge_map.get(&(b.id(), a.id())) {
                    // Fix next and previous for triangles
                    let gN = n[g];
                    let gP = p[g];
                    let hN = n[h];
                    let hP = p[h];

                    // non-manifold edge.
                    if gN == NULL || gP == NULL {
                        // non-manifold edge.
                        let edge = ((a.id()), b.id());
                        let conflict_count = match conflicts.get(&edge) {
                            Some(c) => *c,
                            None => 0,
                        };
                        conflicts.insert(edge, conflict_count + 1);
                    }
                    // First collision: make half edges internal
                    else {
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
                } else if let Some(_) = edge_map.get(&(a.id(), b.id())) {
                    let edge = ((a.id()), b.id());
                    let conflict_count = match conflicts.get(&edge) {
                        Some(c) => *c,
                        None => 0,
                    };
                    conflicts.insert(edge, conflict_count + 1);
                } else {
                    edge_map.insert((a.id(), b.id()), h);
                }
            }
        }

        Self {
            vertex_count: vertex_count,
            triangle_count: obj.faces.len(),
            conflicts,
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

    fn print_edge(&self, id: Id) -> String {
        format!("{:?}", (self.s[id], self.e[id]))
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
    External3(Id),
}

// .--------------------------------------------------------------------------.
// | Entry point                                                              |
// '--------------------------------------------------------------------------'

fn markEdges(
    mark: Mark,
    gate: Id,
    he: &mut HalfEdges,
    previous: &mut Vec<Id>,
    vm: &mut Vec<Mark>,
    hm: &mut Vec<Mark>,
    duplicated: &mut Vec<Id>,
) {
    let mut g = gate;
    loop {
        let mut sv = he.s[g];
        let mut ev = he.e[g];

        // Fix conflicts
        let edge = (sv.id(), ev.id());
        if let Some(&c) = he.conflicts.get(&edge) {
            if c != 0 {
                // Assign new IDs
                vm.push(vm[sv]);
                vm.push(vm[ev]);

                duplicated.push(sv);
                sv = Id::new(he.vertex_count + duplicated.len());
                duplicated.push(ev);
                ev = Id::new(he.vertex_count + duplicated.len());

                // Walk around vertices and assign new ones
                he.s[g] = sv;
                he.e[g] = ev;

                let mut b = he.p[g];
                while b != NULL {
                    he.e[b] = sv;
                    b = HalfEdges::n(b);
                    he.s[b] = sv;
                    b = he.o[b];
                }

                b = g;
                while b != NULL {
                    he.e[b] = ev;
                    b = HalfEdges::n(b);
                    he.s[b] = ev;
                    b = he.o[b];
                }

                he.conflicts.insert(edge, c - 1);
            }
        }

        // Mark as boundary
        if mark == Mark::External1 {
            previous.push(ev);
        }
        vm[ev] = mark;
        hm[g] = mark;
        g = he.n[g];
        if g == NULL || g == gate {
            break;
        }
    }
}

pub fn compress(he: &mut HalfEdges) -> EdgeBreaker {
    let mut history = Vec::new();
    let mut previous = Vec::new();
    let mut lengths = Vec::new();
    let mut m_table = Vec::new();
    let mut stack = Vec::new();
    let mut duplicated = Vec::new();
    let mut components = Vec::new();

    let mut vm = vec![Mark::Unmarked; he.vertex_count];
    let mut hm = vec![Mark::Unmarked; he.triangle_count * 3];

    debug!("conflicts: {:?}", he.conflicts);

    // Find the first gate
    let gate = match he.n.iter().position(|&x| x != NULL) {
        Some(i) => Id::from_offset(i),
        None => Id::new(1),
    };

    debug!("gate: {}", he.print_edge(gate));

    // Mark first boundary
    markEdges(
        Mark::External1,
        gate,
        he,
        &mut previous,
        &mut vm,
        &mut hm,
        &mut duplicated,
    );

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
            let _gate = Id::from_offset(i);
            markEdges(
                Mark::External2,
                _gate,
                he,
                &mut previous,
                &mut vm,
                &mut hm,
                &mut duplicated,
            );
            components.push(_gate);
        }
    }

    // Main algorithm loop
    stack.push(gate);
    'main: loop {
        while let Some(g) = stack.pop() {
            if let Mark::External3(_g) = hm[g] {
                // Mark with External1
                let mut b = g;
                loop {
                    hm[b] = Mark::External1;
                    vm[he.e[b]] = Mark::External1;
                    b = he.n[b];
                    if he.e[b] == he.e[g] {
                        break;
                    }
                }
            }

            match vm[he.v(g)] {
                Mark::Unmarked => {
                    debug!("Case C");
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
                    debug!("Case M");

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
                        previous.push(he.e[b]);
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

                Mark::External1 | Mark::External3(_) => {
                    if HalfEdges::p(g) == he.p[g] {
                        if HalfEdges::n(g) == he.n[g] {
                            // Case E
                            debug!("Case E");
                            history.push(Op::E);

                            let gn = HalfEdges::n(g);
                            let gp = HalfEdges::p(g);
                            hm[g] = Mark::Unmarked;
                            hm[gn] = Mark::Unmarked;
                            hm[gp] = Mark::Unmarked;
                        } else {
                            // Case L
                            debug!("Case L");
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
                            debug!("Case R");
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
                            match vm[he.v(g)] {
                                Mark::External3(split_g) => {
                                    // Case M'
                                    debug!("Case M'");
                                    debug!("split_g: {:?}", split_g);

                                    // Mark with External1
                                    let mut b = split_g;
                                    let mut l = 0;
                                    loop {
                                        if hm[b] != Mark::External3(split_g) {
                                            debug!("mark: {:?}", hm[b]);
                                            debug!("b: {:?}", (he.s[b], he.e[b]));
                                        }

                                        hm[b] = Mark::External1;
                                        vm[he.e[b]] = Mark::External1;
                                        b = he.n[b];
                                        l += 1;
                                        if he.e[b] == he.e[split_g] {
                                            break;
                                        }
                                    }

                                    // Check if this is a self merge
                                    if he.e[split_g] == he.e[g] {
                                        stack.push(g);
                                        continue;
                                    }

                                    // Calculate offset
                                    b = split_g;
                                    let mut o = 0;
                                    loop {
                                        if he.e[b] == he.v(g) {
                                            break;
                                        }
                                        o += 1;
                                        b = he.n[b];
                                        if he.e[b] == he.e[split_g] {
                                            break;
                                        }
                                    }

                                    // Find split_g in stack
                                    let Some(p) = stack.iter().position(|&_g| split_g == _g) else {
                                        panic!(
                                            "Invalid stack structure. Did not find split_g in stack!"
                                        );
                                    };

                                    history.push(Op::M);
                                    m_table.push((p, o, l));

                                    let gp = HalfEdges::p(g);
                                    let gn = HalfEdges::n(g);
                                    let gpo = he.o[gp];
                                    let gno = he.o[gn];
                                    let gP = he.p[g];
                                    let gN = he.n[g];

                                    // Fix links and marks
                                    hm[g] = Mark::Unmarked;
                                    hm[gpo] = Mark::External1;
                                    hm[gno] = Mark::External1;

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
                                }
                                Mark::External1 => {
                                    // Case S
                                    debug!("Case S");
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

                                    debug!("g: {}", he.print_edge(g));
                                    debug!("gpo: {}", he.print_edge(gpo));
                                    debug!("gno: {}", he.print_edge(gno));
                                    debug!("gP: {}", he.print_edge(gP));
                                    debug!("gN: {}", he.print_edge(gN));
                                    debug!("b: {}", he.print_edge(b));
                                    debug!("hm[b]: {:?}", hm[b]);

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

                                    // // Mark left loop with External3
                                    let mut b = gpo;
                                    debug!("marking 3");
                                    let should_mark = loop {
                                        match hm[b] {
                                            Mark::External3(_) => {
                                                break false;
                                            }
                                            _ => {}
                                        }

                                        match vm[he.e[b]] {
                                            Mark::External3(_) => {
                                                break false;
                                            }
                                            _ => {}
                                        }

                                        b = he.n[b];
                                        if he.e[b] == he.e[gpo] {
                                            break true;
                                        }
                                    };

                                    if should_mark {
                                        loop {
                                            // debug!("b: {:?}", (he.s[b], he.e[b]));
                                            debug!("overriding: {:?}", hm[b]);
                                            debug!("vertex: {:?}", vm[he.e[b]]);
                                            hm[b] = Mark::External3(gpo);
                                            vm[he.e[b]] = Mark::External3(gpo);
                                            b = he.n[b];
                                            if he.e[b] == he.e[gpo] {
                                                break;
                                            }
                                        }
                                    }

                                    debug!("g: {:?}", (he.s[g], he.e[g]));
                                    debug!("gpo: {:?}", (he.s[gpo], he.e[gpo]));
                                    debug!("hm[gpo]: {:?}", hm[gpo]);

                                    // s_stack.push(history.len() - 1);
                                    stack.push(gpo);
                                    stack.push(gno);
                                }
                                _ => {
                                    panic!("cannot happen");
                                }
                            }
                        }
                    }
                }
            }
            debug!("hist.len {}", history.len());
        }

        while let Some(_gate) = components.pop() {
            if hm[_gate] == Mark::External2 {
                markEdges(
                    Mark::External1,
                    _gate,
                    he,
                    &mut previous,
                    &mut vm,
                    &mut hm,
                    &mut duplicated,
                );
                stack.push(_gate);
                continue 'main;
            }
        }
        break;
    }

    for v in previous.iter_mut() {
        if v.id() > he.vertex_count {
            *v = duplicated[v.id() - he.vertex_count - 1];
        }
    }

    EdgeBreaker {
        history,
        previous,
        lengths,
        m_table,
    }
}
