use clap::Parser;
use colored::*;
use std::net::IpAddr;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

mod error;
mod scanner;
mod service;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target IP address or hostname
    #[arg(short, long)]
    target: String,

    /// Start port number
    #[arg(short, long, default_value = "1")]
    start_port: u16,

    /// End port number
    #[arg(short, long, default_value = "1024")]
    end_port: u16,

    /// Timeout in milliseconds
    #[arg(short = 'T', long, default_value = "1000")]
    timeout: u64,

    /// Number of concurrent scans
    #[arg(short, long, default_value = "100")]
    concurrent_limit: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!(
        "\n{} {}...",
        "Resolving".bright_blue(),
        args.target.bright_yellow()
    );

    // Try to parse as IP first
    let target = if let Ok(ip) = args.target.parse::<IpAddr>() {
        ip
    } else {
        // Use Google's DNS servers for more reliable resolution
        let resolver = trust_dns_resolver::TokioAsyncResolver::tokio(
            ResolverConfig::google(),
            ResolverOpts::default(),
        );

        match resolver.lookup_ip(args.target.as_str()).await {
            Ok(response) => {
                if let Some(ip) = response.iter().next() {
                    println!(
                        "{} {} -> {}",
                        "Resolved".bright_green(),
                        args.target.bright_yellow(),
                        ip.to_string().bright_green()
                    );
                    ip
                } else {
                    eprintln!(
                        "{} Could not resolve hostname to any IP address",
                        "Error:".bright_red()
                    );
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to resolve hostname: {}",
                    "Error:".bright_red(),
                    e
                );
                eprintln!("Try using IP address directly or check your internet connection");
                std::process::exit(1);
            }
        }
    };

    let scanner = scanner::Scanner::new(
        target,
        args.start_port,
        args.end_port,
        args.timeout,
        args.concurrent_limit,
    );

    scanner.scan().await?;

    Ok(())
}
