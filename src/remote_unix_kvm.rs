use crate::AppResult;

#[allow(warnings)]
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
#[allow(clippy::nursery)]
#[allow(clippy::cargo)]
mod kvm_bindings {
    include!(concat!(env!("OUT_DIR"), "/kvm-bindings.rs"));
}

mod oxidation {
    use super::AppResult;
    use super::kvm_bindings::{
        _POSIX2_LINE_MAX, KERN_PROC_PID, O_RDONLY, kinfo_proc, kvm_close, kvm_getenvv, kvm_geterr,
        kvm_getprocs, kvm_openfiles, kvm_t,
    };

    use crate::definition::AppError;
    use std::ffi::{CStr, CString};

    #[derive(Debug)]
    pub struct Kvm {
        kd: *mut kvm_t,
    }

    impl Kvm {
        pub fn try_new() -> AppResult<Self> {
            let mut errbuf = [0i8; _POSIX2_LINE_MAX as usize];

            let corefile = CString::new("/dev/null")?;
            let kd = unsafe {
                kvm_openfiles(
                    std::ptr::null(),
                    corefile.as_ptr(),
                    std::ptr::null(),
                    O_RDONLY.try_into()?,
                    errbuf.as_mut_ptr(),
                )
            };
            if kd.is_null() {
                Err(AppError::UnixErrorString(
                    unsafe { CStr::from_ptr(errbuf.as_ptr()) }.into(),
                ))
            } else {
                Ok(Self { kd })
            }
        }

        pub fn get_proc(&self, pid: u32) -> AppResult<*mut kinfo_proc> {
            let mut cnt = 1;
            let proc = unsafe {
                kvm_getprocs(
                    self.kd,
                    KERN_PROC_PID.try_into()?,
                    pid.try_into()?,
                    &mut cnt,
                )
            };
            if proc.is_null() {
                let errbuf = unsafe { kvm_geterr(self.kd) };
                Err(AppError::UnixErrorString(
                    unsafe { CStr::from_ptr(errbuf) }.into(),
                ))
            } else {
                assert_eq!(cnt, 1);
                Ok(proc)
            }
        }

        pub fn get_env(&self, proc: *mut kinfo_proc) -> AppResult<Vec<u8>> {
            let mut environ = unsafe { kvm_getenvv(self.kd, proc, 0) };
            if environ.is_null() {
                Err(std::io::Error::last_os_error().into())
            } else {
                let mut result = Vec::new();
                unsafe {
                    while !(*environ).is_null() {
                        let record = CStr::from_ptr(*environ).to_bytes_with_nul();
                        {
                            result.extend(record);
                        }
                        environ = environ.add(1);
                    }
                }

                Ok(result)
            }
        }
    }

    impl Drop for Kvm {
        fn drop(&mut self) {
            let ret = unsafe { kvm_close(self.kd) };
            assert!(ret == 0, "{}", std::io::Error::last_os_error());
        }
    }
}

pub fn get_environment_string(pid: u32) -> AppResult<Vec<u8>> {
    let kvm = oxidation::Kvm::try_new()?;
    let proc = kvm.get_proc(pid)?;
    kvm.get_env(proc)
}
