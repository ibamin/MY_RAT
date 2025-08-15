use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

// 간단한 프로세스 생성 함수 (Windows API 직접 사용)
pub fn Syscall_CreateProcessW(path: &str, args: Option<&str>, hidden: bool) -> Result<()> {
    unsafe {
        // 문자열을 UTF-16으로 변환
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let path_ptr = PWSTR::from_raw(path_wide.as_ptr() as *mut u16);
        
        let command_line = if let Some(arg) = args {
            let arg_wide: Vec<u16> = arg.encode_utf16().chain(std::iter::once(0)).collect();
            Some(PWSTR::from_raw(arg_wide.as_ptr() as *mut u16))
        } else {
            None
        };
        
        // STARTUPINFOW 구조체 초기화
        let mut startup_info = STARTUPINFOW::default();
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        
        // 숨겨진 프로세스 설정
        if hidden {
            startup_info.dwFlags = STARTF_USESHOWWINDOW;
            startup_info.wShowWindow = SW_HIDE.0 as u16;
        }
        
        // PROCESS_INFORMATION 구조체 초기화
        let mut process_info = PROCESS_INFORMATION::default();
        
        // CreateProcessW 호출 (Windows API)
        let result = windows::Win32::System::Threading::CreateProcessW(
            path_ptr,                    // 애플리케이션 이름
            command_line,                // 명령줄 인수
            None,                        // 프로세스 보안 속성
            None,                        // 스레드 보안 속성
            false,                       // 핸들 상속 안함
            if hidden { CREATE_NO_WINDOW } else { CREATE_NEW_CONSOLE },  // 생성 플래그
            None,                        // 환경 변수
            None,                        // 작업 디렉터리
            &startup_info,               // 시작 정보
            &mut process_info,           // 프로세스 정보
        );
        
        match result {
            Ok(_) => {
                println!("✅ 프로세스 생성 성공! PID: {}", process_info.dwProcessId);
                
                // 핸들 정리
                CloseHandle(process_info.hProcess);
                CloseHandle(process_info.hThread);
                
                Ok(())
            },
            Err(e) => {
                println!("❌ 프로세스 생성 실패!");
                Err(e)
            }
        }
    }
}

