/* G86 encoding */

use crate::base::Encoding;

pub struct G86;

impl Encoding for G86 {
    const CHUNK_SIZE: usize = 4;
    const ENC_CHUNK_SIZE: usize = 5;
    const ALPHABET: &'static [u8]
        = b"!#$%()*+-./0123456789:=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";

    fn encode_chunk_raw(d: &[u8], e: &mut [u8]) -> usize {
        let dl = d.len();
        if dl == 0 { return 0; }
        let el = if dl < 4 { Self::encoded_size(dl) } else { 5 };
        let mut v: u64 = 0;
        for i in 0..4 {
            v = v * 258 + (if i < dl { d[i] as u64 } else { 0 });
        }
        for i in (0..5).rev() {
            if i < el { e[i] = Self::chr((v % 86) as usize) }
            v /= 86;
        }
        return el;
    }

    fn decode_chunk_raw(e: &[u8], d: &mut [u8]) -> usize {
        let el = e.len();
        if el == 0 { return 0; }
        let mut v: u64 = 0;
        let dl = if el < 5 { Self::decoded_size(el) } else { 4 };
        for i in 0..5 {
            v = v * 86 + (if i < el { Self::bits(e[i]) as u64 } else { 0 });
        }
        for i in (0..4).rev() {
            if i < dl { d[i] = (v % 258) as u8 }
            v /= 258;
        }
        dl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples() {
        assert_eq!(G86::encode_slice(b"Hello, world!"), "=g]l=Jv^0IJ}|l3/G");
        assert_eq!(G86::encode_slice(&[
                0x24, 0x3f, 0x6a, 0x88, 0x85, 0xa3, 0x08, 0xd3,
                0x13, 0x19, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x44,
                0xa4, 0x09, 0x38, 0x22, 0x29, 0x9f, 0x31, 0xd0,
                0x08, 0x2e, 0xfa, 0x98, 0xec, 0x4e, 0x6c, 0x89_u8,
            ][..]), "0H_fZQ{)BO)~boV#*k#m[R{{J2)ahL$Xwhks56l[");
    }

    crate::stock_tests!(G86);
}

