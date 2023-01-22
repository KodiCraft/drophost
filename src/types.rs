use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    pub hostname: String,
    pub ip: String,
}

impl Host {
    pub fn new(hostname: String, ip: String) -> Self {
        Host { hostname, ip }
    }

    pub fn parse_entry(entry: &str) -> Option<Self> {
        let mut parts = entry.split_whitespace();
        if parts.clone().count() != 2 {
            return None;
        }
        let ip = parts.next()?;
        let hostname = parts.next()?;
        Some(Host::new(hostname.to_string(), ip.to_string()))
    }

    pub fn to_string(&self) -> String {
        format!("{}\t{}", self.ip, self.hostname)
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub struct Hosts {
    pub hosts: Vec<Host>,
}

impl Hosts {
    pub fn new() -> Self {
        Hosts { hosts: vec![] }
    }

    pub fn from(entries: Vec<Host>) -> Self {
        Hosts { hosts: entries }
    }

    pub fn add(&mut self, host: Host) {
        self.hosts.push(host);
    }

    pub fn extend(&mut self, hosts: &Hosts) {
        self.hosts.extend(hosts.hosts.clone());
    }

    pub fn remove(&mut self, host: &Host) {
        self.hosts.retain(|h| h != host);
    }

    pub fn to_string(&self) -> String {
        self.hosts
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}