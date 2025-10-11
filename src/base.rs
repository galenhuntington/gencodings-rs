use std::io::{Result,Read,Write,Error,ErrorKind};
use std::marker::PhantomData;

pub struct Ratio (pub usize, pub usize);

pub trait Encoding {
    const CHUNK_SIZE: usize;
    const ENC_CHUNK_SIZE: usize;
    const ALPHABET: &'static [u8];
    const REV_ALPHABET: [u8; 256] = {
        let mut rev = [0xff; 256];
        let mut i = 0;
        while i < Self::ALPHABET.len() {
            rev[Self::ALPHABET[i] as usize] = i as u8;
            i += 1;
        }
        rev
    };

    fn chr<T: Into<usize>>(i: T) -> u8 { Self::ALPHABET[i.into()] }
    fn bits<T: Into<usize>>(i: T) -> u8 { Self::REV_ALPHABET[i.into()] }
    #[inline(always)]
    fn encoded_size(size: usize) -> usize {
        usize::div_ceil(size * Self::ENC_CHUNK_SIZE, Self::CHUNK_SIZE)
    }
    #[inline(always)]
    fn decoded_size(size: usize) -> usize {
        (size * Self::CHUNK_SIZE) / Self::ENC_CHUNK_SIZE
    }

    // It is assumed the receiving slice has enough space.
    fn encode_chunk_raw(d: &[u8], e: &mut [u8]) -> usize;
    fn decode_chunk_raw(e: &[u8], d: &mut [u8]) -> usize;

    // fn decode_chunk_raw(&self, e: &[u8], d: &mut [u8]);
    fn encode_slice(d: &[u8]) -> String {
        let enc_size = Self::encoded_size(d.len());
        let mut e = vec![0; enc_size];
        for (i, chunk) in d.chunks(Self::CHUNK_SIZE).enumerate() {
            let enc_chunk = &mut e[i * Self::ENC_CHUNK_SIZE ..];
            Self::encode_chunk_raw(chunk, enc_chunk);
        }
        unsafe { String::from_utf8_unchecked(e) }
    }

    fn decode_str(e: &str) -> Vec<u8> {
        let dec_size = Self::decoded_size(e.len());
        let mut d = vec![0; dec_size];
        for (i, chunk) in e.as_bytes().chunks(Self::ENC_CHUNK_SIZE).enumerate() {
            let dec_chunk = &mut d[i * Self::CHUNK_SIZE ..];
            Self::decode_chunk_raw(chunk, dec_chunk);
        }
        d
    }

    fn encode<R: Read, W: Write>(r: &mut R, w: &mut W) -> Result<usize> {
        let mut len = 0;
        let mut ibuf = vec![0; Self::CHUNK_SIZE];
        let mut obuf = vec![0; Self::ENC_CHUNK_SIZE];
        loop {
            let ct = r.read(&mut ibuf)?;
            if ct == 0 { break; }
            let out = Self::encode_chunk_raw(&ibuf[..ct], &mut obuf);
            w.write(&obuf[..out])?;
            len += ct;
        }
        Ok(len)
    }

    fn decode<R: Read, W: Write>(r: &mut R, w: &mut W) -> Result<usize> {
        let mut len = 0;
        let mut ibuf = vec![0; Self::ENC_CHUNK_SIZE];
        let mut obuf = vec![0; Self::CHUNK_SIZE];
        loop {
            let ct = r.read(&mut ibuf)?;
            if ct == 0 { break; }
            let out = Self::decode_chunk_raw(&ibuf[..ct], &mut obuf);
            w.write(&obuf[..out])?;
            len += ct;
        }
        Ok(len)
    }

    fn decode_valid<R: Read, W: Write>(r: &mut R, w: &mut W, ftype: FilterType) -> Result<usize>
            where Self: Sized {
        Self::decode(&mut EncodedFilter::<_, Self>::new(r, ftype), w)
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum FilterType {
    Strict,      // error on invalid byte
    Whitespace,  // ignore whitespace, error on other invalid byte
    Loose,       // ignore all invalid bytes
    None,        // no filtering; invalid bytes pass through with undefined results
}

pub struct EncodedFilter<R: Read, E: Encoding> {
    base: R,
    encoding: PhantomData<E>,
    ftype: FilterType,
}

impl<R: Read, E: Encoding> EncodedFilter<R, E> {
    pub fn new(base: R, ftype: FilterType) -> Self {
        EncodedFilter { base, encoding: PhantomData, ftype }
    }
}

impl<R: Read, E: Encoding> Read for EncodedFilter<R, E> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.ftype == FilterType::None {
            return self.base.read(buf);
        }
        let cbuf = &mut [0u8];
        let mut ct = 0;
        loop {
            if ct == buf.len() { break; }
            let r = self.base.read(cbuf)?;
            if r == 0 { break; }
            let c = cbuf[0];
            let v = E::bits(c);
            if v == 0xff {
                if self.ftype == FilterType::Loose
                    || (self.ftype == FilterType::Whitespace
                        && c.is_ascii_whitespace()) {
                    continue;
                }
                return Err(Error::new(ErrorKind::InvalidData,
                    format!("Invalid byte: `{}`", c.escape_ascii())));
            } else {
                buf[ct] = c;
                ct += 1;
            }
        }
        Ok(ct)
    }
}

