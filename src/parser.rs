use crate::utils::*;

use log::*;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, BufReader, BufRead};
use std::net::{IpAddr};
use std::path::{Path, PathBuf};
use std::boxed::Box;
use std::str::{FromStr};
use std::collections::VecDeque;

#[cfg(any(feature = "interface", feature = "range"))]
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use crate::types::{Hosts, Host};

pub struct DirReader {
    pub dir_path: Box<PathBuf>,
    pub hosts: Box<Hosts>,

    pub vars: HashMap<String, String>,

    pub files: Vec<PathBuf>,
    pub file_index: usize,
}

impl DirReader {
    pub fn new(dir_path: &Path) -> Self {
        // Check if the directory exists
        if !dir_path.exists() {
            info!("Config directory does not exist, creating...");
            let created_dir = fs::create_dir_all(dir_path);

            let _ = unwrap_result_or_err(created_dir,
                                     "Failed to create directory!",
                                     true);
        }

        // Check if the directory is a directory
        if !dir_path.is_dir() {
            error!("Config directory '{}' is not a directory!", dir_path.to_str().unwrap());
            std::process::exit(1);
        }

        // Check if the directory is readable
        if !dir_path.is_dir() {
            error!("Config directory '{}' is not readable!", dir_path.to_str().unwrap());
            std::process::exit(1);
        }

        // Read the directory
        let dir_read_res = fs::read_dir(dir_path);
        let dir_read = unwrap_result_or_err(dir_read_res,
                                        "Failed to read directory!",
                                        true).expect("This should never happen! (DirReader::new)");

        // Sort the files by name alphabetically
        let mut dir_read: Vec<_> = dir_read.collect();
        dir_read.sort_by_key(|entry| entry.as_ref().unwrap().path());

        
        let files = dir_read.iter().map(|res| res.as_ref().unwrap().path())
                            .collect::<Vec<_>>();

        // Add all of the user's environment variables to the vars map with the name 'env_VARNAME'
        let mut vars = HashMap::new();
        for (key, value) in std::env::vars() {
            let key = format!("env_{}", key);
            let value = value.to_owned();
            vars.insert(key, value);
        }

        DirReader {
            dir_path: Box::new(dir_path.to_owned()),
            hosts: Box::new(Hosts::new()),
            files,
            file_index: 0,
            vars,
        }
    }

    pub fn next(&mut self) -> Option<FileReader> {
        if self.file_index >= self.files.len() {
            info!("No more files to parse!");
            return None;
        }

        let file = &self.files[self.file_index];
        info!("Parsing file '{}'...", file.to_str().unwrap());
        self.file_index += 1;

        Some(FileReader::new(file, self.vars.clone()))
    }

    pub fn get_hosts(&self) -> &Hosts {
        &self.hosts
    }

    pub fn parse_all (&mut self) {
        while let Some(mut file) = self.next() {
            file.parse_all();
            self.hosts.extend(&file.hosts);
            self.vars.extend(file.vars);
        }
    }
}

pub enum ParseState {
    Normal,
    Conditional,
    Waiting
}

pub struct FileReader {
    pub path: Box<PathBuf>,
    pub hosts: Box<Hosts>,

    pub parse_state: ParseState,
    pub lines: Vec<String>,

    // Redundant, but useful for error messages
    pub line_index: usize,

    pub vars: HashMap<String, String>,
    pub cond_stack: VecDeque<bool>,
}

impl FileReader {
    pub fn new(path: &Path, vars: HashMap<String, String>) -> Self {
        let file = fs::File::open(path);
        let file = unwrap_result_or_err(file,
                                    "Failed to open file!",
                                    true).expect("This should never happen! (FileReader::new)");

        let mut reader = std::io::BufReader::new(file);
        let mut contents: String = String::new();
        let read_res = reader.read_to_string(&mut contents);
        let _ = unwrap_result_or_err(read_res,
                                 "Failed to read file!",
                                 true).expect("This should never happen! (FileReader::new)");

        let lines = contents.lines();
        let lines = lines.map(|x| x.to_owned()).collect();

        let mut stack = VecDeque::new();
        stack.push_back(true);

        FileReader {
            path: Box::new(path.to_owned()),
            hosts: Box::new(Hosts::new()),
            parse_state: ParseState::Waiting,
            lines,
            line_index: 0,
            vars,
            cond_stack: stack,
        }
    }

