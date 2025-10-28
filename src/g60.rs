/* G60 encoding */

use crate::base::Encoding;
use std::cmp;

pub struct G60;

const CO_0: u128 = 14 * 60_u128.pow(9);
const CO_1: u128 = 3 * 60_u128.pow(8);
const CO_2: u128 = 40 * 60_u128.pow(6);
const CO_3: u128 = 3 * 60_u128.pow(5);
const CO_4: u128 = 2 * 60_u128.pow(4);
const CO_5: u128 = 24 * 60_u128.pow(2);
const CO_6: u128 = 5 * 60;

impl Encoding for G60 {
    const CHUNK_SIZE: usize = 8;
    const ENC_CHUNK_SIZE: usize = 11;
    const ALPHABET: &'static [u8]
        = b"0123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    fn encode_chunk_raw(d: &[u8], e: &mut [u8]) -> usize {
        let g = |i| *d.get(i).unwrap_or(&0) as u128;
        let s = 60_u128;
        let mut v: u128
            = CO_0 * g(0) + CO_1 * g(1) + CO_2 * g(2)
                + CO_3 * (16 * (g(3) >> 7) + 3 * g(3)) + CO_4 * g(4)
                + CO_5 * g(5) + CO_6 * g(6) + g(7);
        let el = Self::encoded_size(cmp::min(d.len(), 8));
        for i in (0..11).rev() {
            if i < el { e[i] = Self::chr((v % s) as usize) }
            v /= s;
        }
        el
    }

    fn decode_chunk_raw(e: &[u8], d: &mut [u8]) -> usize {
        let el = cmp::min(e.len(), 11);
        let dl = Self::decoded_size(el);
        let v: u128
            = (0..11).fold(0, |a, i| a * 60
                + if i < el { Self::bits(e[i]) as u128 } else { 0 });
        let mut w = |i, b| if i < dl { d[i] = b as u8 };
        w(0, v / CO_0);
        w(1, (v % CO_0) / CO_1);
        w(2, (v % CO_1) / CO_2);
        let x3 = (v % CO_2) / CO_3;
        w(3, (if x3 >= 400 { x3 - 16 } else { x3 }) / 3);
        w(4, ((v % (400*CO_3)) % (3*CO_3)) / CO_4);
        w(5, (v % CO_4) / CO_5);
        w(6, (v % CO_5) / CO_6);
        w(7, v % CO_6);
        dl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples() {
        assert_eq!(G60::encode_slice(b"Hello, world!"), "Gt4CGFiHehzRzjCF16");
        assert_eq!(G60::encode_slice(b"Hella, would???"), "Gt4CGFEHehzRzsCF26RHF");
        assert_eq!(G60::encode_slice(&[
                0x24, 0x3f, 0x6a, 0x88, 0x85, 0xa3, 0x08, 0xd3,
                0x13, 0x19, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x44,
                0xa4, 0x09, 0x38, 0x22, 0x29, 0x9f, 0x31, 0xd0,
                0x08, 0x2e, 0xfa, 0x98, 0xec, 0x4e, 0x6c, 0x89_u8,
            ][..]), "8TAB1GT5CjX4TGY6u6kxc8eGTdR7P3g8U1uLn3jsXM2H");
    }

    crate::stock_tests!(G60);
}

