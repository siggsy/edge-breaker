use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Id(usize);

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

        let Some(&gate) = n.iter().find(|&x| *x != null) else {
            // TODO handle sphere-like shapes
            panic!("Surface does not have a border");
        };

        // Find boundary
        let mut g = gate;
        let ev = e[g];
        boundary.push(ev);
        vm[ev] = true;
        hm[g] = true;
        g = n[g];
        while g != gate {
            let ev = e[g];
            boundary.push(ev);
            vm[ev] = true;
            hm[g] = true;
            g = n[g];
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

        // Useful macro rules for dealing with parallel arrays
        macro_rules! get {
            ($half_edge:ident . $($other:tt)+) => (get!(expand $half_edge, $($other)+));
            (expand $inner:expr, $acc:ident ()) => (self.$acc($inner));
            (expand $inner:expr, $acc:ident) => (self.$acc[$inner]);
            (expand $inner:expr, $acc:ident () . $($other:tt)+) => (get!(expand self.$acc($inner), $($other)+));
            (expand $inner:expr, $acc:ident . $($other:tt)+) => (get!(expand self.$acc[$inner], $($other)+));
        }

        macro_rules! set {
            ($val:expr, $half_edge:ident . $($other:tt)+) => (set!(expand $val, $half_edge, $($other)+));
            (expand $val:expr, $inner:expr, $acc:ident) => {
                {
                    let id = $inner;
                    self.$acc[id] = $val
                }
            };
            (expand $val:expr, $inner:expr, $acc:ident () . $($other:tt)+) => (set!(expand $val, self.$acc($inner), $($other)+));
            (expand $val:expr, $inner:expr, $acc:ident . $($other:tt)+) => (set!(expand $val, self.$acc[$inner], $($other)+));
        }

        let mut stack = Vec::new();
        stack.push(self.gate);

        while let Some(g) = stack.pop() {
            if !get!(g.v().vm) {
                // Case C
                self.history.push(Op::C);
                self.previous.push(self.v(g));

                // Fix flags
                set!(false, g.hm);
                set!(true, g.p().o.hm);
                set!(true, g.n().o.hm);
                set!(true, g.v().vm);

                // Link 1
                set!(get!(g.p), g.p().o.p);
                set!(get!(g.p().o), g.p.n);

                // Link 2
                set!(get!(g.n().o), g.p().o.n);
                set!(get!(g.p().o), g.n().o.p);

                // Link 3
                set!(get!(g.n), g.n().o.n);
                set!(get!(g.n().o), g.n.p);
            } else {
                if get!(g.p()) == get!(g.p) {
                    if get!(g.n()) == get!(g.n) {
                        // Case E
                        // TODO
                    } else {
                        // Case L
                        // TODO
                    }
                } else {
                    if get!(g.n()) == get!(g.n) {
                        // Case R
                        // TODO
                    } else {
                        // Case S
                        // TODO
                    }
                }
            }
        }
    }

    fn v(&self, id: Id) -> Id {
        self.e[self.n(id)]
    }

    fn n(&self, id: Id) -> Id {
        let _id = id.0;
        let i = (_id - 1) % 3;
        let t = _id - 1 - i;
        Id((i+1) % 3 + t + 1)
    }

    fn p(&self, id: Id) -> Id {
        let _id = id.0;
        let i = (_id - 1) % 3;
        let t = _id - 1 - i;
        Id((i+2) % 3 + t + 1)
    }
}
