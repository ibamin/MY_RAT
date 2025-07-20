use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::System::Variant::*,
};

fn execute_with_output() -> windows::core::Result<String> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();

        let clsid = GUID::from_u128(0x72C24DD5_D70A_438B_8A42_98424B88AFB8);
        let shell: IDispatch = CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER)?;

        // Exec 메서드 호출
        let command = "powershell.exe -Command \"whoami; Start-Sleep 2; ipconfig\"";
        let output = execute_command(&shell, command)?;
        
        CoUninitialize();
        Ok(output)
    }
}

unsafe fn execute_command(shell: &IDispatch, command: &str) -> windows::core::Result<String> {
    println!("Executing: {}", command);
    
    // Exec 메서드 호출
    let method_name = BSTR::from("Exec");
    let mut dispid = i32::default();
    
    // BSTR을 PCWSTR로 변환
    let method_name_wide: Vec<u16> = method_name.to_string().encode_utf16().chain(std::iter::once(0)).collect();
    let method_pcwstr = PCWSTR::from_raw(method_name_wide.as_ptr());
    
    shell.GetIDsOfNames(
        &GUID::zeroed(),
        &method_pcwstr,
        1,
        0,
        &mut dispid,
    )?;

    let args = [VARIANT::from(command)];
    
    let disp_params = DISPPARAMS {
        rgvarg: args.as_ptr() as *mut VARIANT,
        rgdispidNamedArgs: std::ptr::null_mut(),
        cArgs: args.len() as u32,
        cNamedArgs: 0,
    };

    let mut result = VARIANT::default();
    let mut excep_info = EXCEPINFO::default();
    let mut arg_err = 0u32;

    shell.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD,
        &disp_params as *const DISPPARAMS,
        Some(&mut result as *mut VARIANT),
        Some(&mut excep_info as *mut EXCEPINFO),
        Some(&mut arg_err as *mut u32),
    )?;

    // WshExec 객체에서 StdOut 읽기
    if result.Anonymous.Anonymous.vt == VT_DISPATCH {
        let exec_obj = &result.Anonymous.Anonymous.Anonymous.pdispVal;
        if let Some(exec_dispatch) = exec_obj.as_ref() {
            
            // 프로세스 완료 대기
            loop {
                let status = get_property_simple(exec_dispatch, "Status")?;
                if status.Anonymous.Anonymous.vt == VT_I4 {
                    if status.Anonymous.Anonymous.Anonymous.lVal == 1 {  // WshFinished
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                print!(".");
            }
            println!();
            
            // StdOut 읽기
            let stdout_result = get_property_simple(exec_dispatch, "StdOut")?;
            
            if stdout_result.Anonymous.Anonymous.vt == VT_DISPATCH {
                let stdout_obj = &stdout_result.Anonymous.Anonymous.Anonymous.pdispVal;
                if let Some(stdout_dispatch) = stdout_obj.as_ref() {
                    
                    let output = call_method_simple(stdout_dispatch, "ReadAll", &[])?;
                    if output.Anonymous.Anonymous.vt == VT_BSTR {
                        return Ok(output.Anonymous.Anonymous.Anonymous.bstrVal.to_string());
                    }
                }
            }
        }
    }
    
    Ok(String::new())
}

unsafe fn get_property_simple(obj: &IDispatch, property_name: &str) -> windows::core::Result<VARIANT> {
    let prop_bstr = BSTR::from(property_name);
    let mut dispid = i32::default();
    
    // String을 PCWSTR로 변환
    let prop_wide: Vec<u16> = prop_bstr.to_string().encode_utf16().chain(std::iter::once(0)).collect();
    let prop_pcwstr = PCWSTR::from_raw(prop_wide.as_ptr());
    
    obj.GetIDsOfNames(
        &GUID::zeroed(),
        &prop_pcwstr,
        1,
        0,
        &mut dispid,
    )?;
    
    let disp_params = DISPPARAMS::default();
    let mut result = VARIANT::default();
    let mut excep_info = EXCEPINFO::default();
    let mut arg_err = 0u32;
    
    obj.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_PROPERTYGET,
        &disp_params as *const DISPPARAMS,
        Some(&mut result as *mut VARIANT),
        Some(&mut excep_info as *mut EXCEPINFO),
        Some(&mut arg_err as *mut u32),
    )?;
    
    Ok(result)
}

unsafe fn call_method_simple(obj: &IDispatch, method_name: &str, args: &[VARIANT]) -> windows::core::Result<VARIANT> {
    let method_bstr = BSTR::from(method_name);
    let mut dispid = i32::default();
    
    // String을 PCWSTR로 변환
    let method_wide: Vec<u16> = method_bstr.to_string().encode_utf16().chain(std::iter::once(0)).collect();
    let method_pcwstr = PCWSTR::from_raw(method_wide.as_ptr());
    
    obj.GetIDsOfNames(
        &GUID::zeroed(),
        &method_pcwstr,
        1,
        0,
        &mut dispid,
    )?;
    
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
        &disp_params as *const DISPPARAMS,
        Some(&mut result as *mut VARIANT),
        Some(&mut excep_info as *mut EXCEPINFO),
        Some(&mut arg_err as *mut u32),
    )?;
    
    Ok(result)
}

fn main() {
    match execute_with_output() {
        Ok(output) => {
            println!("=== Command Output ===");
            println!("{}", output.trim());
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}