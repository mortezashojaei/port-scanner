use crate::error::ScanError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub struct ServiceDetector;

#[derive(Debug)]
pub struct ServiceInfo {
    pub protocol: String,
    pub service_name: String,
    pub details: String,
}

impl ServiceDetector {
    pub async fn detect(stream: &mut TcpStream) -> Result<ServiceInfo, ScanError> {
        let port = stream.peer_addr()?.port();
        let addr = stream.peer_addr()?;

        // Try active detection first based on port type
        match port {
            // Ethereum RPC ports - check these first
            8545..=8549 => {
                if let Ok(mut new_stream) = TcpStream::connect(addr).await {
                    if let Ok(rpc_info) = Self::detect_json_rpc(&mut new_stream).await {
                        return Ok(ServiceInfo {
                            protocol: "JSON-RPC".to_string(),
                            service_name: rpc_info.service_type,
                            details: rpc_info.version,
                        });
                    }
                }
                // Fallback to default RPC service info
                return Ok(ServiceInfo {
                    protocol: "JSON-RPC".to_string(),
                    service_name: "ETH-RPC".to_string(),
                    details: "Ethereum JSON-RPC Service".to_string(),
                });
            }

            // Debug ports
            1234 | 4444 | 5555 | 6666 | 7777 => Ok(ServiceInfo {
                protocol: "TCP".to_string(),
                service_name: "Debug".to_string(),
                details: "Debug/Remote Debug Port".to_string(),
            }),

            // API Ports
            5000..=5050 | 7000..=7070 => Ok(ServiceInfo {
                protocol: "HTTP".to_string(),
                service_name: "API".to_string(),
                details: "REST/GraphQL API Service".to_string(),
            }),

            // Web/HTTP Ports
            80 | 443 | 3000..=4999 | 8000..=9000 => {
                if let Ok(mut new_stream) = TcpStream::connect(addr).await {
                    if let Ok(http_info) = Self::detect_http(&mut new_stream).await {
                        return Ok(ServiceInfo {
                            protocol: "HTTP".to_string(),
                            service_name: http_info.server_type,
                            details: http_info.headers,
                        });
                    }
                }
                Ok(ServiceInfo {
                    protocol: "HTTP".to_string(),
                    service_name: "HTTP".to_string(),
                    details: "Web Server".to_string(),
                })
            }

            // Unknown ports
            _ => Ok(ServiceInfo {
                protocol: "TCP".to_string(),
                service_name: "Unknown".to_string(),
                details: "Generic TCP Service".to_string(),
            }),
        }
    }

    async fn detect_http(stream: &mut TcpStream) -> Result<HttpInfo, ScanError> {
        stream.set_nodelay(true)?;

        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

        // Set write timeout
        if let Err(_) = timeout(
            Duration::from_millis(500),
            stream.write_all(request.as_bytes()),
        )
        .await
        {
            return Err(ScanError::ServiceDetection(
                "HTTP write timeout".to_string(),
            ));
        }

        let mut buffer = [0; 4096];
        match timeout(Duration::from_millis(500), stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => {
                let response = String::from_utf8_lossy(&buffer[..n]);
                if response.contains("HTTP/") {
                    let server_type = if response.contains("nginx") {
                        "Nginx"
                    } else if response.contains("Apache") {
                        "Apache"
                    } else if response.to_lowercase().contains("graphql") {
                        "GraphQL API"
                    } else if response.contains("/api") || response.contains("swagger") {
                        "REST API"
                    } else {
                        "HTTP Service"
                    };

                    // Extract status code and server name
                    let status_line = response.lines().next().unwrap_or("");
                    let server_header = response
                        .lines()
                        .find(|line| line.to_lowercase().starts_with("server:"))
                        .unwrap_or("")
                        .trim_start_matches("Server:")
                        .trim();

                    // Create a more concise details string
                    let details = if !server_header.is_empty() {
                        format!("{} ({})", status_line, server_header)
                    } else {
                        status_line.to_string()
                    };

                    return Ok(HttpInfo {
                        server_type: server_type.to_string(),
                        headers: details,
                    });
                }
            }
            _ => {}
        }

        Err(ScanError::ServiceDetection("Not HTTP".to_string()))
    }

    async fn detect_json_rpc(stream: &mut TcpStream) -> Result<RpcInfo, ScanError> {
        stream.set_nodelay(true)?;

        let request = r#"{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}"#;
        let http_request = format!(
            "POST / HTTP/1.1\r\n\
             Host: localhost\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n\
             {}",
            request.len(),
            request
        );

        // Set write timeout
        if let Err(_) = timeout(
            Duration::from_millis(500),
            stream.write_all(http_request.as_bytes()),
        )
        .await
        {
            return Err(ScanError::ServiceDetection("RPC write timeout".to_string()));
        }

        let mut buffer = vec![0; 4096];
        match timeout(Duration::from_millis(500), stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => {
                let response = String::from_utf8_lossy(&buffer[..n]);
                if response.contains("jsonrpc")
                    || response.contains("eth_")
                    || response.contains("web3_")
                {
                    return Ok(RpcInfo {
                        service_type: "Ethereum Node".to_string(),
                        version: "JSON-RPC 2.0".to_string(),
                    });
                }
            }
            _ => {}
        }

        Err(ScanError::ServiceDetection("Not JSON-RPC".to_string()))
    }
}

#[derive(Debug)]
struct HttpInfo {
    server_type: String,
    headers: String,
}

#[derive(Debug)]
struct RpcInfo {
    service_type: String,
    version: String,
}
