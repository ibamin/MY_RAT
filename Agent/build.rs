fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn main() {
    let guid = env_or("AGENT_GUID", "dev-agent-no-guid");
    let server_url = env_or("AGENT_SERVER_URL", "http://127.0.0.1:3000");
    let sleep_sec = env_or("AGENT_SLEEP_SEC", "5");

    println!("cargo:rustc-env=EMBEDDED_GUID={guid}");
    println!("cargo:rustc-env=EMBEDDED_SERVER_URL={server_url}");
    println!("cargo:rustc-env=EMBEDDED_SLEEP_SEC={sleep_sec}");
}
