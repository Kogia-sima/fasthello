use std::fs::File;
use std::io::{stdout, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};

fn main() {
    let stdout = AsRawFd::as_raw_fd(&stdout());
    let stdout: File = unsafe { FromRawFd::from_raw_fd(stdout) };
    let mut buf = std::io::BufWriter::with_capacity(32 * 1024, stdout);
    for i in 0..10_000_000 {
        let _ = buf.write(b"Hello, ").unwrap();
        let _ = itoa::write(&mut buf, i).unwrap();
        let _ = buf.write(b"\t").unwrap();
    }
}
