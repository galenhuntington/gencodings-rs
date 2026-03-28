/* G86 encoding */

use crate::base::*;

pub struct G86;

const THREES: [u64; 6] = [81, 27, 9, 3, 1, 1];

#[derive(PartialEq,Eq,Default,Debug)]
pub struct G86EState { hold: u64, cnt: u8 }
#[derive(PartialEq,Eq,Default,Debug)]
pub struct G86DState { off: u8, rem: u8 }

fn write_num<W: Write>(w: &mut W, hold: u64, cnt: u8) -> io::Result<()> {
    let c = cnt as usize;
    let mut a = [0; 5];
    let mut n = hold * THREES[c];
    for i in (0..=c).rev() {
        let r = n % 86;
        n /= 86;
        a[i] = G86::chr(r as usize);
    }
    w.write_all(&a[0..=c])
}

impl Encoding for G86 {
    const CHUNK_SIZE: usize = 4;
    const ENC_CHUNK_SIZE: usize = 5;
    const ALPHABET: &'static [u8]
        = b"!#$%()*+-./0123456789:=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ\
            []^_`abcdefghijklmnopqrstuvwxyz{|}~";
    type EncState = G86EState;
    type DecState = G86DState;

    fn encode_u8<W: Write>(co: &mut Encoder<G86, W>, b: u8) -> io::Result<()> {
        let cnt = co.state.cnt;
        let hold = co.state.hold*258 + b as u64;
        if cnt < 3 {
            co.state = G86EState { hold, cnt: cnt + 1 };
            Ok(())
        } else {
            co.state = Default::default();
            write_num(&mut co.inner, hold, 4)
        }
    }

    fn finish_encode<W: Write>(co: &mut Encoder<G86, W>) -> io::Result<()> {
        match co.state.cnt {
            0   => Ok(()),
            cnt => write_num(&mut co.inner, co.state.hold, cnt),
        }
    }

    fn decode_u8<W: Write>(co: &mut Decoder<G86, W>, b: u8) -> io::Result<()> {
        let off = co.state.off;
        let e = THREES[off as usize] as u16;
        let v = co.state.rem as u16 * 86 + b as u16;
        let (q, r) = (v / e, v % e);
        co.state = G86DState { rem: r as u8, off: (off + 1) % 5 };
        if off != 0 {
            co.inner.write_all(&[q as u8])
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples() {
        assert_eq!(G86::encode_slice(b"Hello, world!"), "=g]l=Jv^0IJ}|l3/G");
        assert_eq!(G86::decode_str("=g]l=Jv^0IJ}|l3/G"), b"Hello, world!");
        assert_eq!(G86::encode_slice(&[3, 47, 200, 171]), "!~~~~");
        assert_eq!(G86::encode_slice(&[3, 47, 200, 172]), "#!!!!");
        assert_eq!(G86::encode_slice(&[
                0x24, 0x3f, 0x6a, 0x88, 0x85, 0xa3, 0x08, 0xd3,
                0x13, 0x19, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x44,
                0xa4, 0x09, 0x38, 0x22, 0x29, 0x9f, 0x31, 0xd0,
                0x08, 0x2e, 0xfa, 0x98, 0xec, 0x4e, 0x6c, 0x89,
            ][..]), "0H_fZQ{)BO)~boV#*k#m[R{{J2)ahL$Xwhks56l[");
    }

    crate::stock_tests!(G86);
    crate::max_partial_tests!(G86, false, true);
}

