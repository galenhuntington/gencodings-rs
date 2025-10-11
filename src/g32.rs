/* Crockford base-32 encoding */

use crate::base::Encoding;

pub struct G32;

impl Encoding for G32 {
    const CHUNK_SIZE: usize = 5;
    const ENC_CHUNK_SIZE: usize = 8;
    const ALPHABET: &'static [u8] = b"0123456789abcdefghjkmnpqrstvwxyz";

    fn encode_chunk_raw(d: &[u8], e: &mut [u8]) -> usize {
        let dl = d.len();
        if dl == 0 { return 0; }
        e[0] = Self::chr(d[0] >> 3);
        let x = (d[0] & 0x07) << 2;
        if dl == 1 { e[1] = Self::chr(x); return 2; }
        e[1] = Self::chr(x | (d[1] >> 6));
        e[2] = Self::chr((d[1] & 0x3f) >> 1);
        let x = (d[1] & 0x01) << 4;
        if dl == 2 { e[3] = Self::chr(x); return 4; }
        e[3] = Self::chr(x | (d[2] >> 4));
        let x = (d[2] & 0x0f) << 1;
        if dl == 3 { e[4] = Self::chr(x); return 5; }
        e[4] = Self::chr(x | (d[3] >> 7));
        e[5] = Self::chr((d[3] & 0x7f) >> 2);
        let x = (d[3] & 0x03) << 3;
        if dl == 4 { e[6] = Self::chr(x); return 7; }
        e[6] = Self::chr(x | (d[4] >> 5));
        e[7] = Self::chr(d[4] & 0x1f);
        return 8;
    }

    fn decode_chunk_raw(e: &[u8], d: &mut [u8]) -> usize {
        let el = e.len();
        if el <= 1 { return 0; }
        let e1 = Self::bits(e[1]);
        d[0] = Self::bits(e[0]) << 3 | e1 >> 2;
        if el <= 3 { return 1; }
        let e3 = Self::bits(e[3]);
        d[1] = e1 << 6 | Self::bits(e[2]) << 1 | e3 >> 4;
        if el == 4 { return 2; }
        let e4 = Self::bits(e[4]);
        d[2] = e3 << 4 | e4 >> 1;
        if el <= 6 { return 3; }
        let e6 = Self::bits(e[6]);
        d[3] = e4 << 7 | Self::bits(e[5]) << 2 | e6 >> 3;
        if el == 7 { return 4; }
        d[4] = e6 << 5 | Self::bits(e[7]);
        return 5;
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
    }

    crate::stock_tests!(G32);
}

