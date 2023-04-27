#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod writer;
mod types;
mod parser;
#[macro_use]
mod utils;
mod tests;

use std::path::{self, Path};
use notify::{RecommendedWatcher, RecursiveMode, recommended_watcher, Watcher};

use clap::Parser;
use once_cell::sync::Lazy;
use env_logger::{self, Builder};
use log::*;
use nix::{unistd::Uid, unistd::fork, unistd::ForkResult, sys};
use std::io::Write;


#[cfg(dev)]
use compile_warning::compile_warning;
#[cfg(not(dev))]
#[allow(unused_macros)]
macro_rules! compile_warning {($($tt:tt)*) => {}}

#[cfg(feature = "interface")]
compile_warning!(The "interface" feature is not currently implemented.);
#[cfg(feature = "range")]
compile_warning!(The "range" feature is not currently implemented.);

static OPTS: Lazy<cli::Opts> = Lazy::new(|| cli::Opts::parse());

#[tokio::main]
async fn main() {
    Builder::new()
        .filter_level(OPTS.log_level)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();

    debug!("Starting with options: {:?}", OPTS);
    debug!("Running as user: {}", Uid::current().as_raw());
    info!("Starting drophost!");

    if OPTS.backup {
        backup();
    }

    run(!OPTS.check);

    if OPTS.daemon && OPTS.watch {
        daemonize(&OPTS.pid_file);
    }

    if OPTS.daemon && !OPTS.watch {
        error!("Cannot run as daemon without --watch");
        std::process::exit(1);
    }

    if OPTS.watch {
        watch().await;
    }
}

fn backup() {
    let root_prefix: &str;
    if OPTS.dry_run {
        info!("Dry run, not replacing system files");
        root_prefix = "./output";
    } else {
        root_prefix = "/etc";
    }

    if !Uid::current().is_root() && !OPTS.dry_run {
        error!("Must run as root! Use --dry-run to test without root");
        std::process::exit(1);
    }

    let file = root_prefix.to_owned() + "/hosts";
    let backup_file = root_prefix.to_owned() + "/hosts.d/10-old-config.conf";

    // Create target directory if it doesn't exist
    let dir = root_prefix.to_owned() + "/hosts.d";
    let res = std::fs::create_dir_all(&dir);
    let _ = utils::unwrap_result_or_err(res, "Could not create hosts.d directory!", true);

    let res = std::fs::copy(&file, &backup_file);
    let _ = utils::unwrap_result_or_err(res, "Could not backup hosts file!", true);
}

fn run(write: bool) {
    let root_prefix: &str;
    if OPTS.dry_run {
        info!("Dry run, not replacing system files");
        root_prefix = "./output";
    } else {
        root_prefix = "/etc";
    }

    if !Uid::current().is_root() && !OPTS.dry_run && write {
        error!("Must run as root! Use --dry-run to test without root");
        std::process::exit(1);
    }

    let dir = root_prefix.to_owned() + "/hosts.d";

    let mut dir_reader = parser::DirReader::new(&path::Path::new(&dir));

    dir_reader.parse_all();

    if write {
        let output_path = root_prefix.to_owned() + "/hosts";

        writer::write_hosts_to_file(&dir_reader.hosts, &output_path);
        info!("Updated hosts file!")
    } else {
        info!("Hosts file would be written to: {}", root_prefix.to_owned() + "/hosts");
    }
}

fn daemonize(pidfile: &str) {
    let pid = unsafe { fork() };
    match pid {
        Ok(ForkResult::Parent { child, .. }) => {
            info!("Forked to background, child PID: {}", child);
            let mut file = std::fs::File::create(pidfile).unwrap();
            file.write_all(child.to_string().as_bytes()).unwrap();
            std::process::exit(0);
        },
        Ok(ForkResult::Child) => {
            info!("Running as daemon");
            let _ = run(!OPTS.check);
            if OPTS.watch {
                watch();
            }
        },
        Err(e) => {
            error!("Could not fork to background: {}", e);
            std::process::exit(1);
        }
    }
}

async fn watch() {
    let root_prefix: &str;
    if OPTS.dry_run {
        info!("Dry run, not replacing system files");
        root_prefix = "./output";
    } else {
        root_prefix = "/etc";
    }

    if !Uid::current().is_root() && !OPTS.dry_run {
        error!("Must run as root! Use --dry-run to test without root");
        std::process::exit(1);
    }

    let dir = root_prefix.to_owned() + "/hosts.d";

    let path = Path::new(&dir);

    info!("Watching directory: {}", path.display());
    let mut watcher = recommended_watcher(handler).unwrap();

    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    loop {
        std::thread::park();
    }
}

fn handler(res: notify::Result<notify::Event>) {
    match res {
        Ok(event) => {
            debug!("Event: {:?}", event);
            info!("Change detected, re-running drophost's parser");
            let _ = run(!OPTS.check);
        },
        Err(e) => error!("An error has occured while watching: {:?}", e),
    }
}