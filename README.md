# Sysmon - A Simple Terminal System Monitor

## Overview

Sysmon is a lightweight, efficient terminal-based system monitoring tool written in Rust. It provides real-time monitoring of system performance metrics in a clean, intuitive interface.

## Features

- **Real-time monitoring** - Continuously tracks system performance metrics
- **Resource tracking** - Monitors CPU, memory, and other system resources
- **Terminal-based interface** - Runs directly in your terminal without GUI dependencies
- **Rust-powered** - Built with Rust for performance, safety, and efficiency

## Project Structure

```
sysmon/
├── src/                    # Source code
├── .github/workflows/     # GitHub Actions CI/CD configuration
├── Cargo.toml            # Rust project configuration
├── Cargo.lock            # Dependency lock file
└── .gitignore            # Git ignore rules
```

## Getting Started

### Prerequisites

- Rust toolchain (cargo, rustc)
- Git

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/louis-nwosu/sysmon.git
   cd sysmon
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/sysmon
   ```

### Development

For development purposes:

```bash
cargo run
```

## Continuous Integration

This project includes GitHub Actions workflow configured for Rust projects, providing automated testing and building.

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests to improve Sysmon.

## License

This project is open source. See the repository for license details.

---

*Last updated: January 2026*
