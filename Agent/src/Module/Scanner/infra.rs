use ldap3::{LdapConn, Scope, SearchEntry, LdapResult};
use ldap3::result::Result;

pub struct ActiveDirectoryScanner {
    connection: Option<LdapConn>,
    domain_controller: String,
    domain_name: String,
    username: String,
    password: String,
}

impl ActiveDirectoryScanner {
    pub fn new(domain_controller: String, domain_name: String, username: String, password: String) -> Self {
        Self {
            connection: None,
            domain_controller,
            domain_name,
            username,
            password,
        }
    }
    
    // 도메인 컨트롤러 자동 감지
    pub fn new_auto_detect(domain_name: String, username: String, password: String) -> Self {
        let domain_controller = format!("{}.{}", domain_name.to_uppercase(), domain_name);
        Self::new(domain_controller, domain_name, username, password)
    }
    
    pub fn connect(&mut self) -> Result<()> {
        let ldap_url = format!("ldap://{}:389", self.domain_controller);
        println!("🔗 Active Directory 연결 중: {}", ldap_url);
        
        match LdapConn::new(&ldap_url) {
            Ok(conn) => {
                self.connection = Some(conn);
                println!("✅ Active Directory 연결 성공");
                Ok(())
            },
            Err(e) => {
                println!("❌ Active Directory 연결 실패: {:?}", e);
                Err(e)
            }
        }
    }
    
    pub fn bind(&mut self) -> Result<()> {
        if let Some(ref mut conn) = self.connection {
            let bind_dn = format!("{}\\{}", self.domain_name, self.username);
            println!("🔐 Active Directory 바인딩 중: {}", bind_dn);
            
            match conn.simple_bind(&bind_dn, &self.password) {
                Ok(_) => {
                    println!("✅ Active Directory 바인딩 성공");
                    Ok(())
                },
                Err(e) => {
                    println!("❌ Active Directory 바인딩 실패: {:?}", e);
                    Err(e)
                }
            }
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 도메인 정보 조회
    pub fn get_domain_info(&mut self) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            println!("🔍 도메인 정보 조회 중: {}", base_dn);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Base,
                "(objectClass=domain)",
                vec!["dc", "distinguishedName", "whenCreated", "whenChanged"]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ 도메인 정보 조회 완료");
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 사용자 계정 조회
    pub fn search_users(&mut self) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            println!("🔍 사용자 계정 조회 중: {}", base_dn);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                "(&(objectClass=user)(objectCategory=person))",
                vec![
                    "sAMAccountName",      // 로그인 이름
                    "userPrincipalName",   // UPN
                    "cn",                  // 표시 이름
                    "displayName",         // 표시 이름
                    "mail",                // 이메일
                    "memberOf",            // 소속 그룹
                    "userAccountControl",  // 계정 상태
                    "pwdLastSet",          // 마지막 비밀번호 변경
                    "lastLogon",           // 마지막 로그인
                    "logonCount",          // 로그인 횟수
                    "description",         // 설명
                    "department",          // 부서
                    "title",               // 직책
                    "manager",             // 관리자
                    "whenCreated",         // 생성일
                    "whenChanged"          // 수정일
                ]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ {}명의 사용자를 찾았습니다.", entries.len());
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 그룹 조회
    pub fn search_groups(&mut self) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            println!("🔍 그룹 조회 중: {}", base_dn);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                "(objectClass=group)",
                vec![
                    "cn",                  // 그룹 이름
                    "sAMAccountName",      // 그룹 계정명
                    "description",         // 설명
                    "member",              // 그룹 멤버
                    "memberOf",            // 상위 그룹
                    "groupType",           // 그룹 타입
                    "whenCreated",         // 생성일
                    "whenChanged"          // 수정일
                ]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ {}개의 그룹을 찾았습니다.", entries.len());
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 컴퓨터 계정 조회
    pub fn search_computers(&mut self) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            println!("🔍 컴퓨터 계정 조회 중: {}", base_dn);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                "(objectClass=computer)",
                vec![
                    "cn",                      // 컴퓨터 이름
                    "dNSHostName",             // DNS 호스트명
                    "operatingSystem",         // 운영체제
                    "operatingSystemVersion",  // OS 버전
                    "operatingSystemServicePack", // OS 서비스팩
                    "lastLogon",               // 마지막 로그인
                    "logonCount",              // 로그인 횟수
                    "description",             // 설명
                    "location",                // 위치
                    "whenCreated",             // 생성일
                    "whenChanged"              // 수정일
                ]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ {}대의 컴퓨터를 찾았습니다.", entries.len());
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // OU(조직 단위) 조회
    pub fn search_organizational_units(&mut self) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            println!("🔍 조직 단위(OU) 조회 중: {}", base_dn);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                "(objectClass=organizationalUnit)",
                vec![
                    "ou",                 // OU 이름
                    "description",        // 설명
                    "whenCreated",        // 생성일
                    "whenChanged"         // 수정일
                ]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ {}개의 조직 단위를 찾았습니다.", entries.len());
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 특정 사용자 검색
    pub fn search_user_by_name(&mut self, username: &str) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            let filter = format!("(&(objectClass=user)(sAMAccountName={}))", username);
            println!("🔍 사용자 검색 중: {}", username);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                &filter,
                vec![
                    "sAMAccountName", "userPrincipalName", "cn", "displayName",
                    "mail", "memberOf", "userAccountControl", "lastLogon"
                ]
            )?.success()?;
            
            let entries: Vec<SearchEntry> = rs.into_iter()
                .map(|entry| SearchEntry::construct(entry))
                .collect();
            
            println!("✅ {}명의 사용자를 찾았습니다.", entries.len());
            Ok(entries)
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    // 특정 그룹의 멤버 조회
    pub fn search_group_members(&mut self, group_name: &str) -> Result<Vec<SearchEntry>> {
        if let Some(ref mut conn) = self.connection {
            let base_dn = format!("DC={}", self.domain_name.replace(".", ",DC="));
            let filter = format!("(&(objectClass=group)(cn={}))", group_name);
            println!("🔍 그룹 멤버 조회 중: {}", group_name);
            
            let (rs, _res) = conn.search(
                &base_dn,
                Scope::Subtree,
                &filter,
                vec!["member"]
            )?.success()?;
            
            if let Some(group_entry) = rs.first() {
                let group = SearchEntry::construct(group_entry.clone());
                if let Some(members) = group.attrs.get("member") {
                    println!("✅ 그룹 '{}'의 멤버 {}명을 찾았습니다.", group_name, members.len());
                    
                    // 멤버들의 상세 정보 조회
                    let mut member_details = Vec::new();
                    for member_dn in members {
                        let member_filter = format!("(distinguishedName={})", member_dn);
                        let (member_rs, _) = conn.search(
                            &base_dn,
                            Scope::Subtree,
                            &member_filter,
                            vec!["sAMAccountName", "cn", "displayName", "mail"]
                        )?.success()?;
                        
                        for member_entry in member_rs {
                            member_details.push(SearchEntry::construct(member_entry));
                        }
                    }
                    
                    return Ok(member_details);
                }
            }
            
            Ok(Vec::new())
        } else {
            println!("❌ Active Directory 연결이 없습니다.");
            Err(ldap3::result::Error::Other("No connection".to_string()))
        }
    }
    
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            println!("🔌 Active Directory 연결 해제됨");
        }
    }
}

