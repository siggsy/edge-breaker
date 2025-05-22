use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    C,
    H,
    L,
    E,
    R,
    S,
}

// .--------------------------------------------------------------------------.
// | Constants                                                                |
// '--------------------------------------------------------------------------'

pub const NULL: Id = Id(0);

// .--------------------------------------------------------------------------.
// | Definition                                                               |
// '--------------------------------------------------------------------------'

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(usize);

// .--------------------------------------------------------------------------.
// | Implementations                                                          |
// '--------------------------------------------------------------------------'

impl Id {
    pub fn from_offset(off: usize) -> Id {
        Id(off + 1)
    }

    pub fn new(id: usize) -> Id {
        Id(id)
    }

    pub fn offset(&self) -> usize {
        self.0 - 1
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

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

// .--------------------------------------------------------------------------.
// | Definition                                                               |
// '--------------------------------------------------------------------------'

#[derive(Eq)]
pub struct Edge {
    a: usize,
    b: usize,
}

// .--------------------------------------------------------------------------.
// | Implementations                                                          |
// '--------------------------------------------------------------------------'

impl Edge {
    pub fn new(a: usize, b: usize) -> Edge {
        Edge { a, b }
    }
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