    fn parse(&mut self) -> bool {
        let line = self.lines[self.line_index - 1].clone();
        let line = line.trim().to_owned();
        if line.starts_with("#=>") {
            let warning = line.trim_start_matches("#=>");
            warn!("Warning raised while parsing file: {}", warning);
            return true;
        }
        if line.starts_with("#") {
            return true;
        }
        if line.starts_with("if ") {
            self.parse_state = ParseState::Conditional;
            // Parse the conditional
            let cond = line.trim_start_matches("if ");
            let cond = self.parse_conditional(cond);
            self.cond_stack.push_back(cond);
            return true;
        }
        if line.starts_with("try ") {
            let attempt = line.trim_start_matches("try ");
            let attempt = self.parse_try(attempt);
            self.cond_stack.push_back(attempt);
            self.parse_state = ParseState::Conditional;
            return true;
        }

        if line.starts_with("set ") {
            let set = line.trim_start_matches("set ");
            let mut split = set.splitn(2, '=');
            let key = split.next().unwrap().trim();
            let value = split.next().unwrap().trim();
            self.vars.insert(key.to_owned(), value.to_owned());
            return true;
        }

        if line.starts_with("unset ") {
            let unset = line.trim_start_matches("unset ");
            self.vars.remove(unset);
            return true;
        }

        if line.is_empty() {
            return true;
        }

        // Parse the line as a host
        let mut split = line.split_whitespace();
        let ip = split.next();
        let ip = unwrap_or_err(ip,
                                                            format!("Syntax error in file {} at line {}: Missing IP", self.path.to_string_lossy(), self.line_index).as_str(), false);
        if ip.is_err() {
            return false;
        }

        let hostname = split.next();
        let hostname = unwrap_or_err(hostname,
                                                            format!("Syntax error in file {} at line {}: Missing hostname", self.path.to_string_lossy(), self.line_index).as_str(), false);
        if hostname.is_err() {
            return false;
        }

        let hostname = self.parse_var_or_literal(hostname.unwrap());
        let ip = self.parse_var_or_literal(ip.unwrap());

        let host = Host::new(hostname.to_string(), ip.to_string());
        self.hosts.add(host);
        return true;
    }

    pub fn next(&mut self) -> bool {
        self.line_index += 1;
        return self.parse_current_line();
    }

    pub fn parse_current_line(&mut self) -> bool {
        if self.line_index > self.lines.len() {
            return false;
        }

        let line = &self.lines[self.line_index - 1].clone();
        let line = line.trim().to_owned();
        match self.parse_state {
            ParseState::Normal => {
                return self.parse();
            },

            ParseState::Conditional => {
                if line.starts_with("else") {
                    let current_cond = self.cond_stack.pop_back().unwrap();
                    self.cond_stack.push_back(!current_cond);
                    return true;
                }

                if line.starts_with("end") {
                    self.cond_stack.pop_back();
                    self.parse_state = ParseState::Normal;
                    return true;
                }

                if self.cond_stack.back().unwrap() == &true {
                    return self.parse();
                }

                // We're in a conditional but the condition is false, so we don't parse the line
                return true;
            },
            
            ParseState::Waiting => {
                self.parse_state = ParseState::Normal;
                return self.parse();
            }
        }
    }

    fn parse_var_or_literal(&mut self, input: &str) -> String {
        if input.starts_with("$") {
            let var_name = input.trim_start_matches("$");
            let var = self.vars.get(var_name);
            if var.is_none() {
                warn!("Error while reading file '{}' at line {}: Variable '{}' not found!", self.path.to_str().unwrap(), self.line_index, var_name);
                return String::new();
            }
            var.unwrap().to_owned()
        } else {
            input.to_owned()
        }
    }

