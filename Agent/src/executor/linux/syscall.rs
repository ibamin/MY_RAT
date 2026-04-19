use std::{
    ffi::CString,
    fs,
    ptr,
    time::Instant,
};

use async_trait::async_trait;

use crate::executor::{ExecutionResult, Executor};

pub struct RawSyscallExecutor;

#[async_trait]
impl Executor for RawSyscallExecutor {
    fn kind(&self) -> &'static str {
        "raw_syscall"
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, command: &str, args: &[&str]) -> ExecutionResult {
        let started = Instant::now();
        let command_owned = command.to_string();
        let args_owned: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();

        let result =
            tokio::task::spawn_blocking(move || run_raw_syscall(&command_owned, &args_owned)).await;
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
            Ok(Err(err)) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: err.into_bytes(),
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

fn run_raw_syscall(command: &str, args: &[String]) -> Result<Option<i32>, String> {
    let elf_bytes = fs::read(command).map_err(|err| format!("failed to read ELF '{}': {}", command, err))?;

    let fd_name = CString::new("syscall_payload").map_err(|err| format!("invalid memfd name: {}", err))?;

    let mut argv_data: Vec<CString> = Vec::with_capacity(args.len() + 1);
    argv_data.push(CString::new("syscall_payload").map_err(|err| format!("invalid argv[0]: {}", err))?);
    for arg in args {
        argv_data.push(CString::new(arg.as_str()).map_err(|err| format!("invalid arg: {}", err))?);
    }
    let mut argv_ptrs: Vec<*const libc::c_char> = argv_data.iter().map(|s| s.as_ptr()).collect();
    argv_ptrs.push(ptr::null());

    let mut env_data: Vec<CString> = Vec::new();
    for (key, value) in std::env::vars() {
        let pair = format!("{}={}", key, value);
        env_data.push(CString::new(pair).map_err(|err| format!("invalid env var for exec: {}", err))?);
    }
    let mut env_ptrs: Vec<*const libc::c_char> = env_data.iter().map(|s| s.as_ptr()).collect();
    env_ptrs.push(ptr::null());

    let fd = unsafe {
        libc::syscall(
            libc::SYS_memfd_create,
            fd_name.as_ptr(),
            libc::MFD_CLOEXEC,
        ) as libc::c_int
    };
    if fd < 0 {
        return Err(format!("SYS_memfd_create failed (errno={})", errno_code()));
    }

    let write_result = write_all_fd_syscall(fd, &elf_bytes);
    if let Err(err) = write_result {
        unsafe {
            libc::close(fd);
        }
        return Err(err);
    }

    let empty_path = CString::new("").map_err(|err| format!("invalid execveat path: {}", err))?;

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        unsafe {
            libc::close(fd);
        }
        return Err(format!("fork failed (errno={})", errno_code()));
    }

    if pid == 0 {
        unsafe {
            let exec_rc = libc::syscall(
                libc::SYS_execveat,
                fd,
                empty_path.as_ptr(),
                argv_ptrs.as_ptr(),
                env_ptrs.as_ptr(),
                libc::AT_EMPTY_PATH,
            );
            if exec_rc == -1 {
                let msg = b"SYS_execveat failed\n";
                let _ = libc::syscall(
                    libc::SYS_write,
                    libc::STDERR_FILENO,
                    msg.as_ptr() as *const libc::c_void,
                    msg.len(),
                );
            }
            libc::_exit(127);
        }
    }

    let exit_code = wait_for_pid(pid)?;

    unsafe {
        libc::close(fd);
    }

    Ok(Some(exit_code))
}

fn write_all_fd_syscall(fd: libc::c_int, bytes: &[u8]) -> Result<(), String> {
    let mut written: usize = 0;
    while written < bytes.len() {
        let slice = &bytes[written..];
        let rc = unsafe {
            libc::syscall(
                libc::SYS_write,
                fd,
                slice.as_ptr() as *const libc::c_void,
                slice.len(),
            ) as libc::ssize_t
        };

        if rc < 0 {
            let code = errno_code();
            if code == libc::EINTR {
                continue;
            }
            return Err(format!("SYS_write to memfd failed (errno={})", code));
        }

        if rc == 0 {
            return Err("SYS_write to memfd returned 0 bytes".to_string());
        }

        written += rc as usize;
    }

    Ok(())
}

fn wait_for_pid(pid: libc::pid_t) -> Result<i32, String> {
    let mut status: libc::c_int = 0;
    loop {
        let rc = unsafe { libc::waitpid(pid, &mut status, 0) };
        if rc < 0 {
            let code = errno_code();
            if code == libc::EINTR {
                continue;
            }
            return Err(format!("waitpid failed (errno={})", code));
        }
        break;
    }

    if libc::WIFEXITED(status) {
        return Ok(libc::WEXITSTATUS(status));
    }

    if libc::WIFSIGNALED(status) {
        return Ok(128 + libc::WTERMSIG(status));
    }

    Ok(1)
}

fn errno_code() -> i32 {
    unsafe { *libc::__errno_location() }
}
