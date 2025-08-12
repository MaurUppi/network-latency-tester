# Network Latency Tester - Usage Guide

## Overview

The Network Latency Tester is a high-performance command-line tool designed to measure network connectivity and latency to various targets using different DNS configurations. It supports custom DNS servers, DNS-over-HTTPS (DoH) providers, and provides detailed statistics and performance analysis.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Advanced Scenarios](#advanced-scenarios)
- [DNS Configuration](#dns-configuration)
- [Output Interpretation](#output-interpretation)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Installation

### From Source
```bash
git clone <repository-url>
cd network-latency-tester
cargo build --release
./target/release/network-latency-tester --help
```

### System Installation
```bash
cargo install --path .
network-latency-tester --help
```

## Quick Start

### Test Default Configuration
```bash
# Test with default settings (system DNS, 5 iterations)
network-latency-tester --url https://google.com
```

### Test Original Script Compatibility
```bash
# Run the original ctok.ai test for compatibility
network-latency-tester --test-original
```

### Test with Custom DNS
```bash
# Test using Google DNS servers
network-latency-tester --url https://github.com --dns-servers 8.8.8.8,8.8.4.4
```

## Basic Usage

### Single URL Testing
```bash
# Basic test with single URL
network-latency-tester --url https://example.com

# With custom iteration count and timeout
network-latency-tester --url https://example.com --count 10 --timeout 30

# With verbose output
network-latency-tester --url https://example.com --verbose
```

### Multiple URL Testing
```bash
# Test multiple URLs (repeat --url flag)
network-latency-tester \
  --url https://google.com \
  --url https://github.com \
  --url https://cloudflare.com \
  --count 5
```

### Environment Variable Configuration
```bash
# Using environment variables
export TARGET_URLS="https://google.com,https://github.com"
export DNS_SERVERS="8.8.8.8,1.1.1.1"
export TEST_COUNT=10
network-latency-tester
```

### Using .env File
Create a `.env` file:
```bash
# .env file
TARGET_URLS=https://google.com,https://github.com,https://cloudflare.com
DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
TEST_COUNT=15
TIMEOUT_SECONDS=25
ENABLE_COLOR=true
```

Then run:
```bash
network-latency-tester --verbose
```

## Advanced Scenarios

### Performance Benchmarking
```bash
# High-frequency testing for performance analysis
network-latency-tester \
  --url https://api.service.com \
  --count 50 \
  --timeout 10 \
  --dns-servers 8.8.8.8,1.1.1.1,9.9.9.9 \
  --verbose
```

### DNS Provider Comparison
```bash
# Compare different DNS providers
network-latency-tester \
  --url https://example.com \
  --dns-servers 8.8.8.8,1.1.1.1,9.9.9.9,208.67.222.222 \
  --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query \
  --count 20 \
  --verbose
```

### Network Diagnostics
```bash
# Comprehensive network diagnostics with debug output
network-latency-tester \
  --url https://problematic-site.com \
  --debug \
  --timeout 60 \
  --dns-servers 8.8.8.8,1.1.1.1 \
  --verbose
```

### Load Testing Scenario
```bash
# Simulate load testing with multiple targets
network-latency-tester \
  --url https://service1.com/api/health \
  --url https://service2.com/api/health \
  --url https://service3.com/api/health \
  --count 100 \
  --timeout 5 \
  --no-color > load_test_results.txt
```

### CI/CD Integration
```bash
# For automated testing in CI/CD pipelines
network-latency-tester \
  --url https://production-api.com/health \
  --count 5 \
  --timeout 10 \
  --no-color \
  --dns-servers 8.8.8.8 \
  --verbose || exit 1
```

### Geographic Performance Testing
```bash
# Test performance from different geographic perspectives
# using different DNS providers that might route differently

# US-based DNS
network-latency-tester \
  --url https://cdn.example.com \
  --dns-servers 8.8.8.8,1.1.1.1 \
  --count 10

# European DNS
network-latency-tester \
  --url https://cdn.example.com \
  --dns-servers 9.9.9.9,149.112.112.112 \
  --count 10
```

## DNS Configuration

### System DNS
```bash
# Use system default DNS (automatic)
network-latency-tester --url https://example.com
```

### Custom DNS Servers
```bash
# Google DNS
network-latency-tester --url https://example.com --dns-servers 8.8.8.8,8.8.4.4

# Cloudflare DNS
network-latency-tester --url https://example.com --dns-servers 1.1.1.1,1.0.0.1

# OpenDNS
network-latency-tester --url https://example.com --dns-servers 208.67.222.222,208.67.220.220

# Quad9 DNS
network-latency-tester --url https://example.com --dns-servers 9.9.9.9,149.112.112.112

# Mixed IPv4 and IPv6
network-latency-tester --url https://example.com --dns-servers 8.8.8.8,2001:4860:4860::8888
```

### DNS-over-HTTPS (DoH)
```bash
# Google DoH
network-latency-tester --url https://example.com --doh-providers https://dns.google/dns-query

# Cloudflare DoH
network-latency-tester --url https://example.com --doh-providers https://cloudflare-dns.com/dns-query

# Quad9 DoH
network-latency-tester --url https://example.com --doh-providers https://dns.quad9.net/dns-query

# Multiple DoH providers
network-latency-tester \
  --url https://example.com \
  --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
```

### Combined DNS Testing
```bash
# Test system, custom, and DoH all together
network-latency-tester \
  --url https://example.com \
  --dns-servers 8.8.8.8,1.1.1.1 \
  --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query \
  --count 10 \
  --verbose
```

## Output Interpretation

### Understanding the Results

#### Individual Test Results
```
Testing https://example.com with System DNS...
  ✓ Success: 200 OK (Total: 234ms, DNS: 12ms, Connect: 45ms, TLS: 67ms, Response: 110ms)
```

- **✓ Success/✗ Failed**: Request outcome
- **200 OK**: HTTP status code
- **Total**: Complete request duration
- **DNS**: Domain name resolution time
- **Connect**: TCP connection establishment
- **TLS**: TLS/SSL handshake time (HTTPS only)
- **Response**: Server response time

#### Statistics Summary
```
Statistics for https://example.com:
  System DNS: Min: 201ms, Max: 267ms, Avg: 234ms, StdDev: 18ms (Success: 100%)
  Custom DNS (8.8.8.8): Min: 189ms, Max: 245ms, Avg: 217ms, StdDev: 16ms (Success: 100%)
```

- **Min/Max/Avg**: Response time statistics
- **StdDev**: Standard deviation (consistency indicator)
- **Success**: Success rate percentage

#### Performance Classification
- 🟢 **Excellent** (< 100ms): Very fast, local or high-performance servers
- 🟡 **Good** (100-300ms): Typical internet response times
- 🟠 **Fair** (300-1000ms): Acceptable for most applications
- 🔴 **Poor** (> 1000ms): Slow, may indicate network issues

### Verbose Output Details
```bash
# Enable verbose mode for detailed information
network-latency-tester --url https://example.com --verbose
```

Verbose output includes:
- Individual request timings
- DNS resolution details
- Connection establishment logs
- Error details and retry information
- Platform-specific optimizations applied

### Debug Output
```bash
# Enable debug mode for diagnostic information
network-latency-tester --url https://example.com --debug
```

Debug output includes:
- Detailed error messages
- Configuration validation results
- DNS server accessibility checks
- Network capability detection
- Platform-specific adjustments

## Troubleshooting

### Common Issues and Solutions

#### DNS Resolution Failures
```bash
# Problem: DNS resolution fails
# Solution: Try different DNS servers
network-latency-tester --url https://example.com --dns-servers 8.8.8.8,1.1.1.1 --debug

# Try DoH if regular DNS fails
network-latency-tester --url https://example.com --doh-providers https://dns.google/dns-query
```

#### Connection Timeouts
```bash
# Problem: Frequent timeouts
# Solution: Increase timeout and reduce test count
network-latency-tester --url https://slow-server.com --timeout 60 --count 3

# Check with debug information
network-latency-tester --url https://slow-server.com --debug --timeout 30
```

#### High Latency Results
```bash
# Problem: Unexpectedly high latency
# Solution: Test with different DNS providers
network-latency-tester \
  --url https://example.com \
  --dns-servers 8.8.8.8,1.1.1.1,9.9.9.9 \
  --verbose

# Compare with direct IP access (if known)
network-latency-tester --url https://192.0.2.1 --verbose
```

#### Certificate Issues
```bash
# Problem: TLS/SSL certificate errors
# Solution: Check with debug mode
network-latency-tester --url https://problematic-site.com --debug

# Note: Tool always validates certificates for security
```

#### Platform-Specific Issues

##### Windows
```bash
# Windows Firewall may affect results
# Run as Administrator if needed
network-latency-tester --url https://example.com --debug

# Check Windows DNS configuration
network-latency-tester --help dns
```

##### macOS/Linux
```bash
# Check system DNS configuration
network-latency-tester --url https://example.com --debug

# For Linux, check systemd-resolved status
systemctl status systemd-resolved
```

### Getting Help
```bash
# General help
network-latency-tester --help

# Topic-specific help
network-latency-tester --help config
network-latency-tester --help dns
network-latency-tester --help examples
network-latency-tester --help timeout
network-latency-tester --help output
```

## Best Practices

### For Reliable Testing
1. **Use consistent test counts**: Stick to 5-10 iterations for regular testing
2. **Set appropriate timeouts**: 30 seconds for internet, 5 seconds for local
3. **Test multiple DNS providers**: Compare system vs custom vs DoH
4. **Use verbose mode for diagnostics**: Helps identify performance bottlenecks

### For Performance Testing
1. **Higher iteration counts**: Use 20-50 iterations for statistical significance
2. **Multiple measurement sessions**: Run tests at different times
3. **Document environmental factors**: Note network conditions, time of day
4. **Use consistent test environments**: Same machine, network, DNS settings

### For CI/CD Integration
1. **Use --no-color flag**: Prevents ANSI codes in logs
2. **Set conservative timeouts**: Account for variable CI/CD network conditions
3. **Limit test iterations**: Balance thoroughness with CI/CD speed
4. **Use environment variables**: Make configuration flexible per environment

### For Network Diagnostics
1. **Enable debug output**: Use --debug for troubleshooting
2. **Test incrementally**: Start with basic tests, add complexity
3. **Document baseline performance**: Establish normal performance ranges
4. **Compare different DNS providers**: Identify DNS-related issues

### Security Considerations
1. **Certificate validation**: Tool always validates TLS certificates
2. **DNS security**: Consider using DoH for encrypted DNS queries
3. **Network exposure**: Tool only makes outbound connections
4. **Log sensitivity**: Debug logs may contain URLs and timing data

---

For more detailed configuration options, see [Configuration Guide](configuration.md).
For troubleshooting specific issues, use the built-in help system:
```bash
network-latency-tester --help <topic>
```