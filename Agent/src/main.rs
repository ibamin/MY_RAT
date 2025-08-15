mod Module;

use Module::Executor::COM;
use Module::Executor::SYSCALL;

fn main() {
    println!("=== SYSCALL 모듈 테스트 ===");
    match SYSCALL::create_notepad_process() {
        Ok(_) => println!("✅ Notepad 실행 성공"),
        Err(e) => println!("❌ Notepad 실행 실패: {:?}", e),
    }
    
    match SYSCALL::create_calculator_process() {
        Ok(_) => println!("✅ Calculator 실행 성공"),
        Err(e) => println!("❌ Calculator 실행 실패: {:?}", e),
    }
    
    match SYSCALL::create_hidden_process() {
        Ok(_) => println!("✅ 숨겨진 프로세스 실행 성공"),
        Err(e) => println!("❌ 숨겨진 프로세스 실행 실패: {:?}", e),
    }
    
    println!("\n=== PowerShell 실행 테스트 ===");
    match SYSCALL::Syscall_PowerShell_Execute("whoami".to_string()) {
        Ok(output) => {
            println!("✅ PowerShell 실행 성공");
            println!("=== 출력 결과 ===");
            println!("{}", output);
        },
        Err(e) => println!("❌ PowerShell 실행 실패: {:?}", e),
    }
    
    println!("\n=== PowerShell ipconfig 테스트 ===");
    match SYSCALL::Syscall_PowerShell_Execute("ipconfig".to_string()) {
        Ok(output) => {
            println!("✅ PowerShell ipconfig 실행 성공");
            println!("=== 출력 결과 ===");
            println!("{}", output);
        },
        Err(e) => println!("❌ PowerShell ipconfig 실행 실패: {:?}", e),
    }
}