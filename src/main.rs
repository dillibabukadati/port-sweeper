//! CLI entrypoint for psweep.

use clap::{Parser, Subcommand};
use psweep::{kill_ports, list_ports, parse_port_spec};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "psweep")]
#[command(about = "Find and kill processes by port", long_about = None)]
struct Cli {
    /// Subcommand; omit to open the GUI (e.g. when launched from the app)
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List active ports and their processes
    List,
    /// Kill process(es) on the given port(s). Supports: 3000, 3000,8000, 9000-9010
    Kill {
        /// Port(s) to kill: single port, comma-separated, or range (e.g. 3000,8000,9000-9010)
        #[arg(required = true)]
        ports: String,
    },
    /// Open the Port Sweeper GUI
    Gui,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {:?}", e);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        None => {
            // No subcommand = GUI (e.g. double-click from DMG). On macOS, if not yet in /Applications, install then launch.
            #[cfg(target_os = "macos")]
            if !psweep::installer_macos::is_installed() {
                return psweep::installer_macos::install_then_launch();
            }
            run_gui()
        }
        Some(Commands::List) => run_list(),
        Some(Commands::Kill { ports }) => run_kill(&ports),
        Some(Commands::Gui) => run_gui(),
    }
}

fn run_list() -> anyhow::Result<()> {
    let entries = list_ports().map_err(anyhow::Error::msg)?;
    if entries.is_empty() {
        println!("No listening ports found.");
        return Ok(());
    }
    // Table header
    println!("{:>6}  {:<24}  {:>6}  {:<10}", "Port", "Process", "PID", "Status");
    println!("{}", "-".repeat(52));
    for e in &entries {
        println!("{:>6}  {:<24}  {:>6}  {:<10}", e.port, truncate(&e.process_name, 24), e.pid, e.status);
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn run_kill(ports_spec: &str) -> anyhow::Result<()> {
    let ports = parse_port_spec(ports_spec).map_err(anyhow::Error::msg)?;
    if ports.is_empty() {
        anyhow::bail!("No ports specified");
    }
    let results = kill_ports(&ports);
    let mut failed = 0;
    for r in &results {
        if r.success {
            println!("{}", r.message);
        } else {
            eprintln!("{}", r.message);
            failed += 1;
        }
    }
    if failed > 0 {
        anyhow::bail!("{} port(s) failed", failed);
    }
    Ok(())
}

fn run_gui() -> anyhow::Result<()> {
    psweep::gui::run()
}