    fn parse_conditional(&mut self, cond: &str) -> bool {
        let mut cond = cond.split_whitespace();
        let i1 = cond.next();
        let op = cond.next();
        let i2 = cond.next();

        if i1.is_none() || op.is_none() || i2.is_none() {
            warn!("Error while reading file '{}' at line {}: Invalid conditional!", self.path.to_str().unwrap(), self.line_index);
            return false;
        }

        let i1 = self.parse_var_or_literal(i1.unwrap());
        let op = op.unwrap();
        let i2 = self.parse_var_or_literal(i2.unwrap());

        match op {
            "==" => i1 == i2,
            "!=" => i1 != i2,
            _ => {
                warn!("Error while reading file '{}' at line {}: Invalid operator '{}'!", self.path.to_str().unwrap(), self.line_index, op);
                false
            }
        }
    }

    fn parse_try(&mut self, attempt: &str) -> bool {
        // We can try one of the following things:
        // 'ping <ip>' - Ping the IP address, return true if it responds
        // 'file <path>' - Check if the file exists, return true if it does
        //// 'self <ip-range>' - Check if the current IP is in the range, return true if it is
        //// 'int <interface>' - Check if the interface exists and is up, return true if it is
        // 'var <var-name>' - Check if the variable exists, return true if it does
        // 'has <host>' - Check if the host has been set earlier in the file, return true if it has

        let mut attempt = attempt.split_whitespace();
        let attempt_type = attempt.next();
        let attempt_value = attempt.next();

        if attempt_type.is_none() || attempt_value.is_none() {
            warn!("Error while reading file '{}' at line {}: Invalid try statement!", self.path.to_str().unwrap(), self.line_index);
            return false;
        }

        let attempt_type = attempt_type.unwrap();
        let attempt_value = attempt_value.unwrap();

        match attempt_type {
            #[cfg(feature = "ping")]
            "ping" => {
                use oping::Ping;

                warn!("Warning! The 'ping' feature is considered unstable. Please report any bugs you find!");

                let ip = IpAddr::from_str(attempt_value);
                if ip.is_err() {
                    warn!("Error while reading file '{}' at line {}: Invalid IP address '{}'!", self.path.to_str().unwrap(), self.line_index, attempt_value);
                    return false;
                }
                let ip = ip.unwrap();

                let mut ping = Ping::new();

                // Setup the ping
                let res = ping.set_timeout(1.0);
                let _ = unwrap_result_or_err(res,
                                             "An error has occured while handling a ping! This should not happen", true);
                let res = ping.set_ttl(1);
                let _ = unwrap_result_or_err(res,
                                             "An error has occured while handling a ping! This should not happen", true);
                let res = ping.add_host(&ip.to_string());
                let _ = unwrap_result_or_err(res,
                                             "An error has occured while handling a ping! This should not happen", true);

                // Send the ping
                let res = ping.send();
                if res.is_err() {
                    warn!("Error while reading file '{}' at line {}: Failed to send ping!", self.path.to_str().unwrap(), self.line_index);
                    return false;
                }

                // Get the response
                let mut res = res.unwrap();

                if res.next().unwrap().dropped <= 1 {
                    true
                } else {
                    false
                }
            }

            "file" => {
                let path = Path::new(attempt_value);
                path.exists()
            }

            "var" => {
                self.vars.contains_key(attempt_value)
            }

            "has" => {
                self.hosts.hosts.iter().any(|host| host.hostname == attempt_value)
            }
            
            bad => {
                warn!("Error while reading file '{}' at line {}: Invalid try type '{}'!", self.path.to_str().unwrap(), self.line_index, bad);
                false
            }
        }
    }

    pub fn parse_all(&mut self) {
        let file = File::open(self.path.as_path());
        if file.is_err() {
            warn!("Error while reading file '{}': File not found!", self.path.to_str().unwrap());
            return;
        }

        while self.next() {}
    }
}