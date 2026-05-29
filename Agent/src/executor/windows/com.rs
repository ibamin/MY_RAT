use std::time::{Duration, Instant};

use async_trait::async_trait;
use windows::{
    core::{BSTR, GUID, PCWSTR},
    Win32::Foundation::E_FAIL,
    Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, DISPATCH_METHOD, DISPATCH_PROPERTYGET,
        DISPPARAMS, EXCEPINFO, IDispatch, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
    },
    Win32::System::Variant::{VARIANT, VT_BSTR, VT_DISPATCH, VT_I4},
};

use crate::executor::{ExecutionResult, Executor};

pub struct ComExecutor;

#[async_trait]
impl Executor for ComExecutor {
    fn kind(&self) -> &'static str {
        "com"
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

        let command_line = format!(
            "powershell.exe -NoProfile -NonInteractive -Command \"{}\"",
            script.replace('"', "\\\"")
        );

        let result = tokio::task::spawn_blocking(move || unsafe { execute_command(&command_line) }).await;
        let duration_ms = started.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                let success = exit_code.unwrap_or(0) == 0;
                ExecutionResult {
                    success,
                    stdout,
                    stderr,
                    exit_code,
                    evidence: None,
                    duration_ms,
                }
            }
            Ok(Err(err)) => ExecutionResult {
                success: false,
                stdout: Vec::new(),
                stderr: err.message().to_string().into_bytes(),
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

struct CoInitGuard;

impl Drop for CoInitGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize(); }
    }
}

unsafe fn execute_command(command: &str) -> windows::core::Result<(Vec<u8>, Vec<u8>, Option<i32>)> {
    CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    let _com_guard = CoInitGuard;

    let clsid = GUID::from_u128(0x72C24DD5_D70A_438B_8A42_98424B88AFB8);
    let shell: IDispatch = CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER)?;
    let exec_object = call_method_simple(&shell, "Exec", &[VARIANT::from(command)])?;
    let exec_dispatch = dispatch_from_variant(&exec_object)?;

    let wait_start = Instant::now();
    while wait_start.elapsed() < Duration::from_secs(30) {
        let status = get_property_simple(&exec_dispatch, "Status")?;
        if status.Anonymous.Anonymous.vt == VT_I4
            && status.Anonymous.Anonymous.Anonymous.lVal == 1
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if wait_start.elapsed() >= Duration::from_secs(30) {
        return Err(windows::core::Error::new(E_FAIL, "COM command timed out"));
    }

    let stdout = read_stream_text(&exec_dispatch, "StdOut")?.into_bytes();
    let stderr = read_stream_text(&exec_dispatch, "StdErr")?.into_bytes();
    let exit_code = read_exit_code(&exec_dispatch)?;

    Ok((stdout, stderr, exit_code))
}

unsafe fn get_property_simple(obj: &IDispatch, property_name: &str) -> windows::core::Result<VARIANT> {
    let property_bstr = BSTR::from(property_name);
    let property_wide: Vec<u16> = property_bstr
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let property_pcwstr = PCWSTR::from_raw(property_wide.as_ptr());

    let mut dispid = i32::default();
    obj.GetIDsOfNames(&GUID::zeroed(), &property_pcwstr, 1, 0, &mut dispid)?;

    let disp_params = DISPPARAMS::default();
    let mut result = VARIANT::default();
    let mut excep_info = EXCEPINFO::default();
    let mut arg_err = 0u32;

    obj.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_PROPERTYGET,
        &disp_params,
        Some(&mut result),
        Some(&mut excep_info),
        Some(&mut arg_err),
    )?;

    Ok(result)
}

unsafe fn call_method_simple(
    obj: &IDispatch,
    method_name: &str,
    args: &[VARIANT],
) -> windows::core::Result<VARIANT> {
    let method_bstr = BSTR::from(method_name);
    let method_wide: Vec<u16> = method_bstr
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let method_pcwstr = PCWSTR::from_raw(method_wide.as_ptr());

    let mut dispid = i32::default();
    obj.GetIDsOfNames(&GUID::zeroed(), &method_pcwstr, 1, 0, &mut dispid)?;

    let disp_params = DISPPARAMS {
        rgvarg: args.as_ptr() as *mut VARIANT,
        rgdispidNamedArgs: std::ptr::null_mut(),
        cArgs: args.len() as u32,
        cNamedArgs: 0,
    };

    let mut result = VARIANT::default();
    let mut excep_info = EXCEPINFO::default();
    let mut arg_err = 0u32;

    obj.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD,
        &disp_params,
        Some(&mut result),
        Some(&mut excep_info),
        Some(&mut arg_err),
    )?;

    Ok(result)
}

unsafe fn dispatch_from_variant(value: &VARIANT) -> windows::core::Result<IDispatch> {
    if value.Anonymous.Anonymous.vt != VT_DISPATCH {
        return Err(windows::core::Error::new(E_FAIL, "Expected VT_DISPATCH"));
    }

    value
        .Anonymous
        .Anonymous
        .Anonymous
        .pdispVal
        .as_ref()
        .cloned()
        .ok_or_else(|| windows::core::Error::new(E_FAIL, "Missing IDispatch value"))
}

unsafe fn read_stream_text(exec_dispatch: &IDispatch, stream_name: &str) -> windows::core::Result<String> {
    let stream_variant = get_property_simple(exec_dispatch, stream_name)?;
    let stream_dispatch = dispatch_from_variant(&stream_variant)?;
    let content_variant = call_method_simple(&stream_dispatch, "ReadAll", &[])?;

    if content_variant.Anonymous.Anonymous.vt == VT_BSTR {
        return Ok(content_variant.Anonymous.Anonymous.Anonymous.bstrVal.to_string());
    }

    Ok(String::new())
}

unsafe fn read_exit_code(exec_dispatch: &IDispatch) -> windows::core::Result<Option<i32>> {
    let exit_code_variant = get_property_simple(exec_dispatch, "ExitCode")?;
    if exit_code_variant.Anonymous.Anonymous.vt == VT_I4 {
        return Ok(Some(exit_code_variant.Anonymous.Anonymous.Anonymous.lVal));
    }
    Ok(None)
}
