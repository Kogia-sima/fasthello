use itoap::Integer;
use std::cmp;
use std::fs::File;
use std::io::{self, stdout, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::ptr;

const MIN_CAPACITY: usize = 40;

struct BufWriter<W: Write> {
    inner: W,
    buf: Vec<u8>,
    panicked: bool,
}

impl<W: Write> BufWriter<W> {
    #[inline]
    pub fn with_capacity(mut capacity: usize, inner: W) -> BufWriter<W> {
        capacity = cmp::max(MIN_CAPACITY, capacity);

        BufWriter {
            inner,
            buf: Vec::with_capacity(capacity),
            panicked: false,
        }
    }

    #[cold]
    #[inline(never)]
    fn write_slow(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.flush_buf()?;

        if buf.len() <= self.buf.capacity() {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        } else {
            self.panicked = true;
            let r = self.inner.write(buf);
            self.panicked = false;
            r
        }
    }

    fn flush_buf(&mut self) -> io::Result<()> {
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }

        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }

            /// The unwritten part of the buffer
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }

            /// Flag some bytes as removed from the front of the buffer
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }

            /// true if all of the bytes have been written
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    if self.done() {
                        self.buffer.clear();
                    } else {
                        self.buffer.drain(..self.written);
                    }
                }
            }
        }

        let mut guard = BufGuard::new(&mut self.buf);
        let inner = &mut self.inner;
        while !guard.done() {
            self.panicked = true;
            let r = inner.write(guard.remaining());
            self.panicked = false;

            match r {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.consume(n),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_int<I: Integer>(&mut self, value: I) -> Result<usize, io::Error> {
        if self.buf.len() + I::MAX_LEN <= self.buf.capacity() {
            unsafe {
                let dst = self.buf.as_mut_ptr().add(self.buf.len());
                let l = itoap::write_to_ptr(dst, value);
                self.buf.set_len(self.buf.len() + l);
                Ok(l)
            }
        } else {
            self.write_int_slow(value)
        }
    }

    #[cold]
    #[inline(never)]
    fn write_int_slow<I: Integer>(&mut self, value: I) -> Result<usize, io::Error> {
        self.flush_buf()?;

        unsafe {
            let dst = self.buf.as_mut_ptr();
            let l = itoap::write_to_ptr(dst, value);
            self.buf.set_len(l);
            Ok(l)
        }
    }
}

impl<W: Write> Write for BufWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() <= self.buf.capacity() - self.buf.len() {
            unsafe {
                let old_len = self.buf.len();
                let buf_len = buf.len();
                let src = buf.as_ptr();
                let dst = self.buf.as_mut_ptr().add(old_len);
                ptr::copy_nonoverlapping(src, dst, buf_len);
                self.buf.set_len(old_len + buf_len);
            }
            Ok(buf.len())
        } else {
            self.write_slow(buf)
        }
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.write(buf).map(|_| {})
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.inner.flush())
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            let _ = self.flush_buf();
        }
    }
}

fn main() {
    let stdout = AsRawFd::as_raw_fd(&stdout());
    let stdout: File = unsafe { FromRawFd::from_raw_fd(stdout) };
    let mut buf = BufWriter::with_capacity(32 * 1024, stdout);

    buf.write_all(b"Hello, 0").unwrap();

    for i in 0..10_000_000_u32 {
        buf.write_all(b"\tHello, ").unwrap();
        buf.write_int(i).unwrap();
    }

    buf.write_all(b"\t").unwrap();
}
