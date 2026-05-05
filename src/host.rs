use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub enum Entry {
    Host(Host),
    Folder(Folder),
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub name: String,
    pub children: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Host {
    pub name: String,
    pub target: String,
    pub config: Vec<String>,
    pub description: String,
}

pub fn collect_hosts() -> io::Result<Vec<Entry>> {
    let home = std::env::var("HOME").expect("HOME is not set");
    let ssh_dir = PathBuf::from(home).join(".ssh");

    let mut hosts = Vec::new();

    for entry in fs::read_dir(&ssh_dir)? {
        let path = entry?.path();

        if path.is_file() && is_probably_ssh_config(&path)? {
            hosts.extend(find_hosts(&path)?);
        }
    }

    Ok(hosts)
}

fn is_connectable_host_pattern(pattern: &str) -> bool {
    !pattern.contains('*')
        && !pattern.contains('?')
        && !pattern.contains('!')
}

fn find_hosts(path: &Path) -> io::Result<Vec<Entry>> {
    let content = fs::read_to_string(path)?;
    let mut hosts = Vec::new();
    let mut current: Vec<Host> = Vec::new();

    for (_index, line) in content.lines().enumerate() {
        let line = line.replace('\t', &" ".repeat(4));
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };

        if keyword.eq_ignore_ascii_case("Host") {
            for host in current.drain(..) {
                hosts.push(Entry::Host(host));
            }

            let patterns = parts
                .filter(|pattern| is_connectable_host_pattern(pattern)) // TODO: We can merge this into the config of connectionables
                .map(String::from)
                .collect::<Vec<_>>();

            current = patterns
                .into_iter()
                .map(|pattern| Host {
                    name: pattern,
                    target: "localhost".into(),
                    config: vec![line.clone()],
                    description: String::new(),
                })
                .collect();
        } else {
            for host in &mut current {
                host.config.push(line.to_string());
            }
        }
    }

    for host in current.drain(..) {
        hosts.push(Entry::Host(host));
    }

    Ok(hosts)
}

fn is_probably_ssh_config(path: &Path) -> io::Result<bool> {
    let content = fs::read(path)?;

    if content.contains(&0) {
        return Ok(false);
    }

    let text = String::from_utf8_lossy(&content);
    let first_line = text.lines().next().unwrap_or("");

    if first_line.starts_with("-----BEGIN")
        || first_line.starts_with("ssh-ed25519 ")
        || first_line.starts_with("ssh-rsa ")
        || first_line.starts_with("ecdsa-sha2-")
    {
        return Ok(false);
    }

    Ok(true)
}