use std::fs::File;
use std::io::{stdout, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::ptr;

const BUFSIZ: usize = 32 * 1024;
const DIGITS: usize = 12;

/// memcpy implementation based on glibc (https://github.molgen.mpg.de/git-mirror/glibc/blob/master/sysdeps/x86_64/multiarch/memcpy-avx-unaligned.S)
#[allow(clippy::cast_ptr_alignment)]
pub unsafe fn memcpy_16(src: *const u8, dst: *mut u8, len: usize) {
    debug_assert!(len <= 16);
    let len_u8 = len as u8;

    if len_u8 >= 8 {
        let offset = len - 8;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u64);
        let t1 = ptr::read_unaligned(src as *const u64);
        ptr::write_unaligned(dst.add(offset) as *mut u64, t2);
        ptr::write_unaligned(dst as *mut u64, t1);
    } else if len_u8 >= 4 {
        let offset = len - 4;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u32);
        let t1 = ptr::read_unaligned(src as *const u32);
        ptr::write_unaligned(dst.add(offset) as *mut u32, t2);
        ptr::write_unaligned(dst as *mut u32, t1);
    } else if len_u8 >= 2 {
        let offset = len - 2;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u16);
        let t1 = ptr::read_unaligned(src as *const u16);
        ptr::write_unaligned(dst.add(offset) as *mut u16, t2);
        ptr::write_unaligned(dst as *mut u16, t1);
    } else if len_u8 >= 1 {
        *dst = *src;
    }
}

#[inline(always)]
fn increase_str_num(input: &mut [u8; DIGITS]) -> usize {
    input[DIGITS - 1] += 1;
    if input[DIGITS - 1] < b'0' + 10 {
        return 0;
    }

    input[DIGITS - 1] = b'0';
    let mut i = DIGITS - 2;

    while i > 0 {
        input[i] += 1;
        if input[i] - b'0' < 10 {
            return DIGITS - i;
        }
        input[i] = b'0';
        i -= 1;
    }

    unreachable!("overflow input");
}

fn main() {
    let stdout = AsRawFd::as_raw_fd(&stdout());
    let mut stdout: File = unsafe { FromRawFd::from_raw_fd(stdout) };
    let mut buffer = [0u8; BUFSIZ];
    let buf0p: *mut u8 = &mut buffer[0];
    let mut bufp = buf0p;
    let bufep = unsafe { buf0p.add(BUFSIZ) };
    let mut num = [b'0'; DIGITS];
    let mut nump: *const u8 = &num[DIGITS - 1];
    let mut num_len = 1;
    let prefix = b"\tHello, ";
    let prefixp: *const u8 = &prefix[0];
    let nprefix = prefix.len();
    let line_max_len = prefix.len() + DIGITS + 1;
    for _ in 0..10_000_000 {
        unsafe {
            prefixp.copy_to_nonoverlapping(bufp, nprefix);
            bufp = bufp.add(nprefix);
            memcpy_16(nump, bufp, num_len);
            bufp = bufp.add(num_len);
        }
        let l = increase_str_num(&mut num);
        if l > num_len {
            num_len = l;
            nump = &num[DIGITS - num_len];
        }
        let remain_len = bufep as usize - bufp as usize;
        if line_max_len > remain_len {
            let _ = stdout.write(&buffer[..BUFSIZ - remain_len]).unwrap();
            bufp = buf0p;
        }
    }
    let fill_len = bufp as usize - buf0p as usize;
    let _ = stdout.write(&buffer[..fill_len]).unwrap();
}
