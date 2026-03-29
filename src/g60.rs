/* G60 encoding */

use crate::base::*;

pub struct G60;

const COEFFS: [u16; 8] = [14, 3, 40, 9, 2, 24, 5, 1];
const R_COEFFS: [u16; 11] = [0, 14, 3, 0, 40, 9, 2, 0, 24, 5, 1];

#[derive(PartialEq,Eq,Default,Debug)]
pub struct G60State { rem: u16, cnt: u8 }

macro_rules! chrs { ($($e:expr),*) => { &[$(Self::chr($e)),*][..] }; }

impl Encoding for G60 {
    const CHUNK_SIZE: usize = 8;
    const ENC_CHUNK_SIZE: usize = 11;
    const ALPHABET: &[u8]
        = b"0123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    type EncState = G60State;
    type DecState = G60State;

    fn encode_u8<W: Write>(co: &mut Encoder<G60, W>, b: u8) -> io::Result<()> {
        let cnt = co.state.cnt;
        let rem = co.state.rem;
        let b = b as u16;
        let v1 = b * COEFFS[cnt as usize];
        let (out, rem) = match cnt {
            0 => (chrs!(), v1),
            1 | 4 | 6 => {
                let (q1, r1) = (v1 / 60 + rem, v1 % 60);
                (chrs!(q1/60, q1%60), r1)
            },
            2 | 5 => (chrs!(v1/3600+rem), v1%3600),
            3 => {
                let v1 = if b >= 128 { v1 + 48 } else { v1 };
                let (q1, r1) = (v1 / 60 + rem, v1 % 60);
                (chrs!(q1/60), (q1%60)*60+r1)
            },
            7 => (chrs!(v1/60+rem, v1%60), 0),
            _ => unreachable!(),
        };
        co.state = G60State { rem, cnt: (cnt + 1) & 0x7 };
        co.inner.write_all(out)
    }

    fn finish_encode<W: Write>(co: &mut Encoder<G60, W>) -> io::Result<()> {
        let rem = co.state.rem;
        let out = match co.state.cnt {
            0         => chrs!(),
            2 | 5 | 7 => chrs!(rem),
            _         => chrs!(rem / 60, rem % 60),
        };
        co.inner.write_all(out)
    }

    fn decode_u8<W: Write>(co: &mut Decoder<G60, W>, b: u8) -> io::Result<()> {
        let cnt = co.state.cnt;
        let rem = co.state.rem;
        let val = 60*rem + (b as u16);
        let ce = R_COEFFS[cnt as usize];
        let cnt = (cnt + 1) % 11;
        if ce == 0 {
            co.state = G60State { rem: val, cnt };
            Ok(())
        } else {
            let val = if cnt == 6 && val >= 1152 { val - 48 } else { val };
            co.state = G60State { rem: val % ce, cnt };
            co.inner.write_all(&[(val/ce) as u8])
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples() {
        assert_eq!(G60::encode_slice(b""), "");
        assert_eq!(G60::encode_slice(b"H"), "Go");
        assert_eq!(G60::encode_slice(b"He"), "Gt3");
        assert_eq!(G60::encode_slice(b"Hel"), "Gt4C0");
        assert_eq!(G60::encode_slice(b"Hell"), "Gt4CGC");
        assert_eq!(G60::encode_slice(b"Hello"), "Gt4CGFi");
        assert_eq!(G60::encode_slice(b"Hello,"), "Gt4CGFiHc");
        assert_eq!(G60::encode_slice(b"Hello, "), "Gt4CGFiHeg");
        assert_eq!(G60::encode_slice(b"Hello, w"), "Gt4CGFiHehz");
        assert_eq!(G60::encode_slice(b"Hello, world!"), "Gt4CGFiHehzRzjCF16");
        assert_eq!(G60::encode_slice(b"Hella, would???"), "Gt4CGFEHehzRzsCF26RHF");
        assert_eq!(G60::encode_slice(&[47, 61]), "B13");
        assert_eq!(G60::encode_slice(&[197, 121, 221]), "m45TL");
        assert_eq!(G60::encode_slice(&[
                0x24, 0x3f, 0x6a, 0x88, 0x85, 0xa3, 0x08, 0xd3,
                0x13, 0x19, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x44,
                0xa4, 0x09, 0x38, 0x22, 0x29, 0x9f, 0x31, 0xd0,
                0x08, 0x2e, 0xfa, 0x98, 0xec, 0x4e, 0x6c, 0x89_u8,
            ][..]), "8TAB1GT5CjX4TGY6u6kxc8eGTdR7P3g8U1uLn3jsXM2H");
    }

    crate::stock_tests!(G60);
    // Third or sixth byte being 0xff fails partial dec because of gaps.
    // crate::max_partial_tests!(G60, false, true);
}

