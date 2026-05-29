#[cfg(windows)]
pub mod active_directory;
pub mod banner;
pub mod port;

pub use banner::{BannerResult, BannerScanner};
pub use port::{PortResult, PortScanner};

pub async fn run_scan(host: &str, ports: &[u16]) -> Vec<BannerResult> {
    let port_scanner = PortScanner::new(0, 0);
    let port_results = port_scanner.scan(host, ports).await;
    let open_ports: Vec<u16> = port_results
        .into_iter()
        .filter(|result| result.open)
        .map(|result| result.port)
        .collect();

    if open_ports.is_empty() {
        return Vec::new();
    }

    let banner_scanner = BannerScanner::new(0);
    banner_scanner.scan_banners(host, &open_ports, 100).await
}
