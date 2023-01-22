use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Opts {
    /// Dry run, instead of replacing system files, write the new /etc/hosts to the current directory
    #[clap(short, long, default_value = "false")]
    pub dry_run: bool,

    /// Watch the hosts.d directory for changes and update the hosts file
    #[clap(short, long, default_value = "false")]
    pub watch: bool,
}