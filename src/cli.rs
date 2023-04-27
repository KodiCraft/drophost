use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Opts {
    /// Dry run, instead of replacing system files, write the new /etc/hosts to the current directory
    #[clap(short, long, default_value = "false")]
    pub dry_run: bool,

    /// Check Config, parse all files in the hosts.d directory and exit
    #[clap(short, long, default_value = "false")]
    pub check: bool,

    /// Watch the hosts.d directory for changes and update the hosts file
    #[clap(short, long, default_value = "false")]
    pub watch: bool,

    /// Fork to background and run as a daemon. Requires --watch
    #[clap(long, default_value = "false")]
    pub daemon: bool,

    /// PID file location (only used when running as a daemon)
    #[clap(short, long, default_value = "/run/drophost.pid")]
    pub pid_file: String,

    /// Save current hosts file to a backup file
    #[clap(short, long, default_value = "false")]
    pub backup: bool,

    /// Log level: trace, debug, info, warn, error, off
    #[clap(short, long, default_value = "info")]
    pub log_level: log::LevelFilter,

    /// Log file location, must be writable by the user running drophost
    #[clap(long, default_value = "/var/log/drophost.log")]
    pub log_file: String,
}