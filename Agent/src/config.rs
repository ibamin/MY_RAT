const DEFAULT_GUID: &str = "dev-agent-no-guid";
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:3000";
const DEFAULT_SLEEP_SEC: &str = "5";

pub fn guid() -> &'static str {
    option_env!("EMBEDDED_GUID").unwrap_or(DEFAULT_GUID)
}

pub fn server_url() -> &'static str {
    option_env!("EMBEDDED_SERVER_URL").unwrap_or(DEFAULT_SERVER_URL)
}

pub fn sleep_sec() -> u64 {
    option_env!("EMBEDDED_SLEEP_SEC")
        .unwrap_or(DEFAULT_SLEEP_SEC)
        .parse::<u64>()
        .unwrap_or(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_guid() {
        assert_eq!(guid(), "dev-agent-no-guid");
    }

    #[test]
    fn test_default_server_url() {
        assert_eq!(server_url(), "http://127.0.0.1:3000");
    }

    #[test]
    fn test_default_sleep_sec() {
        assert_eq!(sleep_sec(), 5);
    }
}
