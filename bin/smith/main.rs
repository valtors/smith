use clap::{Parser, Subcommand};

use smith_compose::run_compose_server;
use smith_config::{load, save};
use smith_install::{install as do_install, uninstall, update};
use smith_profile as profile_mod;
use smith_registry;
use smith_secure;

#[derive(Parser)]
#[command(name = "smith", version, about = "npm for MCP. install, compose, secure, and manage MCP servers.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        source: String,
        #[arg(long)]
        profile: Option<String>,
    },
    Remove {
        name: String,
    },
    List,
    Update {
        name: Option<String>,
    },
    Compose,
    Secure {
        name: Option<String>,
    },
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    Search {
        query: String,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    List,
    Current,
    Switch { name: String },
    Assign { server: String, profile: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { source, profile } => {
            let mut config = load();
            match do_install(&mut config, &source, profile.as_deref()) {
                Ok(result) => {
                    save(&config).ok();
                    println!("{}: {}", result.name, result.message);
                    println!("  command: {} {}", result.command, result.args.join(" "));
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Commands::Remove { name } => {
            let mut config = load();
            if uninstall(&mut config, &name).unwrap_or(false) {
                save(&config).ok();
                println!("removed: {}", name);
            } else {
                eprintln!("not installed: {}", name);
            }
        }
        Commands::List => {
            let config = load();
            if config.servers.is_empty() {
                println!("no servers installed. try: smith install @modelcontextprotocol/filesystem");
                return;
            }
            let active = config.active_servers();
            for server in &config.servers {
                let marker = if active.iter().any(|a| a.name == server.name) {
                    "[active]"
                } else {
                    "[inactive]"
                };
                println!("{} {} ({})", marker, server.name, server.profile);
                println!("  {} {} v{}", server.command, server.args.join(" "), server.version);
            }
        }
        Commands::Update { name } => {
            let mut config = load();
            match update(&mut config, name.as_deref()) {
                Ok(updated) => {
                    save(&config).ok();
                    if updated.is_empty() {
                        println!("nothing to update");
                    } else {
                        for n in updated {
                            println!("updated: {}", n);
                        }
                    }
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Commands::Compose => {
            let config = load();
            if config.active_servers().is_empty() {
                eprintln!("no active servers. install one: smith install @modelcontextprotocol/filesystem");
                return;
            }
            println!("smith compose: routing {} servers on stdio", config.active_servers().len());
            run_compose_server(&config);
        }
        Commands::Secure { name } => {
            let config = load();
            match name {
                Some(n) => {
                    match smith_secure::audit(&config, &n) {
                        Ok(report) => {
                            println!("{}: {:?}", report.server, report.risk_level);
                            if report.findings.is_empty() {
                                println!("  no issues found");
                            }
                            for f in &report.findings {
                                println!("  [{}] {}: {}", f.severity, f.category, f.message);
                            }
                        }
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
                None => {
                    let reports = smith_secure::audit_all(&config);
                    if reports.is_empty() {
                        println!("no servers to audit");
                    }
                    for report in reports {
                        println!("{}: {:?}", report.server, report.risk_level);
                        for f in &report.findings {
                            println!("  [{}] {}: {}", f.severity, f.category, f.message);
                        }
                    }
                }
            }
        }
        Commands::Profile { action } => {
            let mut config = load();
            match action {
                ProfileAction::List => {
                    let profiles = profile_mod::list(&config);
                    for p in profiles {
                        let marker = if p == config.active_profile { "*" } else { " " };
                        println!("{} {}", marker, p);
                    }
                }
                ProfileAction::Current => {
                    println!("{}", profile_mod::current(&config));
                }
                ProfileAction::Switch { name } => {
                    match profile_mod::switch(&mut config, &name) {
                        Ok(msg) => {
                            save(&config).ok();
                            println!("{}", msg);
                        }
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
                ProfileAction::Assign { server, profile } => {
                    match profile_mod::assign(&mut config, &server, &profile) {
                        Ok(()) => {
                            save(&config).ok();
                            println!("assigned {} to profile: {}", server, profile);
                        }
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
            }
        }
        Commands::Search { query } => {
            match smith_registry::fetch_registry() {
                Ok(entries) => {
                    let results = smith_registry::search(&entries, &query);
                    if results.is_empty() {
                        println!("no servers found for: {}", query);
                    }
                    for entry in results {
                        println!("{}", smith_registry::format_entry(entry));
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}