// 테스트 함수들
#[test]
pub fn test_active_directory_connection() {
    let mut scanner = ActiveDirectoryScanner::new_auto_detect(
        "example.com".to_string(),
        "administrator".to_string(),
        "password".to_string(),
    );
    
    match scanner.connect() {
        Ok(_) => {
            println!("✅ Active Directory 연결 테스트 성공");
            match scanner.bind() {
                Ok(_) => {
                    println!("✅ Active Directory 바인딩 테스트 성공");
                    
                    // 도메인 정보 조회
                    match scanner.get_domain_info() {
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
                    
                    // 사용자 조회
                    match scanner.search_users() {
                        Ok(users) => {
                            println!("=== 사용자 목록 (처음 5명) ===");
                            for (i, user) in users.iter().take(5).enumerate() {
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
                    
                    // 그룹 조회
                    match scanner.search_groups() {
                        Ok(groups) => {
                            println!("=== 그룹 목록 (처음 5개) ===");
                            for (i, group) in groups.iter().take(5).enumerate() {
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
                    
                    // 컴퓨터 조회
                    match scanner.search_computers() {
                        Ok(computers) => {
                            println!("=== 컴퓨터 목록 (처음 5대) ===");
                            for (i, computer) in computers.iter().take(5).enumerate() {
                                println!("[{}] DN: {}", i + 1, computer.dn);
                                if let Some(cn) = computer.attrs.get("cn") {
                                    println!("    컴퓨터명: {:?}", cn);
                                }
                                if let Some(os) = computer.attrs.get("operatingSystem") {
                                    println!("    OS: {:?}", os);
                                }
                                println!("---");
                            }
                        },
                        Err(e) => println!("❌ 컴퓨터 조회 실패: {:?}", e),
                    }
                },
                Err(e) => println!("❌ Active Directory 바인딩 테스트 실패: {:?}", e),
            }
        },
        Err(e) => println!("❌ Active Directory 연결 테스트 실패: {:?}", e),
    }
    
    scanner.disconnect();
}

#[test]
pub fn test_search_specific_user() {
    let mut scanner = ActiveDirectoryScanner::new_auto_detect(
        "example.com".to_string(),
        "administrator".to_string(),
        "password".to_string(),
    );
    
    if let (Ok(_), Ok(_)) = (scanner.connect(), scanner.bind()) {
        match scanner.search_user_by_name("administrator") {
            Ok(users) => {
                println!("=== 특정 사용자 검색 결과 ===");
                for user in users {
                    println!("DN: {}", user.dn);
                    for (attr, values) in &user.attrs {
                        println!("{}: {:?}", attr, values);
                    }
                    println!("---");
                }
            },
            Err(e) => println!("❌ 특정 사용자 검색 실패: {:?}", e),
        }
    }
    
    scanner.disconnect();
}

#[test]
pub fn test_search_group_members() {
    let mut scanner = ActiveDirectoryScanner::new_auto_detect(
        "example.com".to_string(),
        "administrator".to_string(),
        "password".to_string(),
    );
    
    if let (Ok(_), Ok(_)) = (scanner.connect(), scanner.bind()) {
        match scanner.search_group_members("Domain Admins") {
            Ok(members) => {
                println!("=== Domain Admins 그룹 멤버 ===");
                for member in members {
                    println!("DN: {}", member.dn);
                    if let Some(sam) = member.attrs.get("sAMAccountName") {
                        println!("계정명: {:?}", sam);
                    }
                    if let Some(cn) = member.attrs.get("cn") {
                        println!("이름: {:?}", cn);
                    }
                    println!("---");
                }
            },
            Err(e) => println!("❌ 그룹 멤버 검색 실패: {:?}", e),
        }
    }
    
    scanner.disconnect();
} 