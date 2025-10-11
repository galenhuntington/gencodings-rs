use rand::Rng;
use rand::distr::Distribution;

use crate::base::*;
use std::io::Cursor;
use std::cmp;
use std::io::ErrorKind;

pub struct Bytes;

impl Distribution<Vec<u8>> for Bytes {
    fn sample<'a, R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<u8> {
        let len = rng.random_range(0..100);
        let mut bytes = vec![0u8; len];
        rng.fill(&mut bytes[..]);
        bytes
    }
}

pub fn test_encode_decode<E: Encoding>(num: usize) {
    let mut rng = rand::rng();
    for _ in 0..num {
        let d: Vec<u8> = Bytes.sample(&mut rng);
        let e = E::encode_slice(&d);
        let d2 = E::decode_str(&e);
        assert_eq!(d, d2, "Slice/str");
        assert_eq!(e.len(), E::encoded_size(d.len()), "Enc size");
        assert_eq!(d.len(), E::decoded_size(e.len()), "Dec size");
        let mut ebuf = Vec::new();
        let mut dbuf = Vec::new();
        E::encode(&mut Cursor::new(&d), &mut ebuf).unwrap();
        E::decode(&mut Cursor::new(&ebuf), &mut dbuf).unwrap();
        assert_eq!(d, dbuf, "Vec mismatch");
    }
}

// Tests that slices fill and do not overflow in either direction.
pub fn test_slice_lengths<E: Encoding>() {
    let ibuf = vec![E::ALPHABET[1]; E::ENC_CHUNK_SIZE + 3];
    for len in 0 ..= E::CHUNK_SIZE+2 {
        let enc_len = cmp::min(E::encoded_size(len), E::ENC_CHUNK_SIZE);
        let mut obuf = vec![0u8; enc_len];
        assert_eq!(E::encode_chunk_raw(&ibuf[..len], &mut obuf), enc_len, "Enc len");
        if enc_len > 0 && enc_len <= E::ENC_CHUNK_SIZE {
            assert_ne!(obuf[enc_len - 1], 0, "Full enc write");
        }
    }
    for len in 0 ..= E::ENC_CHUNK_SIZE+2 {
        let dec_len = cmp::min(E::decoded_size(len), E::CHUNK_SIZE);
        let mut obuf = vec![0u8; dec_len];
        assert_eq!(E::decode_chunk_raw(&ibuf[..len], &mut obuf), dec_len, "Dec len");
        if dec_len > 0 && dec_len <= E::CHUNK_SIZE {
            assert_ne!(obuf[dec_len - 1], 0, "Full dec write");
        }
    }
}

pub fn test_filters<E: Encoding>() {
    let sample = {
        let mut s = b"_\t \n\r\x01_".to_vec();
        s[0] = E::ALPHABET[0];
        s[4] = E::ALPHABET[1];
        s
    };
    let mut buf = vec![0u8; 2];
    let err = E::decode_valid(&mut sample.as_slice(), &mut buf, FilterType::Strict)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(err.to_string(), "Invalid byte: `\\t`");
    let err = E::decode_valid(&mut sample.as_slice(), &mut buf, FilterType::Whitespace)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(err.to_string(), "Invalid byte: `\\x01`");
    assert!(E::decode_valid(&mut sample.as_slice(), &mut buf, FilterType::Loose).is_ok());
}

pub fn run_stock_tests<E: Encoding>() {
    test_slice_lengths::<E>();
    test_encode_decode::<E>(1000);
}

#[macro_export]
macro_rules! stock_tests {
    ($e:ty) => {
        #[test]
        fn test_encode_decode() {
            crate::test::test_encode_decode::<$e>(2000);
        }
        #[test]
        fn test_slice_lengths() {
            crate::test::test_slice_lengths::<$e>();
        }
        #[test]
        fn test_filters() {
            crate::test::test_filters::<$e>();
        }
    };
}

