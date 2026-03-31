pub use std::io;
pub use std::io::{Error,ErrorKind,Write};
use std::marker::PhantomData;


pub struct Coder<E: Encoding, D: Dir, W: Write> {
    pub inner: W,
    pub state: D::State<E>,
    pub mode: D::Mode,
    pub _marker: PhantomData<(E, D)>,
}
pub type Encoder<E, W> = Coder<E, Enc, W>;
pub type Decoder<E, W> = Coder<E, Dec, W>;

mod sealed { pub trait Sealed {} }
pub trait Dir: sealed::Sealed + Sized {
    type Mode: Default;
    type State<E: Encoding>: Default;
    fn code_u8<E: Encoding, W: Write>(_: &mut Coder<E, Self, W>, _: u8) -> io::Result<()>;
    fn finish<E: Encoding, W: Write>(_: &mut Coder<E, Self, W>) -> io::Result<()>;
}

pub struct Enc;
impl sealed::Sealed for Enc {}
impl Dir for Enc {
    type State<E: Encoding> = E::EncState;
    type Mode = ();
    #[inline]
    fn code_u8<E: Encoding, W: Write>(co: &mut Encoder<E, W>, b: u8) -> io::Result<()> {
        E::encode_u8(co, b)
    }
    #[inline]
    fn finish<E: Encoding, W: Write>(co: &mut Encoder<E, W>) -> io::Result<()> {
        E::finish_encode(co)
    }
}

pub struct Dec;
impl sealed::Sealed for Dec {}
impl Dir for Dec {
    type State<E: Encoding> = E::DecState;
    type Mode = DecMode;
    #[inline]
    fn code_u8<E: Encoding, W: Write>(co: &mut Decoder<E, W>, b: u8) -> io::Result<()> {
        E::decode_chr(co, b)
    }
    #[inline]
    fn finish<E: Encoding, W: Write>(co: &mut Decoder<E, W>) -> io::Result<()> {
        // In current designs, finishing never does anything.
        E::finish_decode(co)
    }
}

pub trait Encoding: Sized {
    type EncState: Default;
    type DecState: Default;
    const CHUNK_SIZE: usize;
    const ENC_CHUNK_SIZE: usize;
    const ALPHABET: &[u8];
    const REV_EXTRAS: &[(u8, u8)] = &[];
    const REV_ALPHABET: [u8; 256] = {
        let mut rev = [0xff; 256];
        let mut i = 0;
        while i < Self::ALPHABET.len() {
            rev[Self::ALPHABET[i] as usize] = i as u8;
            i += 1;
        }
        let ex = Self::REV_EXTRAS;
        i = 0;
        while i < ex.len() {
            rev[ex[i].0 as usize] = ex[i].1;
            i += 1;
        }
        rev
    };

    #[inline]
    fn chr<T: Into<usize>>(i: T) -> u8 { Self::ALPHABET[i.into()] }
    #[inline]
    fn bits<T: Into<usize>>(i: T) -> u8 { Self::REV_ALPHABET[i.into()] }

    #[inline]
    fn encoded_size(size: usize) -> usize {
        usize::div_ceil(size * Self::ENC_CHUNK_SIZE, Self::CHUNK_SIZE)
    }
    #[inline]
    fn decoded_size(size: usize) -> usize {
        (size * Self::CHUNK_SIZE) / Self::ENC_CHUNK_SIZE
    }

    fn encode_u8<W: Write>(_: &mut Encoder<Self, W>, b: u8) -> io::Result<()>;
    fn decode_u8<W: Write>(_: &mut Decoder<Self, W>, b: u8) -> io::Result<()>;
    #[inline]
    fn decode_chr<W: Write>(co: &mut Decoder<Self, W>, c: u8) -> io::Result<()> {
        let v = Self::bits(c);
        if v == 0xff && co.mode != DecMode::None {
            if co.mode == DecMode::Loose
                || (co.mode == DecMode::Whitespace
                    && c.is_ascii_whitespace()) {
                return Ok(());
            }
            return Err(Error::new(ErrorKind::InvalidData,
                format!("Invalid byte: `{}`", c.escape_ascii())));
        }
        Self::decode_u8(co, v)
    }
    fn finish_encode<W: Write>(_: &mut Encoder<Self, W>) -> io::Result<()>;
    #[inline]
    fn finish_decode<W: Write>(_: &mut Decoder<Self, W>) -> io::Result<()> { Ok(()) }

    // fn has_leftover<W: Write>(_: &Decoder<Self, W>) -> bool { false }

    #[inline]
    fn new_encoder<W: Write>(w: W) -> Encoder<Self, W> { Coder::new(w, ()) }
    #[inline]
    fn new_decoder_default<W: Write>(w: W) -> Decoder<Self, W> {
        Coder::new(w, Default::default())
    }
    #[inline]
    fn new_decoder<W: Write>(w: W, m: DecMode) -> Decoder<Self, W> {
        Coder::new(w, m)
    }

    #[cfg(test)]
    // Filter out edge cases where max partial dec fails.
    fn partial_dec_filter(_buf: &[u8]) -> bool { true }

    fn encode_slice(d: &[u8]) -> String {
        let v = Vec::with_capacity(Self::encoded_size(d.len()));
        let mut co = Self::new_encoder(v);
        for b in d { co.write_one(*b).unwrap() }
        let e = co.into_inner();
        String::from_utf8(e).unwrap()
    }

    fn decode_str(e: &str) -> Vec<u8> {
        let d = Vec::with_capacity(Self::decoded_size(e.len()));
        let mut co = Self::new_decoder_default(d);
        for b in e.as_bytes() { co.write_one(*b).unwrap() }
        co.into_inner()
    }
}

#[macro_export]
macro_rules! chrs { ($($e:expr),*) => { &[$(Self::chr($e)),*][..] }; }

impl<E: Encoding, D: Dir, W: Write> Coder<E, D, W> {
    fn new(inner: W, mode: D::Mode) -> Self {
        Coder {
            inner, mode,
            state: Default::default(),
            _marker: Default::default(),
        }
    }
    pub fn into_inner(mut self) -> W {
        let _ = self.finish();
        let me = std::mem::ManuallyDrop::new(self);
        unsafe { std::ptr::read(&me.inner) }
    }
    #[inline]
    pub fn write_one(&mut self, b: u8) -> io::Result<()> {
        D::code_u8::<E, W>(self, b)
    }
    #[inline]
    fn finish(&mut self) -> io::Result<()> {
        D::finish::<E, W>(self)
    }
}

impl<E: Encoding, D: Dir, W: Write> Write for Coder<E, D, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &b in buf { D::code_u8::<E, W>(self, b)? }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy,Default)]
pub enum DecMode {
    Strict,      // error on invalid byte
    #[default]
    Whitespace,  // ignore whitespace, error on other invalid byte
    Loose,       // ignore all invalid bytes
    None,        // no filtering; invalid bytes pass through with undefined results
}

impl<E: Encoding, D: Dir, W: Write> Drop for Coder<E, D, W> {
    #[inline] fn drop(&mut self) {
        let _ = self.finish();
    }
}

