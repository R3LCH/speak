use anyhow::Result;
use clap::{Parser, Subcommand};
use speak::config::Config;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Run,
    Start,
    Stop,
    Status,
    Config,
    Doctor,
}
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let command = Cli::parse().command;
    match command {
        Command::Config => {
            println!("{}", toml::to_string_pretty(&Config::default())?);
            Ok(())
        }
        Command::Run => {
            speak::daemon::Daemon::run(Config::load(Config::default_path()).unwrap_or_default())
        }
        Command::Doctor => speak::control::doctor(&Config::default()),
        Command::Start => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "start", "speak.service"])
                .status()?;
            if !status.success() {
                anyhow::bail!("systemctl failed to start speak.service")
            }
            Ok(())
        }
        Command::Stop => speak::control::client_command("stop"),
        Command::Status => speak::control::client_command("status"),
    }
}
