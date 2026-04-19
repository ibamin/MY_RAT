use std::time::Instant;

use async_trait::async_trait;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::Foundation::{CloseHandle, SetHandleInformation, HANDLE, HANDLE_FLAGS, HANDLE_FLAG_INHERIT, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Win32::Security::SECURITY_ATTRIBUTES,
    Win32::Storage::FileSystem::ReadFile,
    Win32::System::Pipes::CreatePipe,
    Win32::System::Threading::{
        CreateProcessW, GetExitCodeProcess, TerminateProcess, WaitForSingleObject, CREATE_NO_WINDOW,
        PROCESS_INFORMATION, STARTUPINFOW, STARTF_USESHOWWINDOW, STARTF_USESTDHANDLES,
    },
    Win32::UI::WindowsAndMessaging::SW_HIDE,
};

use crate::executor::{ExecutionResult, Executor};

pub struct PowerShellExecutor;

#[async_trait]
impl Executor for PowerShellExecutor {
    fn kind(&self) -> &'static str {
        "powershell"
    }

    fn is_available(&self) -> bool {
        cfg!(windows)
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let script = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        let result = tokio::task::spawn_blocking(move || unsafe { execute_powershell_with_pipes(&script) }).await;
        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((stdout, stderr, exit_code))) => ExecutionResult {
                success: exit_code == Some(0),
                stdout,
                stderr,
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

type PipeResult = Result<(Vec<u8>, Vec<u8>, Option<i32>), String>;

unsafe fn execute_powershell_with_pipes(script: &str) -> PipeResult {
    let mut stdout_read = HANDLE::default();
    let mut stdout_write = HANDLE::default();
    let mut stderr_read = HANDLE::default();
    let mut stderr_write = HANDLE::default();

    let security_attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: true.into(),
    };

    CreatePipe(&mut stdout_read, &mut stdout_write, Some(&security_attributes), 0)
        .map_err(|err| err.message().to_string())?;

    if let Err(err) = CreatePipe(&mut stderr_read, &mut stderr_write, Some(&security_attributes), 0) {
        close_if_valid(stdout_read);
        close_if_valid(stdout_write);
        return Err(err.message().to_string());
    }

    if let Err(err) = SetHandleInformation(stdout_read, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) {
        close_if_valid(stdout_read);
        close_if_valid(stdout_write);
        close_if_valid(stderr_read);
        close_if_valid(stderr_write);
        return Err(err.message().to_string());
    }

    if let Err(err) = SetHandleInformation(stderr_read, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) {
        close_if_valid(stdout_read);
        close_if_valid(stdout_write);
        close_if_valid(stderr_read);
        close_if_valid(stderr_write);
        return Err(err.message().to_string());
    }

    let startup_info = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW,
        wShowWindow: SW_HIDE.0 as u16,
        hStdOutput: stdout_write,
        hStdError: stderr_write,
        ..Default::default()
    };

    let mut process_info = PROCESS_INFORMATION::default();
    let command = format!(
        "powershell.exe -NoProfile -NonInteractive -Command \"{}\"",
        script.replace('"', "\\\"")
    );
    let mut command_wide: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();

    if let Err(err) = CreateProcessW(
        PCWSTR::null(),
        Some(PWSTR::from_raw(command_wide.as_mut_ptr())),
        None,
        None,
        true,
        CREATE_NO_WINDOW,
        None,
        None,
        &startup_info,
        &mut process_info,
    ) {
        close_if_valid(stdout_read);
        close_if_valid(stdout_write);
        close_if_valid(stderr_read);
        close_if_valid(stderr_write);
        return Err(err.message().to_string());
    }

    close_if_valid(stdout_write);
    close_if_valid(stderr_write);

    // SAFETY: HANDLE is a raw pointer (*mut c_void) which is !Send, but we are
    // transferring sole ownership of each pipe handle to exactly one reader thread.
    // No other thread accesses these handles after this point.
    let stdout_raw = stdout_read.0 as usize;
    let stderr_raw = stderr_read.0 as usize;

    let stdout_reader = std::thread::spawn(move || {
        read_pipe_to_vec(HANDLE(stdout_raw as *mut _))
    });
    let stderr_reader = std::thread::spawn(move || {
        read_pipe_to_vec(HANDLE(stderr_raw as *mut _))
    });

    let wait_result = WaitForSingleObject(process_info.hProcess, 30_000);
    let mut exit_code = 0u32;
    if wait_result == WAIT_TIMEOUT {
        let _ = TerminateProcess(process_info.hProcess, 1);
        exit_code = 1;
    } else if wait_result == WAIT_OBJECT_0 {
        GetExitCodeProcess(process_info.hProcess, &mut exit_code)
            .map_err(|err| err.message().to_string())?;
    } else {
        close_if_valid(process_info.hThread);
        close_if_valid(process_info.hProcess);
        let stdout = stdout_reader.join().unwrap_or_default();
        let stderr = stderr_reader.join().unwrap_or_default();
        return Err(format!(
            "WaitForSingleObject returned unexpected status: {:?}; stdout_len={}, stderr_len={}",
            wait_result,
            stdout.len(),
            stderr.len()
        ));
    }

    close_if_valid(process_info.hThread);
    close_if_valid(process_info.hProcess);

    let stdout = stdout_reader.join().unwrap_or_default();
    let mut stderr = stderr_reader.join().unwrap_or_default();

    if wait_result == WAIT_TIMEOUT {
        stderr.extend_from_slice(b"Process timed out after 30 seconds");
    }

    Ok((stdout, stderr, Some(exit_code as i32)))
}

fn read_pipe_to_vec(handle: HANDLE) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_read = 0u32;
        let read_result = unsafe {
            ReadFile(
                handle,
                Some(&mut buffer),
                Some(&mut bytes_read as *mut u32),
                None,
            )
        };

        if read_result.is_err() || bytes_read == 0 {
            break;
        }

        output.extend_from_slice(&buffer[..bytes_read as usize]);
    }

    unsafe {
        close_if_valid(handle);
    }

    output
}

unsafe fn close_if_valid(handle: HANDLE) {
    if !handle.is_invalid() {
        let _ = CloseHandle(handle);
    }
}
