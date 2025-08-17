mod Module;

use Module::Executor::COM;
use Module::Executor::SYSCALL;
use Module::Scanner::infra::ActiveDirectoryScanner;

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
    
    println!("\n=== Active Directory 스캐너 테스트 ===");
    let mut ad_scanner = ActiveDirectoryScanner::new_auto_detect(
        "example.com".to_string(),
        "administrator".to_string(),
        "password".to_string(),
    );
    
    match ad_scanner.connect() {
        Ok(_) => {
            println!("✅ Active Directory 연결 성공");
            match ad_scanner.bind() {
                Ok(_) => {
                    println!("✅ Active Directory 바인딩 성공");
                    
                    // 도메인 정보 조회
                    match ad_scanner.get_domain_info() {
                        Ok(domain_info) => {
                            println!("=== 도메인 정보 ===");
                            for info in domain_info {
                                println!("DN: {}", info.dn);
                                for (attr, values) in &info.attrs {
                                    println!("{}: {:?}", attr, values);
                                }
                                println!("---");
                            }
                        },
                        Err(e) => println!("❌ 도메인 정보 조회 실패: {:?}", e),
                    }
                    
                    // 사용자 조회 (처음 3명만)
                    match ad_scanner.search_users() {
                        Ok(users) => {
                            println!("=== Active Directory 사용자 목록 (처음 3명) ===");
                            for (i, user) in users.iter().take(3).enumerate() {
                                println!("[{}] DN: {}", i + 1, user.dn);
                                if let Some(sam) = user.attrs.get("sAMAccountName") {
                                    println!("    계정명: {:?}", sam);
                                }
                                if let Some(cn) = user.attrs.get("cn") {
                                    println!("    이름: {:?}", cn);
                                }
                                if let Some(mail) = user.attrs.get("mail") {
                                    println!("    이메일: {:?}", mail);
                                }
                                println!("---");
                            }
                        },
                        Err(e) => println!("❌ 사용자 조회 실패: {:?}", e),
                    }
                    
                    // 그룹 조회 (처음 3개만)
                    match ad_scanner.search_groups() {
                        Ok(groups) => {
                            println!("=== Active Directory 그룹 목록 (처음 3개) ===");
                            for (i, group) in groups.iter().take(3).enumerate() {
                                println!("[{}] DN: {}", i + 1, group.dn);
                                if let Some(cn) = group.attrs.get("cn") {
                                    println!("    그룹명: {:?}", cn);
                                }
                                if let Some(desc) = group.attrs.get("description") {
                                    println!("    설명: {:?}", desc);
                                }
                                println!("---");
                            }
                        },
                        Err(e) => println!("❌ 그룹 조회 실패: {:?}", e),
                    }
                },
                Err(e) => println!("❌ Active Directory 바인딩 실패: {:?}", e),
            }
        },
        Err(e) => println!("❌ Active Directory 연결 실패: {:?}", e),
    }
    
    ad_scanner.disconnect();
}