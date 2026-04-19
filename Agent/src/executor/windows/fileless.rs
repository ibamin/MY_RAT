use std::{
    ffi::c_void,
    ptr,
    time::Instant,
};

use async_trait::async_trait;
use windows::{
    Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Win32::System::Memory::{
        VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_EXECUTE_READ, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
    },
    Win32::System::Threading::{
        CreateThread, TerminateThread, WaitForSingleObject, LPTHREAD_START_ROUTINE, THREAD_CREATION_FLAGS,
    },
};

use crate::executor::{ExecutionResult, Executor};

pub struct FilelessExecutor;

#[async_trait]
impl Executor for FilelessExecutor {
    fn kind(&self) -> &'static str {
        "fileless"
    }

    fn is_available(&self) -> bool {
        cfg!(windows)
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let command = command.to_string();
        let args = args.iter().map(|value| value.to_string()).collect::<Vec<_>>();

        let result = tokio::task::spawn_blocking(move || unsafe { execute_shellcode(&command, &args) }).await;
        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((success, exit_code, stdout))) => ExecutionResult {
                success,
                stdout,
                stderr: Vec::new(),
                exit_code,
                evidence: None,
                duration_ms,
            },
            Ok(Err(err_text)) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: err_text.into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms,
            },
            Err(join_err) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: join_err.to_string().into_bytes(),
                exit_code: None,
                evidence: None,
                duration_ms,
            },
        }
    }
}

struct AllocationGuard(*mut c_void);

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = VirtualFree(self.0, 0, MEM_RELEASE);
            }
        }
    }
}

struct ThreadGuard(HANDLE);

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

unsafe fn execute_shellcode(command: &str, args: &[String]) -> Result<(bool, Option<i32>, Vec<u8>), String> {
    let shellcode = if args
        .first()
        .map(|value| value.eq_ignore_ascii_case("base64"))
        .unwrap_or(false)
    {
        decode_base64(command)?
    } else {
        decode_hex(command)?
    };

    if shellcode.is_empty() {
        return Err("Shellcode payload is empty".to_string());
    }

    let alloc = VirtualAlloc(
        None,
        shellcode.len(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_READWRITE,
    );

    if alloc.is_null() {
        return Err("VirtualAlloc failed".to_string());
    }

    let alloc_guard = AllocationGuard(alloc);

    ptr::copy_nonoverlapping(shellcode.as_ptr(), alloc as *mut u8, shellcode.len());

    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(
        alloc,
        shellcode.len(),
        PAGE_EXECUTE_READ,
        &mut old_protect,
    )
    .map_err(|err| format!("VirtualProtect failed: {}", err.message()))?;

    let start_routine: LPTHREAD_START_ROUTINE = Some(std::mem::transmute::<*mut c_void, unsafe extern "system" fn(*mut c_void) -> u32>(alloc));
    let thread_handle = CreateThread(
        None,
        0,
        start_routine,
        Some(ptr::null()),
        THREAD_CREATION_FLAGS(0),
        None,
    )
    .map_err(|err| format!("CreateThread failed: {}", err.message()))?;

    let _alloc_guard = alloc_guard;
    let _thread_guard = ThreadGuard(thread_handle);

    let wait_status = WaitForSingleObject(thread_handle, 30_000);
    if wait_status == WAIT_OBJECT_0 {
        return Ok((true, Some(0), b"Shellcode thread completed".to_vec()));
    }

    if wait_status == WAIT_TIMEOUT {
        let _ = TerminateThread(thread_handle, 1);
        return Ok((false, Some(1), b"Shellcode thread timed out after 30 seconds".to_vec()));
    }

    Err(format!(
        "WaitForSingleObject returned unexpected status: {:?}",
        wait_status
    ))
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.len() % 2 != 0 {
        return Err("Hex payload length must be even".to_string());
    }

    let bytes = trimmed.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);

    for i in (0..bytes.len()).step_by(2) {
        let high = hex_value(bytes[i]).ok_or_else(|| {
            format!(
                "Invalid hex character '{}' at position {}",
                bytes[i] as char, i
            )
        })?;
        let low = hex_value(bytes[i + 1]).ok_or_else(|| {
            format!(
                "Invalid hex character '{}' at position {}",
                bytes[i + 1] as char,
                i + 1
            )
        })?;
        output.push((high << 4) | low);
    }

    Ok(output)
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let cleaned = input
        .bytes()
        .filter(|value| !value.is_ascii_whitespace())
        .collect::<Vec<_>>();

    if cleaned.is_empty() {
        return Ok(Vec::new());
    }

    if cleaned.len() % 4 != 0 {
        return Err("Base64 payload length must be divisible by 4".to_string());
    }

    let mut output = Vec::with_capacity((cleaned.len() / 4) * 3);
    for chunk in cleaned.chunks(4) {
        let a = decode_base64_char(chunk[0]).ok_or_else(|| {
            format!(
                "Invalid base64 character '{}' at position {}",
                chunk[0] as char,
                output.len()
            )
        })?;
        let b = decode_base64_char(chunk[1]).ok_or_else(|| {
            format!(
                "Invalid base64 character '{}' at position {}",
                chunk[1] as char,
                output.len() + 1
            )
        })?;

        let c_padding = chunk[2] == b'=';
        let d_padding = chunk[3] == b'=';

        if c_padding && !d_padding {
            return Err("Invalid base64 padding".to_string());
        }

        let c = if c_padding {
            0
        } else {
            decode_base64_char(chunk[2]).ok_or_else(|| {
                format!(
                    "Invalid base64 character '{}' at position {}",
                    chunk[2] as char,
                    output.len() + 2
                )
            })?
        };

        let d = if d_padding {
            0
        } else {
            decode_base64_char(chunk[3]).ok_or_else(|| {
                format!(
                    "Invalid base64 character '{}' at position {}",
                    chunk[3] as char,
                    output.len() + 3
                )
            })?
        };

        output.push((a << 2) | (b >> 4));
        if !c_padding {
            output.push((b << 4) | (c >> 2));
        }
        if !d_padding {
            output.push((c << 6) | d);
        }
    }

    Ok(output)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn decode_base64_char(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
