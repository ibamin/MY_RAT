use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::System::Variant::*,
};

pub fn execute_with_output(Command : String, Output : bool) -> windows::core::Result<String> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();

        let clsid = GUID::from_u128(0x72C24DD5_D70A_438B_8A42_98424B88AFB8);
        let shell: IDispatch = CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER)?;

        // Exec 메서드 호출
        let command = format!("powershell.exe -Command \"{}\"", Command);
        let output = execute_command(&shell, &command, Output)?;
        
        CoUninitialize();
        Ok(output)
    }
}

pub unsafe fn execute_command(shell: &IDispatch, command: &str , Output : bool) -> windows::core::Result<String> {
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

    // Exec 메서드 호출
    let invoke_result = shell.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD,
        &disp_params as *const DISPPARAMS,
        Some(&mut result as *mut VARIANT),
        Some(&mut excep_info as *mut EXCEPINFO),
        Some(&mut arg_err as *mut u32),
    );

    // Exec 메서드 호출 실패 처리
    if invoke_result.is_err() {
        println!("❌ Exec 메서드 호출 실패: {:?}", invoke_result);
        return Err(invoke_result.unwrap_err());
    }

    // 결과 타입 확인
    if result.Anonymous.Anonymous.vt != VT_DISPATCH {
        println!("❌ 예상하지 못한 반환 타입: {:?}", result.Anonymous.Anonymous.vt);
        return Ok(String::new());
    }

    // 출력 캡처가 필요하지 않으면 빈 문자열 반환
    if !Output {
        println!("✅ 명령어 실행 완료 (출력 캡처 없음)");
        return Ok(String::new());
    }

    // WshExec 객체에서 StdOut 읽기
    let exec_obj = &result.Anonymous.Anonymous.Anonymous.pdispVal;
    if let Some(exec_dispatch) = exec_obj.as_ref() {
        
        // 프로세스 완료 대기
        let mut timeout_count = 0;
        const MAX_TIMEOUT: u32 = 60; // 30초 타임아웃
        
        loop {
            let status_result = get_property_simple(exec_dispatch, "Status");
            match status_result {
                Ok(status) => {
                    if status.Anonymous.Anonymous.vt == VT_I4 {
                        if status.Anonymous.Anonymous.Anonymous.lVal == 1 {  // WshFinished
                            break;
                        }
                    }
                },
                Err(_) => {
                    println!("❌ Status 속성 읽기 실패");
                    return Err(windows::core::Error::new(
                        windows::Win32::Foundation::E_FAIL,
                        "Status 속성 읽기 실패"
                    ));
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(500));
            print!(".");
            timeout_count += 1;
            
            if timeout_count > MAX_TIMEOUT {
                println!("\n❌ 프로세스 타임아웃 (30초 초과)");
                return Ok(String::new());
            }
        }
        println!();
        
        // StdOut 읽기
        let stdout_result = get_property_simple(exec_dispatch, "StdOut")?;
        
        if stdout_result.Anonymous.Anonymous.vt == VT_DISPATCH {
            let stdout_obj = &stdout_result.Anonymous.Anonymous.Anonymous.pdispVal;
            if let Some(stdout_dispatch) = stdout_obj.as_ref() {
                
                let output = call_method_simple(stdout_dispatch, "ReadAll", &[])?;
                if output.Anonymous.Anonymous.vt == VT_BSTR {
                    println!("✅ 명령어 실행 및 출력 캡처 완료");
                    return Ok(output.Anonymous.Anonymous.Anonymous.bstrVal.to_string());
                }
            }
        }
        
        // StdErr 확인 (오류가 있는 경우)
        let stderr_result = get_property_simple(exec_dispatch, "StdErr");
        if let Ok(stderr) = stderr_result {
            if stderr.Anonymous.Anonymous.vt == VT_DISPATCH {
                let stderr_obj = &stderr.Anonymous.Anonymous.Anonymous.pdispVal;
                if let Some(stderr_dispatch) = stderr_obj.as_ref() {
                    let error_output = call_method_simple(stderr_dispatch, "ReadAll", &[])?;
                    if error_output.Anonymous.Anonymous.vt == VT_BSTR {
                        let error_text = error_output.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
                        if !error_text.is_empty() {
                            println!("⚠️ 명령어 실행 중 오류 발생: {}", error_text);
                        }
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

#[test]
pub fn Com_Execute_Output_Test() {
    match execute_with_output("whoami; Start-Sleep 2; ipconfig".to_string(), true) {
        Ok(output) => {
            println!("=== Com_Execute_Output_Test() Result Command Output ===");
            println!("{}", output.trim());
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

#[test]
pub fn Com_Execute_No_Output_Test() {
    match execute_with_output("whoami".to_string(), false) {
        Ok(output) => {
            println!("=== Com_Execute_No_Output_Test() Result Command Output ===");
            println!("{}", output.trim());
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}