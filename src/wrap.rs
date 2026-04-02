use std::io::{Result,Write};
use std::cmp::min;

pub struct WrapWidth<W> {
    inner: W,
    width: usize,
    cur: usize,
}

impl<W> WrapWidth<W> {
    pub fn new(inner: W, width: usize) -> Self {
        WrapWidth { inner, width, cur: 0 }
    }
    pub fn into_inner(self) -> W { self.inner }
    pub fn width(&self) -> usize { self.width }
}

impl<W: Write> Write for WrapWidth<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut ix = 0;
        while ix < buf.len() {
            if self.width > 0 && self.cur >= self.width {
                self.inner.write_all(b"\n")?;
                self.cur = 0;
            }
            let left = buf.len() - ix;
            let writing =
                if self.width > 0 { min(left, self.width - self.cur) }
                else { left };
            let start = ix;
            ix += writing;
            self.inner.write_all(&buf[start .. ix])?;
            self.cur += writing;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> { self.inner.flush() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn test_wrap_width() {
        let mut buf = Vec::new();
        {
            let mut w = WrapWidth::new(&mut buf, 4);
            w.write_all(b"Hello, world").unwrap();
            w.flush().unwrap();
        }
        assert_eq!(buf, b"Hell\no, w\norld");
    }
}

