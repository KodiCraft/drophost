#![feature(backtrace_frames)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature = "interface")]
compile_warning!(The "interface" feature is not currently implemented.);

mod cli;
mod writer;
mod types;
mod parser;
mod utils;
mod tests;

use std::path::{self, Path};
use notify::{RecommendedWatcher, RecursiveMode, recommended_watcher, Watcher};

use clap::Parser;
use once_cell::sync::Lazy;
use env_logger::{self, Builder};
use log::*;
use nix::{unistd::Uid, sys};
use compile_warning::compile_warning;

static OPTS: Lazy<cli::Opts> = Lazy::new(|| cli::Opts::parse());

#[tokio::main]
async fn main() {
    Builder::new()
        .filter_level(OPTS.log_level)
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .init();

    debug!("Starting with options: {:?}", OPTS);
    debug!("Running as user: {}", Uid::current().as_raw());

    if OPTS.backup {
        backup();
    }

    run();

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

fn run() {
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

    let mut dir_reader = parser::DirReader::new(&path::Path::new(&dir));

    dir_reader.parse_all();

    let output_path = root_prefix.to_owned() + "/hosts";

    writer::write_hosts_to_file(&dir_reader.hosts, &output_path);
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
            let _ = run();
        },
        Err(e) => error!("An error has occured while watching: {:?}", e),
    }
}