#[cfg(windows)]
use ldap3::{result::Result, LdapConn, Scope, SearchEntry};
#[cfg(windows)]
use serde::{Deserialize, Serialize};

#[cfg(windows)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdConfig {
    pub domain_controller: String,
    pub domain_name: String,
    pub username: String,
    pub password: String,
    pub base_dn: Option<String>,
    pub ldap_port: u16,
    pub use_ssl: bool,
}

#[cfg(windows)]
impl AdConfig {
    pub fn from_params(params_json: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(params_json)
    }

    pub fn from_env() -> Self {
        let domain_name = std::env::var("AD_DOMAIN").unwrap_or_default();
        let domain_controller = std::env::var("AD_DOMAIN_CONTROLLER")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| domain_name.clone());

        let username = std::env::var("AD_USERNAME").unwrap_or_default();
        let password = std::env::var("AD_PASSWORD").unwrap_or_default();
        let base_dn = std::env::var("AD_BASE_DN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let ldap_port = std::env::var("AD_LDAP_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(389);
        let use_ssl = std::env::var("AD_USE_SSL")
            .ok()
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

        Self {
            domain_controller,
            domain_name,
            username,
            password,
            base_dn,
            ldap_port,
            use_ssl,
        }
    }

    pub fn base_dn(&self) -> String {
        self.base_dn
            .clone()
            .unwrap_or_else(|| format!("DC={}", self.domain_name.replace('.', ",DC=")))
    }

    pub fn ldap_url(&self) -> String {
        let scheme = if self.use_ssl { "ldaps" } else { "ldap" };
        format!("{}://{}:{}", scheme, self.domain_controller, self.ldap_port)
    }
}

#[cfg(windows)]
impl Default for AdConfig {
    fn default() -> Self {
        Self {
            domain_controller: String::new(),
            domain_name: String::new(),
            username: String::new(),
            password: String::new(),
            base_dn: None,
            ldap_port: 389,
            use_ssl: false,
        }
    }
}

#[cfg(windows)]
pub struct ActiveDirectoryScanner {
    connection: Option<LdapConn>,
    config: AdConfig,
}

#[cfg(windows)]
impl ActiveDirectoryScanner {
    pub fn new(config: AdConfig) -> Self {
        Self {
            connection: None,
            config,
        }
    }

    pub fn new_auto_detect(domain_name: &str, username: &str, password: &str) -> Self {
        let config = AdConfig {
            domain_controller: domain_name.to_string(),
            domain_name: domain_name.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            ..AdConfig::default()
        };
        Self::new(config)
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }

        let ldap = LdapConn::new(&self.config.ldap_url())?;
        self.connection = Some(ldap);
        Ok(())
    }

    pub fn bind(&mut self) -> Result<()> {
        self.connect()?;

        let bind_username =
            if self.config.username.contains('\\') || self.config.username.contains('@') {
                self.config.username.clone()
            } else if !self.config.domain_name.trim().is_empty() {
                format!("{}\\{}", self.config.domain_name, self.config.username)
            } else {
                self.config.username.clone()
            };

        if let Some(ldap) = self.connection.as_mut() {
            ldap.simple_bind(&bind_username, &self.config.password)?
                .success()?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "LDAP connection unavailable",
            )
            .into())
        }
    }

    pub fn get_domain_info(&mut self) -> Result<Vec<SearchEntry>> {
        self.search_internal("(objectClass=domain)", vec!["*", "+"])
    }

    pub fn search_users(&mut self) -> Result<Vec<SearchEntry>> {
        self.search_internal(
            "(&(objectClass=user)(!(objectClass=computer)))",
            vec![
                "cn",
                "sAMAccountName",
                "mail",
                "memberOf",
                "distinguishedName",
            ],
        )
    }

    pub fn search_groups(&mut self) -> Result<Vec<SearchEntry>> {
        self.search_internal(
            "(objectClass=group)",
            vec!["cn", "sAMAccountName", "member", "distinguishedName"],
        )
    }

    pub fn search_computers(&mut self) -> Result<Vec<SearchEntry>> {
        self.search_internal(
            "(objectClass=computer)",
            vec!["cn", "dNSHostName", "operatingSystem", "distinguishedName"],
        )
    }

    pub fn search_organizational_units(&mut self) -> Result<Vec<SearchEntry>> {
        self.search_internal(
            "(objectClass=organizationalUnit)",
            vec!["ou", "description", "distinguishedName"],
        )
    }

    pub fn search_user_by_name(&mut self, username: &str) -> Result<Vec<SearchEntry>> {
        let escaped = escape_ldap_filter_value(username);
        let filter = format!(
            "(&(objectClass=user)(|(cn={0})(sAMAccountName={0})))",
            escaped
        );
        self.search_internal(
            &filter,
            vec![
                "cn",
                "sAMAccountName",
                "mail",
                "memberOf",
                "distinguishedName",
            ],
        )
    }

    pub fn search_group_members(&mut self, group_name: &str) -> Result<Vec<SearchEntry>> {
        self.bind()?;

        let escaped = escape_ldap_filter_value(group_name);
        let group_filter = format!(
            "(&(objectClass=group)(|(cn={0})(sAMAccountName={0})))",
            escaped
        );
        let base_dn = self.config.base_dn();

        let ldap = self.connection.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "LDAP connection unavailable",
            )
        })?;

        let (group_rs, _) = ldap
            .search(&base_dn, Scope::Subtree, &group_filter, vec!["member"])?
            .success()?;

        let mut members = Vec::new();
        for group_entry in group_rs {
            let entry = SearchEntry::construct(group_entry);
            if let Some(member_dns) = entry.attrs.get("member") {
                for member_dn in member_dns {
                    let (member_rs, _) = ldap
                        .search(member_dn, Scope::Base, "(objectClass=*)", vec!["*", "+"])?
                        .success()?;
                    members.extend(member_rs.into_iter().map(SearchEntry::construct));
                }
            }
        }

        Ok(members)
    }

    pub fn disconnect(&mut self) {
        if let Some(mut conn) = self.connection.take() {
            let _ = conn.unbind();
        }
    }

    fn search_internal(&mut self, filter: &str, attrs: Vec<&str>) -> Result<Vec<SearchEntry>> {
        self.bind()?;

        let base_dn = self.config.base_dn();
        let ldap = self.connection.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "LDAP connection unavailable",
            )
        })?;

        let (rs, _) = ldap
            .search(&base_dn, Scope::Subtree, filter, attrs)?
            .success()?;

        Ok(rs.into_iter().map(SearchEntry::construct).collect())
    }
}

#[cfg(windows)]
fn escape_ldap_filter_value(input: &str) -> String {
    input
        .replace('\\', "\\5c")
        .replace('*', "\\2a")
        .replace('(', "\\28")
        .replace(')', "\\29")
        .replace('\0', "\\00")
}

#[cfg(windows)]
pub fn is_domain_joined() -> bool {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
            "/v",
            "Domain",
        ])
        .output();

    let output = match output {
        Ok(value) if value.status.success() => value,
        _ => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("Domain") && line.contains("REG_SZ") {
            let value = line
                .split("REG_SZ")
                .nth(1)
                .map(str::trim)
                .unwrap_or_default();

            if !value.is_empty() && value != "(value not set)" {
                return true;
            }
        }
    }

    false
}

#[cfg(not(windows))]
pub fn is_domain_joined() -> bool {
    false
}
