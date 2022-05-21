#![allow(non_snake_case)]

// Ideas taken from https://stackoverflow.com/questions/1202653/check-for-environment-variable-in-another-process#answer-63222041

use super::definition::AppResult;
use std::ffi::c_void;
use windows::Win32::Foundation::UNICODE_STRING;
use windows::Win32::Security::{
    LUID_AND_ATTRIBUTES, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Kernel::STRING;
use windows::Win32::System::SystemServices::SE_DEBUG_NAME;
use windows::Win32::System::Threading::{
    ProcessBasicInformation, PEB, PROCESS_BASIC_INFORMATION, PROCESS_INFORMATION,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};

#[repr(C)]
#[derive(Debug)]
pub struct RTL_DRIVE_LETTER_CURDIR {
    Flags: u16,
    Length: u16,
    TimeStamp: u32,
    DosPath: STRING,
}

#[repr(C)]
#[derive(Debug)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    MaximumLength: u32,
    Length: u32,
    Flags: u32,
    DebugFlags: u32,
    ConsoleHandle: *const c_void,
    ConsoleFlags: u32,
    StdInputHandle: *const c_void,
    StdOutputHandle: *const c_void,
    StdErrorHandle: *const c_void,
    CurrentDirectoryPath: UNICODE_STRING,
    CurrentDirectoryHandle: *const c_void,
    DllPath: UNICODE_STRING,
    ImagePathName: UNICODE_STRING,
    CommandLine: UNICODE_STRING,
    Environment: *const c_void,
    StartingPositionLeft: u32,
    StartingPositionTop: u32,
    Width: u32,
    Height: u32,
    CharWidth: u32,
    CharHeight: u32,
    ConsoleTextAttributes: u32,
    WindowFlags: u32,
    ShowWindowFlags: u32,
    WindowTitle: UNICODE_STRING,
    DesktopName: UNICODE_STRING,
    ShellInfo: UNICODE_STRING,
    RuntimeData: UNICODE_STRING,
    DLCurrentDirectory: [RTL_DRIVE_LETTER_CURDIR; 32],
    EnvironmentSize: u32,
}