pub fn Syscall_PowerShell_Execute(Command: String) -> Result<String> {
    unsafe {
        // 임시 파일 경로 생성
        let temp_dir = std::env::temp_dir();
        let output_file = temp_dir.join(format!("ps_output_{}.txt", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
        
        // 명령어를 UTF-16으로 변환 (출력을 파일로 리다이렉트)
        let command = format!("powershell.exe -Command \"{} | Out-File -FilePath '{}' -Encoding UTF8\"", Command, output_file.to_string_lossy());
        let command_wide: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
        let command_ptr = PWSTR::from_raw(command_wide.as_ptr() as *mut u16);
        
        // STARTUPINFOW 구조체 초기화 (숨겨진 창으로 실행)
        let mut startup_info = STARTUPINFOW::default();
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        startup_info.dwFlags = STARTF_USESHOWWINDOW;
        startup_info.wShowWindow = SW_HIDE.0 as u16;
        
        // PROCESS_INFORMATION 구조체 초기화
        let mut process_info = PROCESS_INFORMATION::default();
        
        // CreateProcessW 호출
        let result = windows::Win32::System::Threading::CreateProcessW(
            None,                        // 애플리케이션 이름 (None = command_line에서 추출)
            Some(command_ptr),           // 명령줄
            None,                        // 프로세스 보안 속성
            None,                        // 스레드 보안 속성
            false,                       // 핸들 상속 안함
            CREATE_NO_WINDOW,            // 창 생성 안함
            None,                        // 환경 변수
            None,                        // 작업 디렉터리
            &startup_info,               // 시작 정보
            &mut process_info,           // 프로세스 정보
        );
        
        match result {
            Ok(_) => {
                println!("✅ PowerShell 프로세스 생성 성공! PID: {}", process_info.dwProcessId);
                
                // 프로세스 완료 대기
                let wait_result = WaitForSingleObject(process_info.hProcess, 30000); // 30초 타임아웃
                
                let mut output = String::new();
                
                if wait_result == WAIT_OBJECT_0 {
                    // 파일에서 출력 읽기
                    match std::fs::read_to_string(&output_file) {
                        Ok(content) => {
                            output = content;
                        },
                        Err(e) => {
                            output = format!("[ERROR] 출력 파일 읽기 실패: {:?}", e);
                        }
                    }
                    
                    // 임시 파일 삭제
                    let _ = std::fs::remove_file(&output_file);
                } else {
                    output = format!("[TIMEOUT] 프로세스가 30초 내에 완료되지 않았습니다. WaitResult: {:?}", wait_result);
                }
                
                // 핸들 정리
                CloseHandle(process_info.hProcess);
                CloseHandle(process_info.hThread);
                
                Ok(output)
            },
            Err(e) => {
                println!("❌ PowerShell 프로세스 생성 실패!");
                Err(e)
            }
        }
    }
}

// 간단한 사용 예시 함수들
pub fn create_notepad_process() -> Result<()> {
    Syscall_CreateProcessW("C:\\Windows\\System32\\notepad.exe", None, false)
}

pub fn create_calculator_process() -> Result<()> {
    Syscall_CreateProcessW("C:\\Windows\\System32\\calc.exe", None, false)
}

pub fn create_powershell_process_with_args() -> Result<()> {
    Syscall_CreateProcessW("powershell.exe", Some("C:\\Windows\\System32\\powershell.exe -Command \"Get-Process | Select-Object -First 5\""), false)
}

pub fn create_hidden_process() -> Result<()> {
    Syscall_CreateProcessW("C:\\Windows\\System32\\cmd.exe", Some("cmd.exe /c dir C:\\"), true)
}

// 테스트 함수들
#[test]
pub fn test_create_notepad() {
    match create_notepad_process() {
        Ok(_) => println!("✅ Notepad 테스트 성공"),
        Err(e) => println!("❌ Notepad 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_create_calculator() {
    match create_calculator_process() {
        Ok(_) => println!("✅ Calculator 테스트 성공"),
        Err(e) => println!("❌ Calculator 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_create_powershell() {
    match create_powershell_process_with_args() {
        Ok(_) => println!("✅ PowerShell 테스트 성공"),
        Err(e) => println!("❌ PowerShell 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_create_hidden_process() {
    match create_hidden_process() {
        Ok(_) => println!("✅ 숨겨진 프로세스 테스트 성공"),
        Err(e) => println!("❌ 숨겨진 프로세스 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_powershell_execute() {
    match Syscall_PowerShell_Execute("whoami".to_string()) {
        Ok(output) => {
            println!("✅ PowerShell 실행 테스트 성공");
            println!("=== 출력 결과 ===");
            println!("{}", output);
        },
        Err(e) => println!("❌ PowerShell 실행 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_powershell_ipconfig() {
    match Syscall_PowerShell_Execute("ipconfig".to_string()) {
        Ok(output) => {
            println!("✅ PowerShell ipconfig 테스트 성공");
            println!("=== 출력 결과 ===");
            println!("{}", output);
        },
        Err(e) => println!("❌ PowerShell ipconfig 테스트 실패: {:?}", e),
    }
}

#[test]
pub fn test_powershell_get_process() {
    match Syscall_PowerShell_Execute("Get-Process | Select-Object -First 3".to_string()) {
        Ok(output) => {
            println!("✅ PowerShell Get-Process 테스트 성공");
            println!("=== 출력 결과 ===");
            println!("{}", output);
        },
        Err(e) => println!("❌ PowerShell Get-Process 테스트 실패: {:?}", e),
    }
}