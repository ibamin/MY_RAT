use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{timeout, Duration};
use std::error::Error;
use futures::future::join_all;

pub async fn port_scan(ip: &str, port: i32) -> Result<bool, Box<dyn Error>> {
    let addr = format!("{}:{}", ip, port);

    // 타임아웃 포함 연결 시도
    if let Ok(Ok(mut stream)) = timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
        // 요청 전송
        stream.write_all(b"Hello, Server!\n").await?;

        // 응답 읽기
        let mut response = Vec::new();
        if let Ok(Ok(_)) = timeout(Duration::from_secs(5), stream.read_to_end(&mut response)).await {
            let response_str = String::from_utf8_lossy(&response);
            if !response_str.is_empty(){
                println!(" 포트 : {}\n 활동 서비스: {} \n", port, response_str);
            }else{
                println!("포트 : {}\n 활동 서비스 없음 \n",port);
            }
        }else{
            println!("포트 {} 연결 실패\n 활동 서비스 없음\n",port);
        }

        return Ok(true); // 포트가 열려 있음
    }

    Ok(false) // 포트가 닫혀 있음
}



pub async fn multi_port_scan(ip: &str, port_range: &str) -> Result<(), Box<dyn std::error::Error + Send>> {
    let mut tasks = Vec::new();
    let split_port: Vec<&str> = port_range.split('-').collect();
    let start_port: i32 = split_port[0].parse().expect("포트 범위의 시작값을 파싱할 수 없습니다.");
    let end_port: i32 = split_port[1].parse().expect("포트 범위의 끝값을 파싱할 수 없습니다.");

    let split_ip: Vec<&str> = ip.split('-').collect();
    if split_ip.len() == 2 {
        let base_ip = split_ip[0];
        let end_ip: i32 = split_ip[1].split('.').last().unwrap().parse().expect("IP 범위를 파싱할 수 없습니다.");

        let mut base_parts: Vec<&str> = base_ip.split('.').collect();
        let start_ip: i32 = base_parts.pop()
            .expect("IP 주소의 마지막 부분을 추출할 수 없습니다.")
            .parse()
            .expect("IP 주소의 마지막 부분을 파싱할 수 없습니다.");

        let ip_prefix = base_parts.join(".");

        for i in start_ip..=end_ip {
            let search_ip = format!("{}.{}", ip_prefix, i);
            for port in start_port..=end_port {
                let search_ip_clone = search_ip.clone();
                tasks.push(tokio::spawn(async move {
                    if let Err(e) = port_scan(&search_ip_clone, port).await {
                        Err::<(), String>(format!("포트 {} 스캔 실패: {:?}", port, e))
                    } else {
                        Ok::<(), String>(())
                    }
                }));
            }
        }
    } else {
        for port in start_port..=end_port {
            let ip = ip.to_string();
            tasks.push(tokio::spawn(async move {
                if let Err(e) = port_scan(&ip, port).await {
                    Err::<(), String>(format!("포트 {} 스캔 실패: {:?}", port, e))
                } else {
                    Ok::<(), String>(())
                }
            }));
        }
    }

    let results = join_all(tasks).await;

    Ok(())
}