impl Default for RTL_USER_PROCESS_PARAMETERS {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

mod oxidation {
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::rc::Rc;
    use windows::Win32::Foundation::{CloseHandle, FARPROC, HANDLE, LUID, NTSTATUS};
    use windows::Win32::Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueA, TOKEN_ACCESS_MASK, TOKEN_PRIVILEGES,
    };
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESSINFOCLASS, PROCESS_ACCESS_RIGHTS, PROCESS_INFORMATION,
    };

    #[derive(Debug)]
    pub struct MyOwnedHandle {
        inner: HANDLE,
    }

    impl From<HANDLE> for MyOwnedHandle {
        fn from(h: HANDLE) -> Self {
            Self { inner: h }
        }
    }

    impl Drop for MyOwnedHandle {
        fn drop(&mut self) {
            let ok = unsafe { CloseHandle(self.inner) };
            assert!(ok.as_bool(), "Failed to close handle");
        }
    }

    #[derive(Debug, Clone)]
    pub struct SharedHandle {
        inner: Rc<MyOwnedHandle>,
    }

    impl From<HANDLE> for SharedHandle {
        fn from(h: HANDLE) -> Self {
            Self {
                inner: Rc::new(MyOwnedHandle { inner: h }),
            }
        }
    }

    impl From<SharedHandle> for HANDLE {
        fn from(shared_handle: SharedHandle) -> Self {
            shared_handle.inner.inner
        }
    }

    pub fn get_current_process() -> HANDLE {
        unsafe { windows::Win32::System::Threading::GetCurrentProcess() }
    }

    pub fn open_process_token(
        process_handle: HANDLE,
        desired_access: TOKEN_ACCESS_MASK,
    ) -> ::windows::core::Result<SharedHandle> {
        let mut out_handle: HANDLE = HANDLE::default();
        let result = unsafe {
            windows::Win32::System::Threading::OpenProcessToken(
                process_handle,
                desired_access,
                &mut out_handle,
            )
        };
        if result.as_bool() {
            Ok(out_handle.into())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }

    pub fn open_process(
        desired_access: PROCESS_ACCESS_RIGHTS,
        inherit_handle: bool,
        process_id: u32,
    ) -> ::windows::core::Result<SharedHandle> {
        let result = unsafe { OpenProcess(desired_access, inherit_handle, process_id) };
        result.map(std::convert::Into::into)
    }

    pub fn lookup_privilege_value(name: &str) -> ::windows::core::Result<LUID> {
        let system_name = None;

        let mut luid: LUID = LUID::default();
        let result = unsafe { LookupPrivilegeValueA(system_name, name, &mut luid) };
        if result.as_bool() {
            Ok(luid)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }

    pub fn adjust_token_privileges<T: Into<HANDLE>>(
        token_handle: T,
        disable_all_privileges: bool,
        new_state: TOKEN_PRIVILEGES,
    ) -> ::windows::core::Result<()> {
        // We don't need returning the old value
        let buffer_length: u32 = 0;
        let previous_state: *mut TOKEN_PRIVILEGES = null_mut();
        let mut return_length: u32 = 0;
        let result = unsafe {
            AdjustTokenPrivileges(
                token_handle.into(),
                disable_all_privileges,
                &new_state,
                buffer_length,
                previous_state,
                &mut return_length,
            )
        };
        if result.as_bool() {
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }

    #[inline]
    unsafe fn NtQueryInformationProcess_dyn(
        processhandle: HANDLE,
        processinformationclass: PROCESSINFOCLASS,
        processinformation: *mut c_void,
        processinformationlength: u32,
        returnlength: *mut u32,
    ) -> ::windows::core::Result<()> {
        type NtQueryInformationProcess = extern "system" fn(
            processhandle: HANDLE,
            processinformationclass: PROCESSINFOCLASS,
            processinformation: *mut c_void,
            processinformationlength: u32,
            returnlength: *mut u32,
        ) -> NTSTATUS;

        static mut FARPROC_STATIC: FARPROC = None;
        if FARPROC_STATIC.is_none() {
            let ntdll_handle = GetModuleHandleA("ntdll.dll")?;
            FARPROC_STATIC.replace(
                GetProcAddress(ntdll_handle, "NtQueryInformationProcess")
                    .expect("NtQueryInformationProcess unavailable"),
            );
        }

        let proc: NtQueryInformationProcess = ::core::mem::transmute(FARPROC_STATIC.unwrap());
        proc(
            processhandle,
            processinformationclass,
            processinformation,
            processinformationlength,
            returnlength,
        )
        .ok()
    }

    pub fn nt_query_information_process<T: Into<HANDLE>>(
        process_handle: T,
        process_information_class: PROCESSINFOCLASS,
        process_information: *mut PROCESS_INFORMATION,
        size: usize,
    ) -> ::windows::core::Result<()> {
        let mut return_length = 0;
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            NtQueryInformationProcess_dyn(
                process_handle.into(),
                process_information_class,
                process_information.cast::<c_void>(),
                size as u32,
                &mut return_length,
            )
        }
    }

    pub fn read_process_memory<T: Into<HANDLE>>(
        process_handle: T,
        base_addr: *const c_void,
        buffer: *mut c_void,
        nsize: usize,
    ) -> ::windows::core::Result<()> {
        let mut number_of_bytes_read = 0;
        let result = unsafe {
            ReadProcessMemory(
                process_handle.into(),
                base_addr,
                buffer,
                nsize,
                &mut number_of_bytes_read,
            )
        };

        if result.as_bool() {
            assert_eq!(number_of_bytes_read, nsize, "Not read");
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

pub fn get_environment_string(pid: u32) -> AppResult<String> {
    let current_process = oxidation::get_current_process();

    let token_handle =
        oxidation::open_process_token(current_process, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY)?;
    let luid = oxidation::lookup_privilege_value(SE_DEBUG_NAME)?;

    let token_privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            ..LUID_AND_ATTRIBUTES::default()
        }; 1],
    };
    oxidation::adjust_token_privileges(token_handle.clone(), false, token_privileges)?;

    let process_handle =
        oxidation::open_process(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;

    let basic_info = {
        let mut out = PROCESS_BASIC_INFORMATION::default();
        oxidation::nt_query_information_process(
            process_handle.clone(),
            ProcessBasicInformation,
            std::ptr::addr_of_mut!(out).cast::<PROCESS_INFORMATION>(),
            core::mem::size_of::<PROCESS_BASIC_INFORMATION>(),
        )?;
        out
    };

    let peb = {
        let mut out = PEB::default();
        oxidation::read_process_memory(
            process_handle.clone(),
            basic_info.PebBaseAddress.cast::<c_void>(),
            std::ptr::addr_of_mut!(out).cast::<c_void>(),
            core::mem::size_of::<PROCESS_BASIC_INFORMATION>(),
        )?;
        out
    };

    let user_process_parameters = {
        let mut out = RTL_USER_PROCESS_PARAMETERS::default();
        oxidation::read_process_memory(
            process_handle.clone(),
            peb.ProcessParameters.cast::<c_void>(),
            std::ptr::addr_of_mut!(out).cast::<c_void>(),
            core::mem::size_of::<RTL_USER_PROCESS_PARAMETERS>(),
        )?;
        out
    };

    let cap = (user_process_parameters.EnvironmentSize / 2) as usize;

    let environment = {
        let mut out = vec![0u16; cap];

        oxidation::read_process_memory(
            process_handle.clone(),
            user_process_parameters.Environment,
            out.as_mut_ptr().cast::<c_void>(),
            user_process_parameters.EnvironmentSize as usize,
        )?;
        out
    };

    // Prevent pre-maturely dropping these handles by passing non-clones to functions.
    drop(token_handle);
    drop(process_handle);

    Ok(String::from_utf16(&environment)?)
}

#[cfg(test)]
mod tests {
    use super::get_environment_string;
    use crate::env::{get_record_pairs_for_current_process, parse_env_var_string};

    #[test]
    fn test_get_environment_string() {
        let env_string = get_environment_string(std::process::id())?;
        let actual = parse_env_var_string(env_string.as_bytes());
        let expected = get_record_pairs_for_current_process();
        assert_eq!(actual, expected);
    }
}
