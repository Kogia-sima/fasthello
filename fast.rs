use std::io::Write;

fn main() {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut buf = std::io::BufWriter::with_capacity(32 * 1024, lock);
    for i in 0..10_000_000 {
        let _ = buf.write(b"Hello, ").unwrap();
        let _ = itoa::write(&mut buf, i).unwrap();
        let _ = buf.write(b"\t").unwrap();
    }
}
