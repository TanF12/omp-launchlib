#[cfg(windows)]
use std::ffi::{OsStr, c_void};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessW, CreateRemoteThread, INFINITE, PROCESS_INFORMATION,
    ResumeThread, STARTUPINFOW, TerminateProcess, WaitForSingleObject,
};

#[cfg(windows)]
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn inject_dll_crt(h_process: HANDLE, dll_path: &str) -> Result<(), String> {
    let dll_wide = to_wide_string(dll_path);
    let size = dll_wide.len() * std::mem::size_of::<u16>();

    unsafe {
        let p_remote_mem = VirtualAllocEx(
            h_process,
            std::ptr::null(),
            size,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if p_remote_mem.is_null() {
            return Err(format!("VirtualAllocEx failed: {}", GetLastError()));
        }

        let mut bytes_written = 0;
        let write_res = windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory(
            h_process,
            p_remote_mem,
            dll_wide.as_ptr() as *const c_void,
            size,
            &mut bytes_written,
        );

        if write_res == 0 {
            VirtualFreeEx(h_process, p_remote_mem, 0, MEM_RELEASE);
            return Err(format!("WriteProcessMemory failed: {}", GetLastError()));
        }

        let kernel32_name = to_wide_string("kernel32.dll");
        let h_kernel32 = GetModuleHandleW(kernel32_name.as_ptr());
        let p_load_library = GetProcAddress(h_kernel32, b"LoadLibraryW\0".as_ptr());

        if p_load_library.is_none() {
            VirtualFreeEx(h_process, p_remote_mem, 0, MEM_RELEASE);
            return Err("GetProcAddress failed for LoadLibraryW".into());
        }

        let h_thread = CreateRemoteThread(
            h_process,
            std::ptr::null(),
            0,
            std::mem::transmute(p_load_library.unwrap()),
            p_remote_mem as *const c_void,
            0,
            std::ptr::null_mut(),
        );

        if h_thread.is_null() {
            VirtualFreeEx(h_process, p_remote_mem, 0, MEM_RELEASE);
            return Err(format!("CreateRemoteThread failed: {}", GetLastError()));
        }

        WaitForSingleObject(h_thread, INFINITE);
        CloseHandle(h_thread);
        VirtualFreeEx(h_process, p_remote_mem, 0, MEM_RELEASE);
    }
    Ok(())
}

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("Injector must be compiled for Windows.");
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 4 {
            eprintln!("Usage: omp-injector <gta_path> <dll_count> <dll1> [dll2] <game_args...>");
            std::process::exit(1);
        }

        let gta_path = &args[1];
        let dll_count: usize = args[2].parse().expect("Invalid number of DLLs");
        let dlls = &args[3..3 + dll_count];
        let game_args = &args[3 + dll_count..];

        let mut cmd_line = format!("\"{}\"", gta_path);
        for arg in game_args {
            let escaped_arg = arg.replace("\"", "\\\"");
            if escaped_arg.contains(' ') || escaped_arg.contains('\t') {
                cmd_line.push_str(&format!(" \"{}\"", escaped_arg));
            } else {
                cmd_line.push_str(&format!(" {}", escaped_arg));
            }
        }

        let mut cmd_wide = to_wide_string(&cmd_line);
        let gta_dir_wide = to_wide_string(
            std::path::Path::new(gta_path)
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
        );

        let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

        unsafe {
            let success = CreateProcessW(
                std::ptr::null(),
                cmd_wide.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                CREATE_SUSPENDED,
                std::ptr::null(),
                gta_dir_wide.as_ptr(),
                &si,
                &mut pi,
            );

            if success == 0 {
                eprintln!("CreateProcessW failed: {}", GetLastError());
                std::process::exit(1);
            }

            let mut all_injected = true;
            for dll in dlls {
                if let Err(e) = inject_dll_crt(pi.hProcess, dll) {
                    eprintln!("Injection failed: {}", e);
                    all_injected = false;
                    break;
                }
            }

            if !all_injected {
                TerminateProcess(pi.hProcess, 1);
            } else {
                ResumeThread(pi.hThread);
            }

            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
        }
    }
}
