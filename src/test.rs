use rand::Rng;
use std::io::{Write,ErrorKind};
use rand::distr::Distribution;

use crate::base::*;
// use std::io::ErrorKind;

pub struct Bytes;

impl Distribution<Vec<u8>> for Bytes {
    fn sample<'a, R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<u8> {
        let len = rng.random_range(0..60);
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
    }
}

pub fn test_dec_modes<E: Encoding>() {
    let sample = {
        let mut s = b"_\t \n\r\x01_".to_vec();
        s[0] = E::ALPHABET[0];
        s[4] = E::ALPHABET[1];
        s
    };
    let mut buf = vec![0u8; 2];
    let err = E::new_decoder(&mut buf, DecMode::Strict).write(&sample)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(err.to_string(), "Invalid byte: `\\t`");
    let err = E::new_decoder(&mut buf, DecMode::Whitespace).write(&sample)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(err.to_string(), "Invalid byte: `\\x01`");
    assert!(E::new_decoder(&mut buf, DecMode::Loose).write(&sample).is_ok());
}

pub fn test_max_partial_enc<E: Encoding>(num: usize) {
    let mut rng = rand::rng();
    for _ in 0..num {
        let mut d: Vec<u8> = Bytes.sample(&mut rng);
        let len = d.len();
        let written = {
            let mut e = Vec::new();
            let mut co = std::mem::ManuallyDrop::new(E::new_encoder(&mut e));
            let _ = co.write(&d);
            e.len()
        };
        d.append(&mut vec!(0x00; 60));
        let e1 = E::encode_slice(&d);
        let e1 = e1.as_bytes();
        d.truncate(len);
        d.append(&mut vec!(0xff; 60));
        // TODO factor this
        let e2 = E::encode_slice(&d);
        let e2 = e2.as_bytes();
        let split = (0..).find(|i| e1[*i] != e2[*i]).unwrap();
        d.truncate(len);
        assert_eq!(written, split, "{:?}", d);
    }
}

pub fn test_max_partial_dec<E: Encoding>(num: usize) {
    let mut rng = rand::rng();
    for _ in 0..num {
        let mut d: Vec<u8> = Bytes.sample(&mut rng);
        let len = d.len();
        if len == 0 || d[0] == 0x00 || d[0] == 0xff { continue }
        let mut d2 = d.clone();
        d.append(&mut Bytes.sample(&mut rng));
        d.append(&mut vec!(0x7f; 60));
        let e_mid = E::encode_slice(&d);
        let e_mid = e_mid.as_bytes();
        d.truncate(len);
        // Decrement
        for i in (d2.len()..0).rev() {
            if d2[i] > 0x00 { d2[i] -= 1; break }
            d2[i] = 0xff;
        }
        d2.append(&mut vec!(0xff; 60));
        let e_lo = E::encode_slice(&d2);
        let e_lo = e_lo.as_bytes();
        // Increment
        let mut d2 = d.clone();
        for i in (d2.len()..0).rev() {
            if d2[i] < 0xff { d2[i] += 1; break }
            d2[i] = 0x00;
        }
        d2.append(&mut vec!(0x00; 60));
        let e_hi = E::encode_slice(&d2);
        let e_hi = e_hi.as_bytes();
        let split = (0..).find(|&i| e_lo[i] != e_mid[i] && e_mid[i] != e_hi[i])
            .unwrap();
        let mut d_new = Vec::new();
        let mut co = std::mem::ManuallyDrop::new(
            E::new_decoder(&mut d_new, DecMode::Strict));
        let _ = co.write(&e_mid[0..=split]);
        assert!(d_new.len() >= len, "{:?}", d);
    }
}

macro_rules! if_ {
    (true, { $($code:tt)* }) => { $($code)* };
    (false, { $($_:tt)* }) => { };
}
pub(crate) use if_;

#[macro_export]
macro_rules! stock_tests {
    ($e:ty) => {
        #[test]
        fn test_encode_decode() {
            $crate::test::test_encode_decode::<$e>(5000);
        }
        #[test]
        fn test_dec_modes() {
            $crate::test::test_dec_modes::<$e>();
        }
    };
}

#[macro_export]
macro_rules! max_partial_tests {
    ($e:ty, $enc:ident, $dec:ident) => {
        crate::test::if_!($enc, {
            #[test]
            fn test_max_partial_enc() {
                $crate::test::test_max_partial_enc::<$e>(5000);
            }
        });
        crate::test::if_!($dec, {
            #[test]
            fn test_max_partial_dec() {
                $crate::test::test_max_partial_dec::<$e>(5000);
            }
        });
    };
}

