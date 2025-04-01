use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::fmt::Debug;

use crate::obj::Obj;


#[derive(Debug, PartialEq, Eq)]
enum Op {
    C,
    L,
    E,
    R,
    S,
}

#[derive(Debug)]
pub struct EdgeBreaker {
    s: Vec<Id>,
    e: Vec<Id>,
    n: Vec<Id>,
    o: Vec<Id>,
    p: Vec<Id>,

    vm: Vec<bool>,
    hm: Vec<bool>,

    gate: Id,
    history: Vec<Op>,
    previous: Vec<Id>,
}

#[derive(Eq)]
struct Edge {
    a: usize,
    b: usize,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b || self.a == other.b && self.b == other.a
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.a < self.b {
            self.a.hash(state);
            self.b.hash(state);
        } else {
            self.b.hash(state);
            self.a.hash(state);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Id(usize);

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => write!(f, "NULL"),
            id => write!(f, "#{}", id),
        }
    }
}

impl<T> Index<Id> for Vec<T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        self.index(index.0 - 1)
    }
}

impl<T> IndexMut<Id> for Vec<T> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.index_mut(index.0 - 1)
    }
}

impl EdgeBreaker {
    pub fn compress(obj: &Obj) -> Self {
        let mut eb = Self::init(obj);
        println!("Before:\n{:?}", eb);
        eb.run();
        println!("After:\n{:?}", eb);

        eb
    }

    pub fn decompress(&mut self) -> Obj {
        todo!("Not yet implemented")
    }

    fn init(obj: &Obj) -> Self {
        let capacity = obj.faces.len() * 3; // 0 == null
        let null = Id(0);
        let mut s: Vec<Id> = vec![null; capacity];
        let mut e: Vec<Id> = vec![null; capacity];
        let mut n: Vec<Id> = vec![null; capacity];
        let mut p: Vec<Id> = vec![null; capacity];
        let mut o: Vec<Id> = vec![null; capacity];
        let mut vm: Vec<bool> = vec![false; obj.vertices.len()];
        let mut hm: Vec<bool> = vec![false; capacity];
        let mut boundary: Vec<Id> = Vec::new();

        let mut edge_map: HashMap<Edge, Id> = HashMap::new();
        for (t, face) in obj.faces.iter().enumerate() {
            let offset = t * 3 + 1;

            // Construct half-edges from triangle
            for i in 0..3 {
                let h = Id(i + offset);

                s[h] = Id(face[i]);
                e[h] = Id(face[(i + 1) % 3]);
                n[h] = Id((i + 1) % 3 + offset);
                p[h] = Id((i + 2) % 3 + offset);
            }

            // Check for collisions and fix boundary
            for i in 0..3 {
                let h = Id(i + offset);
                let edge = Edge {
                    a: face[i],
                    b: face[(i + 1) % 3],
                };

                if let Some(_h) = edge_map.insert(edge, h) {
                    // Fix next and previous for triangles
                    let _next = n[_h];
                    let _prev = p[_h];
                    let next = n[h];
                    let prev = p[h];

                    // TODO handle non-orientable and non-manifold meshes
                    if _next == null || _prev == null || next == null || prev == null {
                        panic!("Surface is not a 2-manifold");
                    }

                    n[prev] = _next;
                    p[_next] = prev;
                    n[_prev] = next;
                    p[next] = _prev;

                    // Reset next and previous half edges
                    n[_h] = null;
                    p[_h] = null;
                    n[h] = null;
                    p[h] = null;

                    o[h] = _h;
                    o[_h] = h;
                }
            }
        }

        let gate = match n.iter().find(|&x| *x != null) {
            Some(&gate) => gate,
            None => Id(1),
        };

        // Find boundary
        let mut g = gate;
        while g != null && e[g] != s[gate] {
            let ev = e[g];
            boundary.push(ev);
            vm[ev] = true;
            hm[g] = true;
            g = n[g];
        }
        boundary.push(s[gate]);
        vm[s[gate]] = true;

        if n[gate] == null {
            n[gate] = o[gate];
            p[gate] = o[gate];
        }

        Self {
            s,
            e,
            n,
            o,
            p,
            vm,
            hm,
            gate,
            history: vec![],
            previous: boundary,
        }
    }


