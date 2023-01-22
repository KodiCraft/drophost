use std::io::Write;

use crate::{types::Hosts, utils::unwrap_result_or_err};

pub fn write_hosts_to_file(hosts: &Hosts, path: &str) {
    let mut file = std::fs::File::create(path).unwrap();
    let res = file.write_all("# Generated automatically by drophost\n".as_bytes());
    let _ = unwrap_result_or_err(res,
                                "Could not write to output file!", true);
    for host in &hosts.hosts {
        file.write_all(host.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
}