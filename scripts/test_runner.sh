#!/bin/bash

# Network Latency Tester - Automated Test Runner
# 
# This script provides comprehensive testing for CI/CD environments
# and validates feature parity with the original bash script.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY_NAME="network-latency-tester"
BINARY_PATH=""
TEST_RESULTS_DIR="$PROJECT_DIR/test-results"
LOG_FILE="$TEST_RESULTS_DIR/test_runner.log"
PARALLEL_JOBS="${PARALLEL_JOBS:-4}"
VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test configuration
declare -a TEST_URLS=(
    "https://httpbin.org/delay/0.1"
    "https://www.bing.com"
    "https://apple.com"
)

declare -a TEST_DNS_SERVERS=(
    "120.53.53.102"
    "223.5.5.5"
    "223.6.6.6"
)

declare -a TEST_DOH_URLS=(
    "https://137618-io7m09tk35h1lurw.alidns.com/dns-query"
    "https://hk1.pro.xns.one/6EMqIkLe5E4/dns-query"
)

# Counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Initialize
initialize() {
    echo -e "${BLUE}🚀 Network Latency Tester - Automated Test Runner${NC}"
    echo -e "${BLUE}=================================================${NC}"
    
    # Create results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Initialize log file
    echo "Test run started at $(date)" > "$LOG_FILE"
    
    # Find or build binary
    find_or_build_binary
    
    echo -e "${GREEN}✓ Initialization complete${NC}"
    echo
}

# Find or build the binary
find_or_build_binary() {
    echo -e "${CYAN}🔍 Locating binary...${NC}"
    
    # Try different locations
    local possible_paths=(
        "$PROJECT_DIR/target/release/$BINARY_NAME"
        "$PROJECT_DIR/target/debug/$BINARY_NAME"
        "$(which $BINARY_NAME 2>/dev/null || true)"
    )
    
    for path in "${possible_paths[@]}"; do
        if [[ -x "$path" ]]; then
            BINARY_PATH="$path"
            echo -e "${GREEN}✓ Found binary at: $BINARY_PATH${NC}"
            return 0
        fi
    done
    
    # Build if not found
    echo -e "${YELLOW}⚠ Binary not found, building...${NC}"
    cd "$PROJECT_DIR"
    
    if command -v cargo &> /dev/null; then
        cargo build --release
        BINARY_PATH="$PROJECT_DIR/target/release/$BINARY_NAME"
        if [[ -x "$BINARY_PATH" ]]; then
            echo -e "${GREEN}✓ Built binary successfully${NC}"
            return 0
        fi
    fi
    
    echo -e "${RED}❌ Failed to find or build binary${NC}"
    exit 1
}

# Logging functions
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG_FILE"
    [[ "$VERBOSE" == "true" ]] && echo -e "$*"
}

