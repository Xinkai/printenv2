use crate::AppResult;
use std::mem::size_of;

#[allow(warnings)]
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
#[allow(clippy::nursery)]
#[allow(unaligned_references)]
mod apple_sysctl_bindings {
    include!(concat!(env!("OUT_DIR"), "/apple-sysctl-bindings.rs"));
}
mod oxidation {
    use super::apple_sysctl_bindings::{CTL_KERN, KERN_ARGMAX, KERN_PROCARGS2, sysctl};
    use super::{AppResult, size_of};

    use std::ptr::null_mut;

    pub fn argmax() -> AppResult<i32> {
        // Get max process args size.
        let mut mib = [CTL_KERN, KERN_ARGMAX];
        let mut maxarg: i32 = 0;
        let mut size: usize = size_of::<i32>();
        let ret = unsafe {
            sysctl(
                mib.as_mut_ptr().cast::<i32>(),
                2,
                std::ptr::addr_of_mut!(maxarg).cast::<std::ffi::c_void>(),
                std::ptr::addr_of_mut!(size),
                null_mut(),
                0,
            )
        };
        if ret == 0 {
            Ok(maxarg)
        } else {
            Err(std::io::Error::last_os_error().into())
        }
    }

    pub fn procargs2(pid: u32, buffer: &mut Vec<u8>) -> AppResult<()> {
        let mut mib = [CTL_KERN, KERN_PROCARGS2, pid];
        let mut size: usize = buffer.capacity();
        let err = unsafe {
            sysctl(
                mib.as_mut_ptr().cast::<i32>(),
                3,
                buffer.as_mut_ptr().cast::<std::ffi::c_void>(),
                &mut size,
                null_mut(),
                0,
            )
        };
        if err == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().into())
        }
    }
}

struct BufferWalker {
    data: Vec<u8>,
    cursor: usize,
}

impl BufferWalker {
    fn next_string_with_nul(&mut self) -> &[u8] {
        // Always starts at a non-null
        let start = self.cursor;
        let end = self.data[self.cursor..]
            .iter()
            .position(|c| *c == 0)
            .expect("finds end of NUL")
            + 1
            + start;
        self.cursor = self.data[end..]
            .iter()
            .position(|c| *c != 0)
            .expect("finds start")
            + end;
        &self.data[start..end]
    }

    fn next_string_with_nulnul(&mut self) -> &[u8] {
        let start = self.cursor;
        let end = self.data[self.cursor..]
            .windows(2)
            .position(|cc| cc == [0, 0])
            .expect("finds NULNUL")
            + 1
            + start;
        self.cursor = self.data[end..]
            .iter()
            .position(|c| *c != 0)
            .expect("finds start")
            + end;
        &self.data[start..end]
    }
}

pub fn get_environment_string(pid: u32) -> AppResult<Vec<u8>> {
    let argmax = oxidation::argmax()?;
    let mut buffer = vec![0; argmax.try_into()?];
    oxidation::procargs2(pid, &mut buffer)?;

    let arg_count = u32::from_ne_bytes(buffer[0..size_of::<u32>()].try_into()?);

    let mut bw = BufferWalker {
        data: buffer,
        cursor: size_of::<u32>(),
    };
    bw.next_string_with_nul();

    for _ in 0..arg_count {
        bw.next_string_with_nul();
    }

    let slice = bw.next_string_with_nulnul();

    Ok(slice.into())
}