    fn run(&mut self) {

        let mut stack = Vec::new();
        stack.push(self.gate);

        while let Some(g) = stack.pop() {
            if !self.vm[self.v(g)] {
                // Case C
                self.history.push(Op::C);
                self.previous.push(self.v(g));

                let gpo = self.o[self.p(g)];
                let gno = self.o[self.n(g)];
                let gv = self.v(g);
                let gN = self.n[g];
                let gP = self.p[g];

                // Fix flags
                self.hm[g] = false;
                self.hm[gpo] = true;
                self.hm[gno] = true;
                self.vm[gv] = true;

                // Link 1
                self.p[gpo] = self.p[g];
                self.n[gP] = gpo;

                // Link 2
                self.n[gpo] = gno;
                self.p[gno] = gpo;

                // Link 3
                self.n[gno] = gN;
                self.p[gN] = gno;

                stack.push(gno);
            } else {
                if self.p(g) == self.p[g] {
                    if self.n(g) == self.n[g] {
                        // Case E
                        self.history.push(Op::E);

                        let gn = self.n(g);
                        let gp = self.p(g);
                        self.hm[g] = false;
                        self.hm[gn] = false;
                        self.hm[gp] = false;
                    } else {
                        // Case L
                        self.history.push(Op::L);

                        let gno = self.o[self.n(g)];
                        let gPP = self.p[self.p[g]];
                        let gN = self.n[g];
                        let gP = self.p[g];

                        // Flags
                        self.hm[g] = false;
                        self.hm[gP] = false;
                        self.hm[gno] = true;

                        // Link 1
                        self.n[gPP] = gno;
                        self.p[gno] = gPP;

                        // Link 2
                        self.n[gno] = gN;
                        self.p[gN] = gno;

                        stack.push(gno);
                    }
                } else {
                    if self.n(g) == self.n[g] {
                        // Case R
                        self.history.push(Op::R);

                        let gN = self.n[g];
                        let gNN = self.n[gN];
                        let gpo = self.o[self.p(g)];
                        let gP = self.p[g];

                        // Flags
                        self.hm[g] = false;
                        self.hm[gN] = false;
                        self.hm[gpo] = true;

                        // Link 1
                        self.p[gNN] = gpo;
                        self.n[gpo] = gNN;

                        // Link 2
                        self.p[gpo] = gP;
                        self.n[gP] = gpo;

                        stack.push(gpo);
                    } else {
                        // Case S
                        self.history.push(Op::S);

                        let gno = self.o[self.n(g)];
                        let gpo = self.o[self.p(g)];
                        let gN = self.n[g];
                        let gP = self.p[g];

                        // Flags
                        self.hm[g] = false;
                        self.hm[gpo] = true;
                        self.hm[gno] = true;

                        // Find b by rotating around v
                        let mut b = self.n(g);
                        while !self.hm[b] {
                            b = self.p(self.o[b]);
                        }

                        // Link 1
                        self.n[gP] = gpo;
                        self.p[gpo] = gP;

                        // Link 2
                        let bN = self.n[b];
                        self.n[gpo] = bN;
                        self.p[bN] = gpo;

                        // Link 3
                        self.n[b] = gno;
                        self.p[gno] = b;

                        // Link 4
                        self.n[gno] = gN;
                        self.p[gN] = gno;

                        stack.push(gpo);
                        stack.push(gno);
                    }
                }
            }
        }
    }

    fn v(&self, id: Id) -> Id {
        self.e[self.n(id)]
    }

    fn n(&self, id: Id) -> Id {
        assert!(id != Id(0));
        let _id = id.0;
        let i = (_id - 1) % 3;
        let t = _id - 1 - i;
        Id((i+1) % 3 + t + 1)
    }

    fn p(&self, id: Id) -> Id {
        assert!(id != Id(0));
        let _id = id.0;
        let i = (_id - 1) % 3;
        let t = _id - 1 - i;
        Id((i+2) % 3 + t + 1)
    }
}
