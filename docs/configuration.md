# Network Latency Tester - Configuration Reference

## Overview

This document provides a comprehensive reference for configuring the Network Latency Tester. The tool supports multiple configuration methods with a clear precedence hierarchy, extensive validation, and platform-specific optimizations.

## Table of Contents

- [Configuration Priority](#configuration-priority)
- [Command-Line Arguments](#command-line-arguments)
- [Environment Variables](#environment-variables)
- [Configuration File (.env)](#configuration-file-env)
- [Default Values](#default-values)
- [Validation Rules](#validation-rules)
- [Platform-Specific Settings](#platform-specific-settings)
- [DNS Configuration](#dns-configuration)
- [Advanced Configuration](#advanced-configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Configuration Priority

The Network Latency Tester uses a hierarchical configuration system where settings are merged in the following order (highest to lowest priority):

1. **Command-line arguments** - Highest priority, overrides everything
2. **Environment variables** - Medium priority, can be set in shell or .env file
3. **Default values** - Lowest priority, platform-optimized defaults

### Example Priority Resolution
```bash
# If you have:
export TEST_COUNT=10                    # Environment variable
echo "TEST_COUNT=15" > .env             # .env file
network-latency-tester --count 20      # Command-line argument

# Result: count = 20 (command-line wins)
```

## Command-Line Arguments

### Basic Options

#### `--url <URL>` / `-u <URL>`
- **Description**: Target URL to test (can be used multiple times)
- **Type**: String (URL)
- **Validation**: Must be valid HTTP/HTTPS URL
- **Examples**:
  ```bash
  network-latency-tester --url https://google.com
  network-latency-tester -u https://github.com -u https://cloudflare.com
  ```

#### `--count <NUMBER>` / `-c <NUMBER>`
- **Description**: Number of test iterations per configuration
- **Type**: Integer
- **Range**: 1-100
- **Default**: 5
- **Examples**:
  ```bash
  network-latency-tester --count 10
  network-latency-tester -c 25
  ```

#### `--timeout <SECONDS>` / `-t <SECONDS>`
- **Description**: Request timeout in seconds
- **Type**: Integer
- **Range**: 1-300
- **Default**: 10 (platform-dependent)
- **Examples**:
  ```bash
  network-latency-tester --timeout 30
  network-latency-tester -t 60
  ```

### DNS Configuration Options

#### `--dns-servers <IPS>`
- **Description**: Custom DNS servers (comma-separated IP addresses)
- **Type**: Comma-separated list
- **Validation**: Must be valid IPv4 or IPv6 addresses
- **Examples**:
  ```bash
  network-latency-tester --dns-servers 8.8.8.8,8.8.4.4
  network-latency-tester --dns-servers 1.1.1.1,2001:4860:4860::8888
  ```

#### `--doh-providers <URLS>`
- **Description**: DNS-over-HTTPS providers (comma-separated HTTPS URLs)
- **Type**: Comma-separated list
- **Validation**: Must be valid HTTPS URLs
- **Examples**:
  ```bash
  network-latency-tester --doh-providers https://dns.google/dns-query
  network-latency-tester --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
  ```

### Output and Behavior Options

#### `--test-original`
- **Description**: Test the original target URL from the bash script
- **Type**: Flag (boolean)
- **Default**: false
- **Example**:
  ```bash
  network-latency-tester --test-original
  ```

#### `--verbose` / `-v`
- **Description**: Enable verbose output with detailed timing information
- **Type**: Flag (boolean)
- **Default**: false
- **Example**:
  ```bash
  network-latency-tester --verbose
  ```

#### `--debug`
- **Description**: Enable debug output with diagnostic information
- **Type**: Flag (boolean)
- **Default**: false
- **Example**:
  ```bash
  network-latency-tester --debug
  ```

#### `--no-color`
- **Description**: Disable colored output
- **Type**: Flag (boolean)
- **Default**: false (colors enabled if terminal supports them)
- **Example**:
  ```bash
  network-latency-tester --no-color
  ```

### Help Options

#### `--help [TOPIC]` / `-h`
- **Description**: Show help information (optionally for specific topic)
- **Type**: Optional string
- **Valid Topics**: config, dns, examples, timeout, output
- **Examples**:
  ```bash
  network-latency-tester --help
  network-latency-tester --help config
  network-latency-tester --help dns
  ```

## Environment Variables

### Core Configuration Variables

#### `TARGET_URLS`
- **Description**: Target URLs to test (comma-separated)
- **Format**: Comma-separated list of URLs
- **Validation**: Each URL must be valid HTTP/HTTPS
- **Example**: `TARGET_URLS=https://google.com,https://github.com,https://cloudflare.com`

#### `DNS_SERVERS`
- **Description**: Custom DNS servers (comma-separated IP addresses)
- **Format**: Comma-separated list of IP addresses
- **Validation**: Must be valid IPv4 or IPv6 addresses
- **Example**: `DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9`

#### `DOH_PROVIDERS`
- **Description**: DNS-over-HTTPS providers (comma-separated HTTPS URLs)
- **Format**: Comma-separated list of HTTPS URLs
- **Validation**: Must be valid HTTPS URLs ending in typical DoH paths
- **Example**: `DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query`

#### `TEST_COUNT`
- **Description**: Number of test iterations per configuration
- **Format**: Integer
- **Range**: 1-100
- **Example**: `TEST_COUNT=15`

#### `TIMEOUT_SECONDS`
- **Description**: Request timeout in seconds
- **Format**: Integer
- **Range**: 1-300
- **Example**: `TIMEOUT_SECONDS=25`

#### `ENABLE_COLOR`
- **Description**: Enable colored output
- **Format**: Boolean (true/false)
- **Case-Sensitive**: Only lowercase "true" and "false" accepted
- **Example**: `ENABLE_COLOR=false`

### Environment Variable Loading

The tool loads environment variables in this order:

1. **System environment variables**: Set in your shell session
2. **`.env` file**: Located in the current working directory
3. **Command-line overrides**: Applied last, highest priority

### Setting Environment Variables

#### In Shell (Temporary)
```bash
export TARGET_URLS="https://google.com,https://github.com"
export DNS_SERVERS="8.8.8.8,1.1.1.1"
export TEST_COUNT=10
network-latency-tester
```

#### In Shell Profile (Permanent)
Add to `~/.bashrc`, `~/.zshrc`, or equivalent:
```bash
export TARGET_URLS="https://google.com,https://github.com"
export DNS_SERVERS="8.8.8.8,1.1.1.1"
export TEST_COUNT=10
```

#### In .env File (Project-specific)
Create a `.env` file in your working directory:
```env
# Network Latency Tester Configuration
TARGET_URLS=https://google.com,https://github.com,https://cloudflare.com
DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
TEST_COUNT=15
TIMEOUT_SECONDS=25
ENABLE_COLOR=true
```

## Configuration File (.env)

### .env File Format

The `.env` file uses a simple `KEY=VALUE` format:

```env
# Comments are supported with #
TARGET_URLS=https://example.com,https://another.com

# Spaces around = are not supported
DNS_SERVERS=8.8.8.8,1.1.1.1

# Boolean values must be lowercase
ENABLE_COLOR=true

# Numeric values as plain integers
TEST_COUNT=10
TIMEOUT_SECONDS=30

# Long URLs can be split (no line continuation supported)
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
```

### .env File Location

The tool looks for `.env` file in the current working directory where you run the command:

```bash
# This will look for .env in /home/user/project/
cd /home/user/project/
network-latency-tester

# This will look for .env in /home/user/
cd /home/user/
network-latency-tester
```

### Creating Example .env File

Generate an example .env file:
```bash
network-latency-tester --help config
# This displays configuration help including example .env content
```

### .env File Best Practices

1. **Use comments**: Document your configuration choices
2. **Group related settings**: Organize DNS settings together
3. **Use meaningful values**: Choose appropriate test counts and timeouts
4. **Version control**: Consider whether to commit .env files (usually don't)
5. **Environment-specific files**: Use `.env.development`, `.env.production`, etc.

## Default Values

### Platform-Independent Defaults

- **Test Count**: 5 iterations
- **Enable Color**: true (if terminal supports colors)
- **Verbose**: false
- **Debug**: false

### Platform-Specific Defaults

#### Windows
- **Connection Timeout**: 10 seconds
- **Request Timeout**: 30 seconds  
- **Max Concurrent Connections**: 10
- **DNS Servers**: 8.8.8.8, 1.1.1.1, 208.67.222.222 (Google, Cloudflare, OpenDNS)
- **DoH Providers**: Google, Cloudflare, OpenDNS

#### macOS
- **Connection Timeout**: 5 seconds
- **Request Timeout**: 20 seconds
- **Max Concurrent Connections**: 15
- **DNS Servers**: 8.8.8.8, 1.1.1.1, 2001:4860:4860::8888 (includes IPv6)
- **DoH Providers**: Google, Cloudflare, OpenDNS

#### Linux
- **Connection Timeout**: 5 seconds
- **Request Timeout**: 20 seconds
- **Max Concurrent Connections**: 20
- **DNS Servers**: 8.8.8.8, 1.1.1.1, 9.9.9.9 (Google, Cloudflare, Quad9)
- **DoH Providers**: Google, Cloudflare, Quad9

## Validation Rules

### URL Validation
- Must start with `http://` or `https://`
- Must have a valid hostname or IP address
- Port numbers are allowed (e.g., `https://example.com:8080`)
- Path and query parameters are allowed
- Fragment identifiers are allowed but ignored

**Valid Examples**:
```
https://example.com
http://192.168.1.1:8080
https://api.service.com/v1/health?check=true
```

**Invalid Examples**:
```
ftp://example.com          # Unsupported protocol
example.com                # Missing protocol
https://                   # Missing hostname
```

### DNS Server Validation
- Must be valid IPv4 or IPv6 addresses
- Hostnames are not allowed (must resolve to IP first)
- Port numbers are not supported (DNS uses standard ports)

**Valid Examples**:
```
8.8.8.8                    # IPv4
2001:4860:4860::8888       # IPv6
192.168.1.1                # Private IPv4
```

**Invalid Examples**:
```
dns.google.com             # Hostname not allowed
8.8.8.8:53                # Port not supported
256.256.256.256           # Invalid IPv4
```

### DoH Provider Validation
- Must be valid HTTPS URLs (HTTP not allowed for security)
- Must have a hostname (IP addresses discouraged but allowed)
- Typically end in `/dns-query` but not required
- Must respond to DoH protocol (not validated until runtime)

**Valid Examples**:
```
https://dns.google/dns-query
https://cloudflare-dns.com/dns-query
https://custom-doh-server.com/resolve
```

**Invalid Examples**:
```
http://dns.google/dns-query    # HTTP not allowed
https://                       # Missing hostname
dns.google/dns-query          # Missing protocol
```

### Numeric Validation

#### Test Count
- **Range**: 1-100
- **Reasoning**: 1 minimum for meaningful results, 100 maximum to prevent excessive load

#### Timeout
- **Range**: 1-300 seconds
- **Reasoning**: 1 second minimum for basic connectivity, 5 minutes maximum for very slow connections

### Boolean Validation
Environment variables only accept lowercase `true` and `false`:

**Valid**: `ENABLE_COLOR=true`, `ENABLE_COLOR=false`  
**Invalid**: `ENABLE_COLOR=True`, `ENABLE_COLOR=1`, `ENABLE_COLOR=yes`

## Platform-Specific Settings

### Windows-Specific Configuration

Windows networking can be slower and less predictable, so the tool applies conservative settings:

- **Extended Timeouts**: 20% longer than Unix systems
- **Certificate Validation**: Uses Windows certificate store
- **IPv6 Preference**: Disabled by default (inconsistent support)
- **Concurrency**: Lower limits due to Windows networking characteristics

### macOS-Specific Configuration

macOS has excellent networking performance and IPv6 support:

- **Aggressive Timeouts**: Shorter timeouts for faster failure detection
- **IPv6 Preference**: Enabled (excellent IPv6 support)
- **TLS 1.3**: Preferred when available
- **High Concurrency**: Supports more concurrent connections

### Linux-Specific Configuration

Linux systems, especially servers, have optimized networking:

- **High Performance**: Optimized for server workloads
- **systemd-resolved**: Considers systemd DNS resolution
- **IPv6**: Enabled but depends on system configuration
- **Maximum Concurrency**: Highest concurrent connection limits

### Unknown Platform Configuration

For unrecognized platforms, conservative defaults are used:

- **Conservative Timeouts**: Longer timeouts for reliability
- **Limited Features**: Basic HTTP/1.1, disabled HTTP/2
- **IPv6**: Disabled for maximum compatibility
- **Low Concurrency**: Minimal concurrent connections

## DNS Configuration

### DNS Resolver Types

#### 1. System DNS
- **Description**: Uses the operating system's default DNS resolver
- **Configuration**: Automatic, no additional setup needed
- **Platform Integration**: Respects system DNS settings
- **Use Cases**: Default testing, respecting corporate DNS policies

#### 2. Custom DNS Servers
- **Description**: Uses specified DNS servers for resolution
- **Configuration**: Provide IP addresses via `--dns-servers` or `DNS_SERVERS`
- **Validation**: Must be valid IPv4 or IPv6 addresses
- **Use Cases**: Testing with specific DNS providers, bypassing local DNS issues

#### 3. DNS-over-HTTPS (DoH)
- **Description**: Encrypted DNS queries over HTTPS
- **Configuration**: Provide HTTPS URLs via `--doh-providers` or `DOH_PROVIDERS`
- **Security**: Prevents DNS interception and manipulation
- **Use Cases**: Privacy-focused testing, bypassing DNS filtering

### Popular DNS Providers

#### Public DNS Servers
```env
# Google DNS
DNS_SERVERS=8.8.8.8,8.8.4.4,2001:4860:4860::8888,2001:4860:4860::8844

# Cloudflare DNS
DNS_SERVERS=1.1.1.1,1.0.0.1,2606:4700:4700::1111,2606:4700:4700::1001

# OpenDNS
DNS_SERVERS=208.67.222.222,208.67.220.220,2620:119:35::35,2620:119:53::53

# Quad9 DNS
DNS_SERVERS=9.9.9.9,149.112.112.112,2620:fe::fe,2620:fe::9
```

#### DoH Providers
```env
# Google DoH
DOH_PROVIDERS=https://dns.google/dns-query

# Cloudflare DoH
DOH_PROVIDERS=https://cloudflare-dns.com/dns-query

# Quad9 DoH
DOH_PROVIDERS=https://dns.quad9.net/dns-query

# Multiple providers
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query,https://dns.quad9.net/dns-query
```

### DNS Configuration Best Practices

1. **Test Multiple Providers**: Compare performance across different DNS providers
2. **Consider Geography**: Use DNS servers geographically close to your targets
3. **Mix IPv4 and IPv6**: Test both protocols if your network supports IPv6
4. **Use DoH for Privacy**: Consider DoH for sensitive or monitored networks
5. **Validate DNS Health**: Use `--help dns` to check DNS configuration guidance

## Advanced Configuration

### Performance Tuning

#### High-Frequency Testing
For load testing or detailed performance analysis:
```env
TARGET_URLS=https://api.production.com/health
TEST_COUNT=50
TIMEOUT_SECONDS=5
DNS_SERVERS=8.8.8.8,1.1.1.1  # Fast, reliable DNS
```

#### Conservative Testing
For unreliable networks or distant servers:
```env
TARGET_URLS=https://slow-server.example.com
TEST_COUNT=3
TIMEOUT_SECONDS=60
DNS_SERVERS=8.8.8.8,9.9.9.9  # Multiple providers for reliability
```

#### Diagnostic Testing
For troubleshooting network issues:
```env
TARGET_URLS=https://problematic-site.com
TEST_COUNT=10
TIMEOUT_SECONDS=30
DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9,208.67.222.222
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query
ENABLE_COLOR=true
```

### Automation Configuration

#### CI/CD Pipeline Configuration
```env
# Optimized for automated testing
TARGET_URLS=https://api.service.com/health,https://app.service.com
TEST_COUNT=5
TIMEOUT_SECONDS=15
ENABLE_COLOR=false  # Disable colors for log clarity
DNS_SERVERS=8.8.8.8  # Single, fast DNS server
```

#### Monitoring Configuration
```env
# For continuous monitoring
TARGET_URLS=https://critical-service.com/health
TEST_COUNT=3
TIMEOUT_SECONDS=10
DNS_SERVERS=8.8.8.8,1.1.1.1  # Fast, redundant DNS
ENABLE_COLOR=false
```

### Environment-Specific Configuration

#### Development Environment
```env
# .env.development
TARGET_URLS=http://localhost:3000,http://localhost:8080
TEST_COUNT=3
TIMEOUT_SECONDS=5
ENABLE_COLOR=true
```

#### Staging Environment  
```env
# .env.staging
TARGET_URLS=https://staging-api.service.com,https://staging-app.service.com
TEST_COUNT=5
TIMEOUT_SECONDS=15
DNS_SERVERS=8.8.8.8,1.1.1.1
ENABLE_COLOR=false
```

#### Production Environment
```env
# .env.production
TARGET_URLS=https://api.service.com,https://app.service.com
TEST_COUNT=10
TIMEOUT_SECONDS=30
DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9
DOH_PROVIDERS=https://dns.google/dns-query
ENABLE_COLOR=false
```

## Examples

### Complete Configuration Examples

#### Example 1: Basic Web Service Testing
```bash
# Command-line approach
network-latency-tester \
  --url https://api.myservice.com \
  --url https://cdn.myservice.com \
  --count 10 \
  --timeout 20 \
  --dns-servers 8.8.8.8,1.1.1.1 \
  --verbose

# Environment variable approach
export TARGET_URLS="https://api.myservice.com,https://cdn.myservice.com"
export DNS_SERVERS="8.8.8.8,1.1.1.1"
export TEST_COUNT=10
export TIMEOUT_SECONDS=20
network-latency-tester --verbose
```

#### Example 2: Comprehensive DNS Testing
```env
# .env file for comprehensive DNS testing
TARGET_URLS=https://google.com,https://cloudflare.com,https://github.com
DNS_SERVERS=8.8.8.8,1.1.1.1,9.9.9.9,208.67.222.222
DOH_PROVIDERS=https://dns.google/dns-query,https://cloudflare-dns.com/dns-query,https://dns.quad9.net/dns-query
TEST_COUNT=15
TIMEOUT_SECONDS=25
ENABLE_COLOR=true
```

#### Example 3: Load Testing Configuration
```env
# High-frequency load testing
TARGET_URLS=https://api.loadtest.com/endpoint1,https://api.loadtest.com/endpoint2
TEST_COUNT=100
TIMEOUT_SECONDS=5
DNS_SERVERS=8.8.8.8  # Single fast DNS server
ENABLE_COLOR=false   # For clean log output
```

#### Example 4: Network Diagnostics
```bash
# Comprehensive diagnostic testing
network-latency-tester \
  --url https://problematic-site.com \
  --dns-servers 8.8.8.8,1.1.1.1,9.9.9.9,208.67.222.222 \
  --doh-providers https://dns.google/dns-query,https://cloudflare-dns.com/dns-query \
  --count 20 \
  --timeout 60 \
  --debug \
  --verbose
```

## Troubleshooting

### Configuration Issues

#### Issue: "Invalid URL format"
**Cause**: URL doesn't start with http:// or https://  
**Solution**: Ensure URLs include the protocol:
```bash
# Wrong
network-latency-tester --url example.com

# Correct
network-latency-tester --url https://example.com
```

#### Issue: "Invalid DNS server IP address"
**Cause**: DNS server is not a valid IP address  
**Solution**: Use IP addresses, not hostnames:
```bash
# Wrong
network-latency-tester --dns-servers dns.google.com

# Correct
network-latency-tester --dns-servers 8.8.8.8
```

#### Issue: "Test count must be between 1 and 100"
**Cause**: Test count is outside valid range  
**Solution**: Use valid range:
```bash
# Wrong
network-latency-tester --count 150

# Correct
network-latency-tester --count 50
```

#### Issue: "DoH URL must use HTTPS"
**Cause**: DoH provider URL uses HTTP instead of HTTPS  
**Solution**: Use HTTPS URLs only:
```bash
# Wrong
network-latency-tester --doh-providers http://dns.google/dns-query

# Correct
network-latency-tester --doh-providers https://dns.google/dns-query
```

### Environment Variable Issues

#### Issue: Boolean environment variables not working
**Cause**: Using capitalized or non-standard boolean values  
**Solution**: Use lowercase "true" or "false":
```bash
# Wrong
export ENABLE_COLOR=True
export ENABLE_COLOR=1
export ENABLE_COLOR=yes

# Correct
export ENABLE_COLOR=true
export ENABLE_COLOR=false
```

#### Issue: .env file not loaded
**Cause**: .env file not in current working directory  
**Solution**: Ensure .env file is in the directory where you run the command:
```bash
# Check current directory
pwd
ls -la .env

# Or use absolute path
cd /path/to/project
network-latency-tester
```

#### Issue: Environment variables not overriding defaults
**Cause**: Command-line arguments taking precedence  
**Solution**: Remember the priority order: CLI > Environment > Defaults
```bash
# Environment variable set
export TEST_COUNT=20

# But command-line argument overrides
network-latency-tester --count 10  # Uses 10, not 20
```

### Platform-Specific Issues

#### Windows Issues
- **Slow DNS resolution**: Try different DNS servers or DoH
- **Firewall blocking**: Run as Administrator or configure Windows Firewall
- **Certificate errors**: Windows certificate store issues

#### macOS Issues
- **System integrity protection**: Some network diagnostics may be limited
- **IPv6 configuration**: Check network preferences for IPv6 settings

#### Linux Issues
- **systemd-resolved conflicts**: Check `/etc/systemd/resolved.conf`
- **Network namespace issues**: Ensure proper network access
- **IPv6 disabled**: Check if IPv6 is enabled in network configuration

### Getting Configuration Help

```bash
# General configuration help
network-latency-tester --help config

# DNS-specific help
network-latency-tester --help dns

# See current configuration (when running)
network-latency-tester --url https://example.com --debug
```

---

For usage examples and common scenarios, see [Usage Guide](usage.md).  
For troubleshooting network issues, use: `network-latency-tester --help dns`