# Test execution functions
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    local expected_result="${3:-success}"
    local timeout="${4:-30}"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    echo -e "${CYAN}🧪 Running: $test_name${NC}"
    log "Running test: $test_name"
    log "Command: $test_cmd"
    
    local start_time=$(date +%s)
    local test_output=""
    local test_exit_code=0
    
    # Run test with timeout
    if test_output=$(timeout "$timeout" bash -c "$test_cmd" 2>&1); then
        test_exit_code=$?
    else
        test_exit_code=$?
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Evaluate result
    local test_passed=false
    case "$expected_result" in
        "success")
            [[ $test_exit_code -eq 0 ]] && test_passed=true
            ;;
        "failure")
            [[ $test_exit_code -ne 0 ]] && test_passed=true
            ;;
        "timeout")
            [[ $test_exit_code -eq 124 ]] && test_passed=true
            ;;
        *)
            # Custom validation function
            if command -v "$expected_result" &> /dev/null; then
                if "$expected_result" "$test_output" "$test_exit_code"; then
                    test_passed=true
                fi
            fi
            ;;
    esac
    
    # Record result
    if [[ "$test_passed" == "true" ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        echo -e "${GREEN}✓ PASSED${NC} (${duration}s)"
        log "PASSED: $test_name (${duration}s)"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo -e "${RED}❌ FAILED${NC} (exit code: $test_exit_code, duration: ${duration}s)"
        log "FAILED: $test_name (exit code: $test_exit_code, duration: ${duration}s)"
        log "Output: $test_output"
        
        # Save failed test output
        local fail_file="$TEST_RESULTS_DIR/failed_${test_name//[^a-zA-Z0-9]/_}.log"
        echo "Test: $test_name" > "$fail_file"
        echo "Command: $test_cmd" >> "$fail_file"
        echo "Exit Code: $test_exit_code" >> "$fail_file"
        echo "Duration: ${duration}s" >> "$fail_file"
        echo "Output:" >> "$fail_file"
        echo "$test_output" >> "$fail_file"
    fi
    
    echo
}

# Validation functions for custom test results
validate_timing_output() {
    local output="$1"
    local exit_code="$2"
    
    [[ $exit_code -eq 0 ]] || return 1
    [[ "$output" =~ [0-9]+\.[0-9]+.*ms ]] || return 1
    [[ "$output" =~ (网络延迟测试结果|Network.*Latency.*Test) ]] || return 1
    return 0
}

validate_verbose_output() {
    local output="$1"
    local exit_code="$2"
    
    [[ $exit_code -eq 0 ]] || return 1
    [[ "$output" =~ (DETAILED.*TIMING|详细时间分析) ]] || return 1
    [[ "$output" =~ (DNS.*Resolution|DNS解析) ]] || return 1
    [[ "$output" =~ (INDIVIDUAL.*REQUEST|单个请求) ]] || return 1
    return 0
}

validate_debug_output() {
    local output="$1"
    local exit_code="$2"
    
    [[ $exit_code -eq 0 ]] || return 1
    [[ "$output" =~ (DEBUG|INFO) ]] || return 1
    return 0
}

validate_help_output() {
    local output="$1"
    local exit_code="$2"
    
    [[ $exit_code -eq 0 ]] || return 1
    [[ "$output" =~ (--url|--dns|--count|--timeout) ]] || return 1
    return 0
}

validate_error_output() {
    local output="$1"
    local exit_code="$2"
    
    [[ $exit_code -ne 0 ]] || return 1
    [[ "$output" =~ (Error|错误|Invalid|无效) ]] || return 1
    return 0
}

# Test suites
run_basic_functionality_tests() {
    echo -e "${PURPLE}📋 Basic Functionality Tests${NC}"
    echo -e "${PURPLE}=============================${NC}"
    
    # Basic execution
    run_test "basic_execution" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 2 --timeout 10" \
        "validate_timing_output"
    
    # Help output
    run_test "help_output" \
        "'$BINARY_PATH' --help" \
        "validate_help_output"
    
    # Version output
    run_test "version_output" \
        "'$BINARY_PATH' --version" \
        "success"
    
    # Multiple URLs
    run_test "multiple_urls" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --url '${TEST_URLS[1]}' --count 2 --timeout 15" \
        "validate_timing_output"
}

run_dns_configuration_tests() {
    echo -e "${PURPLE}📋 DNS Configuration Tests${NC}"
    echo -e "${PURPLE}==========================${NC}"
    
    # Custom DNS
    run_test "custom_dns" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --dns '${TEST_DNS_SERVERS[0]}' --count 2 --timeout 10" \
        "validate_timing_output"
    
    # Multiple DNS servers
    run_test "multiple_dns" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --dns '${TEST_DNS_SERVERS[0]}' --dns '${TEST_DNS_SERVERS[1]}' --count 2 --timeout 10" \
        "validate_timing_output"
    
    # DNS over HTTPS
    run_test "dns_over_https" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --doh '${TEST_DOH_URLS[0]}' --count 2 --timeout 15" \
        "validate_timing_output"
}

run_output_format_tests() {
    echo -e "${PURPLE}📋 Output Format Tests${NC}"
    echo -e "${PURPLE}======================${NC}"
    
    # Verbose output
    run_test "verbose_output" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 3 --verbose --timeout 10" \
        "validate_verbose_output"
    
    # Debug output
    run_test "debug_output" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 2 --debug --timeout 10" \
        "validate_debug_output"
    
    # Color output
    run_test "color_output" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 2 --color --timeout 10" \
        "validate_timing_output"
    
    # No color output
    run_test "no_color_output" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 2 --no-color --timeout 10" \
        "validate_timing_output"
}

run_error_handling_tests() {
    echo -e "${PURPLE}📋 Error Handling Tests${NC}"
    echo -e "${PURPLE}=======================${NC}"
    
    # Invalid URL
    run_test "invalid_url" \
        "'$BINARY_PATH' --url 'not-a-valid-url' --count 1 --timeout 5" \
        "validate_error_output"
    
    # Invalid DNS server
    run_test "invalid_dns" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --dns 'not.a.valid.ip' --count 1 --timeout 5" \
        "validate_error_output"
    
    # Invalid timeout
    run_test "invalid_timeout" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --timeout 0 --count 1" \
        "validate_error_output"
    
    # Invalid count
    run_test "invalid_count" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 0 --timeout 5" \
        "validate_error_output"
}

run_performance_tests() {
    echo -e "${PURPLE}📋 Performance Tests${NC}"
    echo -e "${PURPLE}===================${NC}"
    
    # High count test
    run_test "high_count_test" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 10 --timeout 20" \
        "validate_timing_output"
    
    # Concurrent DNS test
    run_test "concurrent_dns_test" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --dns '${TEST_DNS_SERVERS[0]}' --dns '${TEST_DNS_SERVERS[1]}' --dns '${TEST_DNS_SERVERS[2]}' --count 5 --timeout 25" \
        "validate_timing_output"
    
    # Memory usage test (long running)
    run_test "memory_usage_test" \
        "'$BINARY_PATH' --url '${TEST_URLS[0]}' --count 20 --timeout 30" \
        "validate_timing_output" \
        45
}

run_integration_tests() {
    echo -e "${PURPLE}📋 Integration Tests${NC}"
    echo -e "${PURPLE}===================${NC}"
    
    # Run actual Rust integration tests
    if command -v cargo &> /dev/null; then
        run_test "rust_unit_tests" \
            "cd '$PROJECT_DIR' && cargo test --lib" \
            "success" \
            60
        
        run_test "rust_integration_tests" \
            "cd '$PROJECT_DIR' && cargo test --test '*'" \
            "success" \
            120
        
        run_test "rust_e2e_tests" \
            "cd '$PROJECT_DIR' && cargo test e2e" \
            "success" \
            180
        
        run_test "rust_validation_tests" \
            "cd '$PROJECT_DIR' && cargo test validation" \
            "success" \
            120
    else
        echo -e "${YELLOW}⚠ Cargo not available, skipping Rust tests${NC}"
        TESTS_SKIPPED=$((TESTS_SKIPPED + 4))
    fi
}

# Environment configuration tests
run_environment_tests() {
    echo -e "${PURPLE}📋 Environment Configuration Tests${NC}"
    echo -e "${PURPLE}==================================${NC}"
    
    # Create temporary config file
    local temp_env_file=$(mktemp)
    cat > "$temp_env_file" << EOF
TARGET_URLS=${TEST_URLS[0]}
TEST_COUNT=3
TIMEOUT_SECONDS=10
DEBUG=false
EOF
    
    # Test environment file loading
    run_test "environment_config" \
        "DOTENV_PATH='$temp_env_file' '$BINARY_PATH'" \
        "validate_timing_output"
    
    # Test CLI override of environment
    run_test "cli_override_env" \
        "DOTENV_PATH='$temp_env_file' '$BINARY_PATH' --url '${TEST_URLS[1]}' --count 2" \
        "validate_timing_output"
    
    # Cleanup
    rm -f "$temp_env_file"
}

# Generate test report
generate_report() {
    echo -e "${BLUE}📊 Test Results Summary${NC}"
    echo -e "${BLUE}======================${NC}"
    
    local success_rate=0
    if [[ $TESTS_TOTAL -gt 0 ]]; then
        success_rate=$((TESTS_PASSED * 100 / TESTS_TOTAL))
    fi
    
    echo -e "Total Tests:  ${TESTS_TOTAL}"
    echo -e "Passed:       ${GREEN}${TESTS_PASSED}${NC}"
    echo -e "Failed:       ${RED}${TESTS_FAILED}${NC}"
    echo -e "Skipped:      ${YELLOW}${TESTS_SKIPPED}${NC}"
    echo -e "Success Rate: ${success_rate}%"
    echo
    
    # Generate detailed report file
    local report_file="$TEST_RESULTS_DIR/test_report_$(date +%Y%m%d_%H%M%S).txt"
    {
        echo "Network Latency Tester - Test Report"
        echo "====================================="
        echo "Generated: $(date)"
        echo "Binary: $BINARY_PATH"
        echo ""
        echo "Summary:"
        echo "  Total Tests:  $TESTS_TOTAL"
        echo "  Passed:       $TESTS_PASSED"
        echo "  Failed:       $TESTS_FAILED"
        echo "  Skipped:      $TESTS_SKIPPED"
        echo "  Success Rate: ${success_rate}%"
        echo ""
        echo "Detailed Log:"
        cat "$LOG_FILE"
    } > "$report_file"
    
    echo -e "${GREEN}📄 Detailed report saved to: $report_file${NC}"
    
    # Return appropriate exit code
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "${RED}❌ Some tests failed${NC}"
        return 1
    else
        echo -e "${GREEN}✅ All tests passed${NC}"
        return 0
    fi
}

# Cleanup function
cleanup() {
    log "Test run completed at $(date)"
    echo -e "${CYAN}🧹 Cleanup complete${NC}"
}

# Signal handlers
trap cleanup EXIT

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Network Latency Tester - Automated Test Runner

OPTIONS:
    -v, --verbose           Enable verbose output
    -j, --jobs NUM          Number of parallel jobs (default: 4)
    -h, --help             Show this help message
    
ENVIRONMENT VARIABLES:
    VERBOSE                Enable verbose output (true/false)
    PARALLEL_JOBS          Number of parallel jobs
    
EXAMPLES:
    $0                     Run all tests
    $0 --verbose           Run with verbose output
    $0 --jobs 8            Run with 8 parallel jobs
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -j|--jobs)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    initialize
    
    # Run test suites
    run_basic_functionality_tests
    run_dns_configuration_tests
    run_output_format_tests
    run_error_handling_tests
    run_environment_tests
    run_performance_tests
    run_integration_tests
    
    # Generate report
    generate_report
}

# Execute main function
main "$@"