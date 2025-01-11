use crate::error::ScanError;
use crate::service::ServiceDetector;
use crate::service::ServiceInfo;
use colored::*;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::net::IpAddr;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

pub struct Scanner {
    target: IpAddr,
    start_port: u16,
    end_port: u16,
    timeout_ms: u64,
    concurrent_limit: usize,
}

impl Scanner {
    pub fn new(
        target: IpAddr,
        start_port: u16,
        end_port: u16,
        timeout_ms: u64,
        concurrent_limit: usize,
    ) -> Self {
        Scanner {
            target,
            start_port,
            end_port,
            timeout_ms,
            concurrent_limit,
        }
    }

    pub async fn scan(&self) -> Result<(), ScanError> {
        println!(
            "\n{} {} ({}-{})",
            "Scanning".bright_blue(),
            self.target.to_string().bright_yellow(),
            self.start_port,
            self.end_port
        );

        // Print header
        println!(
            "\n{:<8} {:<7} {:<15} {:<20} {:<}",
            "STATUS", "PORT", "PROTOCOL", "SERVICE", "DETAILS"
        );
        println!("{}", "-".repeat(80));

        // Create progress bar with improved style
        let pb = ProgressBar::new((self.end_port - self.start_port + 1) as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("\n{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ports scanned\n{msg}")
                .unwrap()
                .progress_chars("█▇▆▅▄▃▂▁  "),
        );
        pb.set_message("Starting scan...");

        let mut tasks = FuturesUnordered::new();
        let mut port = self.start_port;
        let mut open_ports = 0;

        while port <= self.end_port || !tasks.is_empty() {
            while tasks.len() < self.concurrent_limit && port <= self.end_port {
                tasks.push(self.scan_port(port));
                port += 1;
            }

            if let Some(result) = tasks.next().await {
                pb.inc(1);
                if let Ok(Some((port, service_info))) = result {
                    open_ports += 1;
                    pb.set_message(format!("Open ports found: {}", open_ports));

                    // Clear the progress bar temporarily
                    pb.suspend(|| {
                        println!(
                            "{:<8} {:<7} {:<15} {:<20} {:<}",
                            "OPEN".bright_green(),
                            port,
                            service_info.protocol.bright_blue(),
                            service_info.service_name.bright_blue(),
                            service_info.details.bright_white()
                        );
                    });
                }
            }
        }

        pb.finish_and_clear();
        println!(
            "\n{} Found {} open ports.",
            "Scan completed!".bright_green(),
            open_ports
        );

        Ok(())
    }

    async fn scan_port(&self, port: u16) -> Result<Option<(u16, ServiceInfo)>, ScanError> {
        let addr = format!("{}:{}", self.target, port);

        // Try to establish connection with timeout
        match timeout(
            Duration::from_millis(self.timeout_ms),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(mut stream)) => {
                // Set TCP_NODELAY to avoid buffering
                if let Ok(()) = stream.set_nodelay(true) {
                    if let Ok(service_info) = ServiceDetector::detect(&mut stream).await {
                        return Ok(Some((port, service_info)));
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
