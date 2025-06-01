// ,---------------------------------------------------------------------------
// | Op: history commands
// '---------------------------------------------------------------------------

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use bitvec::{bitvec, order::Msb0, view::BitView};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Op {
    C,
    H,
    L,
    E,
    R,
    S,
    M,
}

impl Op {
    pub fn encode_history(hist: &[Self]) -> (String, usize) {
        let mut bvec = bitvec![u8, Msb0;];
        for op in hist {
            match op {
                Op::C => bvec.push(false),
                Op::S | Op::H | Op::M => bvec.extend(&0b100u8.view_bits::<Msb0>()[5..8]),
                Op::R => bvec.extend(&0b101u8.view_bits::<Msb0>()[5..8]),
                Op::L => bvec.extend(&0b110u8.view_bits::<Msb0>()[5..8]),
                Op::E => bvec.extend(&0b111u8.view_bits::<Msb0>()[5..8]),
            }
        }

        let pad = {
            let _p = bvec.len() % 8;
            if _p == 0 { 0 } else { 8 - _p }
        };
        (BASE64_STANDARD_NO_PAD.encode(bvec.into_vec()), pad)
    }

    pub fn decode_history(enc: &str, pad: usize) -> Vec<Op> {
        let mut ops = Vec::new();
        let bytes = BASE64_STANDARD_NO_PAD.decode(enc).unwrap();
        let mut bits = bytes.view_bits::<Msb0>().iter();
        while let Some(b) = bits.next() {
            if bits.len() < pad {
                break;
            }

            if *b {
                let b1 = bits.next().unwrap();
                let b2 = bits.next().unwrap();
                match (*b1, *b2) {
                    (false, false) => ops.push(Op::S),
                    (false, true) => ops.push(Op::R),
                    (true, false) => ops.push(Op::L),
                    (true, true) => ops.push(Op::E),
                }
            } else {
                ops.push(Op::C)
            }
        }

        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_test() {
        let ops = vec![
            Op::C,
            Op::C,
            Op::C,
            Op::S,
            Op::S,
            Op::L,
            Op::E,
            Op::E,
            Op::M,
        ];
        let (base64, pad) = Op::encode_history(&ops);
        println!("original: {:?}", ops);
        println!("encoded:  {:?}", base64);
        println!("decoded:  {:?}", Op::decode_history(&base64, pad));
    }
}
