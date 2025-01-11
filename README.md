# Port Scanner

A fast, concurrent port scanner written in Rust that intelligently detects common services and protocols with a focus on blockchain and web services.

## Features

- 🚀 **High Performance**:

  - Async/await based concurrent scanning
  - Configurable concurrency limits
  - Non-blocking I/O operations

- 🔍 **Smart Service Detection**:

  - Ethereum JSON-RPC nodes (ports 8545-8549)
  - HTTP/HTTPS web servers with server fingerprinting
  - REST/GraphQL API endpoints
  - Debug/Remote ports
  - Generic TCP services

- 🌐 **Network Support**:

  - IPv4 and IPv6 addresses
  - Hostname resolution via Google DNS (1.1.1.1)
  - Fallback DNS mechanisms
  - Custom port range scanning

- 📊 **User Experience**:

  - Real-time progress bar with ETA
  - Colored terminal output
  - Detailed service information including:
    - Protocol detection
    - Service identification
    - Version information (when available)
    - Response headers analysis

- ⚙️ **Configuration Options**:
  - Port range selection (`--start-port`, `--end-port`)
  - Connection timeout (`--timeout`)
  - Concurrent scan limit (`--concurrent-limit`)
  - Custom service detection rules

## Prerequisites

- Rust 1.56 or higher
- Cargo package manager

## Installation

### From Crates.io (Recommended)

```bash
cargo install port-scanner
```

### From Source

```bash
git clone https://github.com/morteza-shojaei/port-scanner.git
cd port-scanner
cargo build --release
```

## Usage

Basic usage:

```bash
port-scanner --target <host>
```

Advanced options:

```bash
port-scanner --target <host> --start-port 1 --end-port 1024 --timeout 1000 --concurrent-limit 100
```

### Examples

Scan a specific IP:

```bash
port-scanner --target 192.168.1.1
```

Scan a domain:

```bash
port-scanner --target ethereum.org
```

Scan specific port range:

```bash
port-scanner --target 192.168.1.1 --start-port 8545 --end-port 8549
```

## Security Considerations

- Always ensure you have permission to scan the target system
- Be aware that aggressive scanning may trigger IDS/IPS systems
- Consider rate limiting when scanning production systems
- Some networks may block or flag port scanning activities

## License

MIT License
