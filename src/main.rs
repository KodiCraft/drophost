#![feature(backtrace_frames)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod writer;
mod types;
mod parser;
mod utils;


use std::path;

use clap::Parser;
use pretty_env_logger;
use log::*;
use nix::unistd::Uid;
use proc_macros::compile_warning;

fn main() {
    pretty_env_logger::init();

    let opts: cli::Opts = cli::Opts::parse();

    debug!("Starting with options: {:?}", opts);
    debug!("Running as user: {}", Uid::current().as_raw());

    run(opts);
}

fn run(opts: cli::Opts) {
    let root_prefix: &str;
    if opts.dry_run {
        info!("Dry run, not replacing system files");
        root_prefix = "./output";
    } else {
        root_prefix = "/etc";
    }

    if !Uid::current().is_root() && !opts.dry_run {
        error!("Must run as root! Use --dry-run to test without root");
        std::process::exit(1);
    }

    let dir = root_prefix.to_owned() + "/hosts.d";

    let mut dir_reader = parser::DirReader::new(&path::Path::new(&dir));

    dir_reader.parse_all();

    let output_path = root_prefix.to_owned() + "/hosts";

    writer::write_hosts_to_file(&dir_reader.hosts, &output_path);
}