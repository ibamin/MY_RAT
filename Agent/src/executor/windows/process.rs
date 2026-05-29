use std::time::Instant;

use async_trait::async_trait;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Win32::System::Threading::{
        CreateProcessW, GetExitCodeProcess, WaitForSingleObject, CREATE_NEW_CONSOLE,
        CREATE_NO_WINDOW, PROCESS_INFORMATION, STARTUPINFOW, STARTF_USESHOWWINDOW,
    },
    Win32::UI::WindowsAndMessaging::SW_HIDE,
};

use crate::executor::{ExecutionResult, Executor};

pub struct ProcessExecutor;

#[async_trait]
impl Executor for ProcessExecutor {
    fn kind(&self) -> &'static str {
        "process"
    }

    fn is_available(&self) -> bool {
        cfg!(windows)
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let executable = command.to_string();
        let command_line = args.first().map(|value| value.to_string());
        let hidden = args
            .get(1)
            .map(|value| value.eq_ignore_ascii_case("hidden") || value == &"true" || value == &"1")
            .unwrap_or(false);

        let result = tokio::task::spawn_blocking(move || unsafe {
            create_process_and_wait(&executable, command_line.as_deref(), hidden)
        })
        .await;

        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(exit_code)) => ExecutionResult {
                success: exit_code == Some(0),
                stdout: Vec::new(),
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

unsafe fn create_process_and_wait(
    executable: &str,
    command_line: Option<&str>,
    hidden: bool,
) -> Result<Option<i32>, String> {
    let app_wide: Vec<u16> = executable.encode_utf16().chain(std::iter::once(0)).collect();
    let app_name = PCWSTR::from_raw(app_wide.as_ptr());

    let mut cmd_wide = command_line
        .map(|value| value.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>());
    let cmd_ptr = cmd_wide
        .as_mut()
        .map(|value| PWSTR::from_raw(value.as_mut_ptr()));

    let startup_info = if hidden {
        STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE.0 as u16,
            ..Default::default()
        }
    } else {
        STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        }
    };

    let mut process_info = PROCESS_INFORMATION::default();

    CreateProcessW(
        app_name,
        cmd_ptr,
        None,
        None,
        false,
        if hidden {
            CREATE_NO_WINDOW
        } else {
            CREATE_NEW_CONSOLE
        },
        None,
        None,
        &startup_info,
        &mut process_info,
    )
    .map_err(|err| err.message().to_string())?;

    let wait_result = WaitForSingleObject(process_info.hProcess, 30_000);
    if wait_result == WAIT_TIMEOUT {
        close_if_valid(process_info.hThread);
        close_if_valid(process_info.hProcess);
        return Err("Process timed out after 30 seconds".to_string());
    }

    if wait_result != WAIT_OBJECT_0 {
        close_if_valid(process_info.hThread);
        close_if_valid(process_info.hProcess);
        return Err(format!("WaitForSingleObject failed: {:?}", wait_result));
    }

    let mut exit_code = 0u32;
    let status = GetExitCodeProcess(process_info.hProcess, &mut exit_code);

    close_if_valid(process_info.hThread);
    close_if_valid(process_info.hProcess);

    status
        .map_err(|err| err.message().to_string())
        .map(|_| Some(exit_code as i32))
}

unsafe fn close_if_valid(handle: HANDLE) {
    if !handle.is_invalid() {
        let _ = CloseHandle(handle);
    }
}
