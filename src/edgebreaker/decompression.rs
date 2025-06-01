use log::debug;

use crate::edgebreaker::common::{Id, NULL};

use super::{EdgeBreaker, public::Op};

pub fn decompress(eb: &EdgeBreaker) -> Vec<[usize; 3]> {
    let t = eb.history.len();
    let mut components = Vec::new();
    let mut d: i32 = 0; // |S| - |E|
    let mut c: usize = 0; // |C| = |V_i|
    let mut e: i32 = 0; // 3|E| + |L| + |R| - |C| - |S| = |V_e|
    let mut s: usize = 0; // |S|
    let mut stack: Vec<(i32, usize)> = Vec::new();
    let mut offsets: Vec<usize> = vec![0; eb.history.iter().filter(|&o| *o == Op::S).count()];
    let mut edge_count = 0;
    let mut vertex_count = 0;
    let mut h = 0;
    let mut a = 0;
    let mut li = 0;
    let mut mi = 0;

    // Create bounding loop
    let mut end = vec![NULL; edge_count];
    let mut next = vec![NULL; edge_count];
    let mut prev = vec![NULL; edge_count];

    // .----------------------------------------
    // | Preprocessing phase

    for op in eb.history.iter() {
        match op {
            Op::S => {
                e -= 1;
                stack.push((e, s));
                s += 1;
                d += 1;
                a += 1;
            }

            Op::E => {
                e += 3;
                if d <= 0 {
                    let new_edge_count = edge_count + a + e as usize;
                    end.resize(new_edge_count, NULL);
                    next.resize(new_edge_count, NULL);
                    prev.resize(new_edge_count, NULL);
                    let (_, _e) = components.last().unwrap_or(&(NULL, 0));
                    let bc = e as usize;

                    for b in 0..bc {
                        next[b + edge_count] = Id::from_offset(((b + 1) % bc) + edge_count);
                        prev[b + edge_count] = Id::from_offset(((b + bc - 1) % bc) + edge_count);
                        end[b + edge_count] = Id::from_offset(b + vertex_count);
                    }

                    components.push((Id::new(1 + edge_count), bc));
                    edge_count = new_edge_count;
                    vertex_count += bc + h + c;
                    debug!("components: {:?}", components.last());
                    debug!("next: {:?}", next);
                    debug!("end: {:?}", end);
                    e = 0;
                    c = 0;
                    d = 0;
                } else {
                    let (_e, _s) = stack.pop().expect("(e,s) stack prematurely empty!");
                    offsets[_s] = (e - _e - 2)
                        .try_into()
                        .expect("Encountered negative S offset!");
                    d -= 1;
                }
            }

            Op::C => {
                e -= 1;
                c += 1;
                a += 1;
            }

            Op::R => {
                e += 1;
            }

            Op::L => {
                e += 1;
            }

            Op::H => {
                let l = eb.lengths[li];
                e -= l as i32 + 1;
                h += l + 1;
                li += 1;
                a += l + 1;
            }
            Op::M => {
                let (p, _, l) = eb.m_table[mi];
                mi += 1;

                e -= 1;
                a += 1;
                let (_e, _s) = stack.remove(p);

                offsets[_s] = (-_e - l as i32)
                    .try_into()
                    .expect("Encountered negative S offset!");
                d -= 1;
            }
        }
    }

    // '----------------------------------------

    // .----------------------------------------
    // | Generation phase

    let mut tv: Vec<[usize; 3]> = Vec::with_capacity(t);
    let mut ci = 0;
    let (mut g, _e) = components[ci];
    ci += 1;

    let mut vc = _e as usize;
    let mut ec: usize = _e as usize;
    s = 0;
    li = 0;
    mi = 0;

    let mut stack: Vec<Id> = vec![];

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

                if let Some(_g) = stack.pop() {
                    g = _g;
                } else if ci < components.len() {
                    debug!("new component!");
                    let (_g, _e) = components[ci];
                    g = _g;
                    ec += _e as usize;
                    vc += _e;
                    ci += 1;
                    debug!("g: {:?}", g);
                    debug!("next: {:?}", next);
                    debug!("end: {:?}", end);
                }
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

            Op::M => {
                let gp = prev[g];
                let (p, o, _) = eb.m_table[mi];
                mi += 1;

                let mut d = stack[p];
                for _ in 0..o {
                    d = next[d];
                }
                let dn = next[d];

                tv.push([end[gp].id(), end[g].id(), end[d].id()]);

                ec += 1;
                let a = Id::new(ec);
                end[a] = end[d];
                next[gp] = a;
                prev[a] = gp;
                next[a] = dn;
                prev[dn] = a;
                next[d] = g;
                prev[g] = d;

                g = stack.pop().expect("Invalid decompression stack");
            }
        }
        // debug!("after: {:?}", op);
        // debug!(
        //     "last: {:?}",
        //     tv.last().expect("test").map(|v| eb.previous[v - 1].id())
        // );
        // debug!("next: {:?}", next);
        // debug!("end: {:?}", end);
    }

    // '----------------------------------------

    for t in tv.iter_mut() {
        for v in t.iter_mut() {
            *v = eb.previous[*v - 1].id();
        }
    }
    tv
}
