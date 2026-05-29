use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

pub const TOP_PORTS: [u16; 21] = [
    21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995, 1723, 3306, 3389,
    5900, 8080, 8443,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortResult {
    pub host: String,
    pub port: u16,
    pub open: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PortScanner {
    timeout: Duration,
    concurrency: usize,
}

impl Default for PortScanner {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            concurrency: 100,
        }
    }
}

impl PortScanner {
    pub fn new(timeout_ms: u64, concurrency: usize) -> Self {
        let timeout = if timeout_ms == 0 {
            Duration::from_secs(3)
        } else {
            Duration::from_millis(timeout_ms)
        };
        let concurrency = if concurrency == 0 { 100 } else { concurrency };

        Self {
            timeout,
            concurrency,
        }
    }

    pub async fn scan(&self, host: &str, ports: &[u16]) -> Vec<PortResult> {
        if ports.is_empty() {
            return Vec::new();
        }

        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut tasks = Vec::with_capacity(ports.len());

        for &port in ports {
            let semaphore = Arc::clone(&semaphore);
            let host = host.to_string();
            let timeout_duration = self.timeout;

            let task = tokio::spawn(async move {
                let start = Instant::now();
                let permit = semaphore.acquire_owned().await;

                if permit.is_err() {
                    return PortResult {
                        host,
                        port,
                        open: false,
                        latency_ms: 0,
                    };
                }

                let connect_result =
                    timeout(timeout_duration, TcpStream::connect((host.as_str(), port))).await;
                let elapsed = start.elapsed().as_millis() as u64;

                match connect_result {
                    Ok(Ok(_stream)) => PortResult {
                        host,
                        port,
                        open: true,
                        latency_ms: elapsed,
                    },
                    Ok(Err(_)) | Err(_) => PortResult {
                        host,
                        port,
                        open: false,
                        latency_ms: elapsed,
                    },
                }
            });

            tasks.push((port, task));
        }

        let mut results = Vec::with_capacity(tasks.len());
        for (port, task) in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(_) => results.push(PortResult {
                    host: host.to_string(),
                    port,
                    open: false,
                    latency_ms: 0,
                }),
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_ports_count() {
        assert_eq!(TOP_PORTS.len(), 21);
    }

    #[test]
    fn test_top_ports_contains_common() {
        assert!(TOP_PORTS.contains(&22));
        assert!(TOP_PORTS.contains(&80));
        assert!(TOP_PORTS.contains(&443));
    }

    #[test]
    fn test_port_scanner_default() {
        let scanner = PortScanner::default();
        assert_eq!(scanner.timeout, Duration::from_secs(3));
        assert_eq!(scanner.concurrency, 100);
    }

    #[test]
    fn test_port_scanner_new_zero_values() {
        let scanner = PortScanner::new(0, 0);
        assert_eq!(scanner.timeout, Duration::from_secs(3));
        assert_eq!(scanner.concurrency, 100);
    }

    #[test]
    fn test_port_scanner_new_custom() {
        let scanner = PortScanner::new(5000, 50);
        assert_eq!(scanner.timeout, Duration::from_millis(5000));
        assert_eq!(scanner.concurrency, 50);
    }

    #[test]
    fn test_port_result_serde() {
        let result = PortResult {
            host: "10.0.0.1".into(),
            port: 80,
            open: true,
            latency_ms: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: PortResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.host, "10.0.0.1");
        assert_eq!(decoded.port, 80);
        assert!(decoded.open);
        assert_eq!(decoded.latency_ms, 42);
    }

    #[tokio::test]
    async fn test_scan_empty_ports() {
        let scanner = PortScanner::default();
        let results = scanner.scan("localhost", &[]).await;
        assert!(results.is_empty());
    }
}
