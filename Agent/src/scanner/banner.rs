use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerResult {
    pub host: String,
    pub port: u16,
    pub banner_raw: String,
    pub service_hint: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct BannerScanner {
    read_timeout: Duration,
    max_banner_size: usize,
}

impl Default for BannerScanner {
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(3),
            max_banner_size: 4096,
        }
    }
}

impl BannerScanner {
    pub fn new(read_timeout_ms: u64) -> Self {
        let read_timeout = if read_timeout_ms == 0 {
            Duration::from_secs(3)
        } else {
            Duration::from_millis(read_timeout_ms)
        };

        Self {
            read_timeout,
            max_banner_size: 4096,
        }
    }

    pub async fn grab_banner(&self, host: &str, port: u16) -> Option<BannerResult> {
        let mut stream = match timeout(self.read_timeout, TcpStream::connect((host, port))).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(_)) | Err(_) => return None,
        };

        let mut buffer = vec![0_u8; self.max_banner_size];

        if matches!(port, 80 | 443 | 8080 | 8443) {
            let probe = format!("GET / HTTP/1.0\r\nHost: {host}\r\n\r\n");
            if stream.write_all(probe.as_bytes()).await.is_err() {
                return None;
            }
        }

        let read_len = match timeout(self.read_timeout, stream.read(&mut buffer)).await {
            Ok(Ok(size)) => size,
            Ok(Err(_)) | Err(_) => return None,
        };

        if read_len == 0 {
            return None;
        }

        let banner_raw = String::from_utf8_lossy(&buffer[..read_len]).trim().to_string();
        if banner_raw.is_empty() {
            return None;
        }

        Some(BannerResult {
            host: host.to_string(),
            port,
            service_hint: infer_service_hint(port, &banner_raw),
            banner_raw,
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    pub async fn scan_banners(
        &self,
        host: &str,
        ports: &[u16],
        concurrency: usize,
    ) -> Vec<BannerResult> {
        if ports.is_empty() {
            return Vec::new();
        }

        let concurrency = if concurrency == 0 { 100 } else { concurrency };
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut tasks = Vec::with_capacity(ports.len());

        for &port in ports {
            let scanner = self.clone();
            let host = host.to_string();
            let semaphore = Arc::clone(&semaphore);

            let task = tokio::spawn(async move {
                let permit = semaphore.acquire_owned().await;
                if permit.is_err() {
                    return None;
                }

                scanner.grab_banner(&host, port).await
            });

            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            if let Ok(Some(result)) = task.await {
                results.push(result);
            }
        }

        results
    }
}

fn infer_service_hint(port: u16, banner: &str) -> String {
    let banner_upper = banner.to_ascii_uppercase();
    let banner_lower = banner.to_ascii_lowercase();

    if banner_upper.starts_with("SSH-") {
        return "ssh".to_string();
    }
    if banner_upper.starts_with("HTTP/") || banner_lower.contains("http/") {
        return "http".to_string();
    }
    if banner_upper.starts_with("220 ") {
        if port == 21 || banner_lower.contains("ftp") {
            return "ftp".to_string();
        }
        if port == 25 || port == 587 || banner_lower.contains("smtp") {
            return "smtp".to_string();
        }
        return "smtp/ftp".to_string();
    }

    match port {
        21 => "ftp".to_string(),
        22 => "ssh".to_string(),
        25 | 587 => "smtp".to_string(),
        80 | 443 | 8080 | 8443 => "http".to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_ssh_banner() {
        assert_eq!(infer_service_hint(22, "SSH-2.0-OpenSSH_8.9"), "ssh");
    }

    #[test]
    fn test_infer_http_banner() {
        assert_eq!(infer_service_hint(80, "HTTP/1.1 200 OK"), "http");
    }

    #[test]
    fn test_infer_ftp_220_banner() {
        assert_eq!(infer_service_hint(21, "220 FTP server ready"), "ftp");
    }

    #[test]
    fn test_infer_smtp_220_banner() {
        assert_eq!(infer_service_hint(25, "220 mail.example.com SMTP"), "smtp");
    }

    #[test]
    fn test_infer_220_ambiguous() {
        assert_eq!(infer_service_hint(8080, "220 Something"), "smtp/ftp");
    }

    #[test]
    fn test_infer_by_port_21() {
        assert_eq!(infer_service_hint(21, "some random banner"), "ftp");
    }

    #[test]
    fn test_infer_by_port_22() {
        assert_eq!(infer_service_hint(22, "some random banner"), "ssh");
    }

    #[test]
    fn test_infer_by_port_80() {
        assert_eq!(infer_service_hint(80, "some random banner"), "http");
    }

    #[test]
    fn test_infer_by_port_443() {
        assert_eq!(infer_service_hint(443, "some random banner"), "http");
    }

    #[test]
    fn test_infer_unknown_port() {
        assert_eq!(infer_service_hint(9999, "some random banner"), "unknown");
    }

    #[test]
    fn test_banner_scanner_default() {
        let scanner = BannerScanner::default();
        assert_eq!(scanner.read_timeout, Duration::from_secs(3));
        assert_eq!(scanner.max_banner_size, 4096);
    }

    #[test]
    fn test_banner_scanner_new_zero_timeout() {
        let scanner = BannerScanner::new(0);
        assert_eq!(scanner.read_timeout, Duration::from_secs(3));
    }

    #[test]
    fn test_banner_scanner_new_custom_timeout() {
        let scanner = BannerScanner::new(5000);
        assert_eq!(scanner.read_timeout, Duration::from_millis(5000));
    }

    #[test]
    fn test_banner_result_serde() {
        let result = BannerResult {
            host: "10.0.0.1".into(),
            port: 22,
            banner_raw: "SSH-2.0-OpenSSH".into(),
            service_hint: "ssh".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: BannerResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.host, "10.0.0.1");
        assert_eq!(decoded.port, 22);
        assert_eq!(decoded.service_hint, "ssh");
    }
}
