/* Crockford base-32 encoding */

use crate::base::*;
use crate::chrs;

pub struct G32;

#[derive(PartialEq,Eq,Default,Debug)]
pub struct G32State { cnt: u8, bits: u8 }

const UPPER_ARR: [(u8, u8); 22] = {
    let mut v = [(0, 0); 22];
    let mut i = 10;
    while i < 32 {
        v[i - 10] = (G32::ALPHABET[i] - 32, i as u8);
        i += 1;
    }
    v
};

impl Encoding for G32 {
    const CHUNK_SIZE: usize = 5;
    const ENC_CHUNK_SIZE: usize = 8;
    const ALPHABET: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";
    const REV_EXTRAS: &[(u8, u8)] = &UPPER_ARR;
    type EncState = G32State;
    type DecState = G32State;

    fn encode_u8<W: Write>(co: &mut Encoder<G32, W>, b: u8) -> io::Result<()> {
        let cnt = co.state.cnt;
        let c1 = co.state.bits | b >> (cnt + 3);
        if cnt < 2 {
            co.state = G32State {
                cnt: cnt + 3,
                bits: (b << (2 - cnt)) & 0x1f,
            };
            co.inner.write_all(chrs!(c1))
        } else {
            co.state = G32State {
                cnt: cnt - 2, 
                bits: (b << (7 - cnt)) & 0x1f,
            };
            co.inner.write_all(chrs!(c1, (b >> (cnt - 2)) & 0x1f))
        }
    }

    fn finish_encode<W: Write>(co: &mut Encoder<G32, W>) -> io::Result<()> {
        if co.state.cnt != 0 {
            co.inner.write_all(chrs!(co.state.bits))
        } else {
            Ok(())
        }
    }

    fn decode_u8<W: Write>(co: &mut Decoder<G32, W>, b: u8) -> io::Result<()> {
        let G32State { cnt, bits } = co.state;
        if cnt < 3 {
            co.state = G32State {
                cnt: cnt + 5,
                bits: bits | b << (3 - cnt),
            };
            Ok(())
        } else {
            co.state = G32State {
                cnt: cnt - 3,
                bits: if cnt == 3 { 0 } else { b << (11 - cnt) },
            };
            co.inner.write_all(&[bits | b >> (cnt - 3)])
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroes() {
        assert_eq!(G32::encode_slice(b""), "");
        assert_eq!(G32::encode_slice(b"\0"), "00");
        assert_eq!(G32::encode_slice(b"\0\0"), "0000");
        assert_eq!(G32::encode_slice(b"\0\0\0"), "00000");
        assert_eq!(G32::encode_slice(b"\0\0\0\0"), "0000000");
        assert_eq!(G32::encode_slice(b"\0\0\0\0\0"), "00000000");
        assert_eq!(G32::encode_slice(b"\0\0\0\0\0\0"), "0000000000");
    }

    #[test]
    fn test_ones() {
        assert_eq!(G32::encode_slice(b"\xff"), "zw");
        assert_eq!(G32::encode_slice(b"\xff\xff"), "zzzg");
        assert_eq!(G32::encode_slice(b"\xff\xff\xff"), "zzzzy");
        assert_eq!(G32::encode_slice(b"\xff\xff\xff\xff"), "zzzzzzr");
        assert_eq!(G32::encode_slice(b"\xff\xff\xff\xff\xff"), "zzzzzzzz");
    }

    #[test]
    fn test_hello() {
        assert_eq!(G32::encode_slice(b"Hello"), "91jprv3f");
        assert_eq!(G32::decode_str("91jprv3f"), b"Hello");
        assert_eq!(G32::decode_str("91JPRV3F"), b"Hello"); // Upper case
    }

    crate::stock_tests!(G32);
    crate::max_partial_tests!(G32, true, true);
}

