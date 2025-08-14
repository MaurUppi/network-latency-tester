#!/usr/bin/env bash
# enhanced_check_ctok.sh
# 增强版网络延迟测试脚本
# 使用方法：chmod +x enhanced_check_ctok.sh && ./enhanced_check_ctok.sh

# 配置参数
TEST_COUNT=5                    # 每个配置的测试次数
TIMEOUT=10                      # 超时时间（秒）
ENABLE_COLOR=true              # 是否启用彩色输出
DNS_SERVERS_SUPPORTED=false     # curl是否支持 --dns-servers
CUSTOM_URL=""                   # 自定义测试URL
TEST_ORIGINAL=false             # 测试原始target URL

# 测试 URL 列表
urls=(
    "https://as.target"
)

# DNS 配置 - 注意：系统默认必须为空字符串！
declare -A dns_configs=(
    ["系统默认"]=""
    ["腾讯"]="120.53.53.102"
    ["阿里"]="223.5.5.5,223.6.6.6"
)

# DoH 配置
declare -A doh_configs=(
    ["Aliyun DoH"]="https://137618-io7m09tk35h1lurw.alidns.com/dns-query"
    ["NovaXNS"]="https://hk1.pro.xns.one/6EMqIkLe5E4/dns-query"
)

# 颜色定义
if [ "$ENABLE_COLOR" = true ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    PURPLE='\033[0;35m'
    CYAN='\033[0;36m'
    WHITE='\033[1;37m'
    NC='\033[0m' # No Color
else
    RED='' GREEN='' YELLOW='' BLUE='' PURPLE='' CYAN='' WHITE='' NC=''
fi

# 数学计算工具检测
MATH_TOOL=""

detect_math_tool() {
    if command -v bc &> /dev/null; then
        MATH_TOOL="bc"
        return 0
    elif command -v awk &> /dev/null; then
        MATH_TOOL="awk"
        return 0
    else
        return 1
    fi
}

# 数学计算函数（支持bc和awk）
math_calc() {
    local expression="$1"
    case "$MATH_TOOL" in
        "bc")
            echo "$expression" | bc -l
            ;;
        "awk")
            awk "BEGIN { print $expression }"
            ;;
        *)
            echo "0"
            ;;
    esac
}

# 浮点数比较函数
float_compare() {
    local num1="$1"
    local operator="$2"
    local num2="$3"
    
    case "$MATH_TOOL" in
        "bc")
            [ "$(echo "$num1 $operator $num2" | bc -l)" = "1" ]
            ;;
        "awk")
            awk "BEGIN { exit !($num1 $operator $num2) }"
            ;;
        *)
            return 1
            ;;
    esac
}

# 工具函数
print_header() {
    echo -e "${WHITE}================================================${NC}"
    echo -e "${WHITE}          网络延迟综合测试工具${NC}"
    echo -e "${WHITE}================================================${NC}"
    echo
}

print_section() {
    echo -e "${CYAN}--- $1 ---${NC}"
}

print_error() {
    echo -e "${RED}错误: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# 计算统计值（兼容bc和awk）
calculate_stats() {
    local values=("$@")
    local count=${#values[@]}
    
    if [ $count -eq 0 ]; then
        echo "0 0 0 0"
        return
    fi
    
    # 将数组转换为awk可处理的字符串
    local values_str=""
    for val in "${values[@]}"; do
        values_str="$values_str $val"
    done
    
    # 使用awk计算所有统计值
    case "$MATH_TOOL" in
        "bc")
            # BC版本的计算
            local sum=0
            for val in "${values[@]}"; do
                sum=$(echo "$sum + $val" | bc -l)
            done
            local avg=$(echo "scale=3; $sum / $count" | bc -l)
            
            local min=${values[0]}
            local max=${values[0]}
            for val in "${values[@]}"; do
                if (( $(echo "$val < $min" | bc -l) )); then
                    min=$val
                fi
                if (( $(echo "$val > $max" | bc -l) )); then
                    max=$val
                fi
            done
            
            local variance=0
            for val in "${values[@]}"; do
                local diff=$(echo "$val - $avg" | bc -l)
                variance=$(echo "$variance + ($diff * $diff)" | bc -l)
            done
            variance=$(echo "scale=6; $variance / $count" | bc -l)
            local stddev=$(echo "scale=3; sqrt($variance)" | bc -l)
            
            echo "$avg $min $max $stddev"
            ;;
        "awk")
            # AWK版本的计算
            awk -v count="$count" -v values="$values_str" '
            BEGIN {
                n = split(values, arr, " ")
                
                # 计算总和和平均值
                sum = 0
                for (i = 1; i <= n; i++) {
                    if (arr[i] != "") {
                        sum += arr[i]
                    }
                }
                avg = sum / count
                
                # 找最小值和最大值
                min = arr[1]
                max = arr[1]
                for (i = 1; i <= n; i++) {
                    if (arr[i] != "") {
                        if (arr[i] < min) min = arr[i]
                        if (arr[i] > max) max = arr[i]
                    }
                }
                
                # 计算标准差
                variance = 0
                for (i = 1; i <= n; i++) {
                    if (arr[i] != "") {
                        diff = arr[i] - avg
                        variance += diff * diff
                    }
                }
                variance = variance / count
                stddev = sqrt(variance)
                
                printf "%.3f %.3f %.3f %.3f\n", avg, min, max, stddev
            }'
            ;;
        *)
            echo "0 0 0 0"
            ;;
    esac
}

# 检查curl版本和功能支持
check_curl_features() {
    local curl_version
    curl_version=$(curl --version | head -n1)
    echo "Curl版本: $curl_version"
    
    # 测试 --dns-servers 支持
    echo -n "测试DNS服务器参数支持: "
    local test_result
    test_result=$(timeout 5 curl --dns-servers 8.8.8.8 -sS -I -o /dev/null --connect-timeout 3 --max-time 5 "https://httpbin.org/get" 2>&1)
    local test_exit=$?
    
    if [ $test_exit -eq 0 ] && ! echo "$test_result" | grep -qE "(Unknown option|unrecognized option|or 'curl --manual'|not compiled|requires|unsupported)"; then
        print_success "DNS服务器指定 (--dns-servers): 已支持且可用"
        DNS_SERVERS_SUPPORTED=true
    else
        print_warning "DNS服务器指定 (--dns-servers): 不可用"
        DNS_SERVERS_SUPPORTED=false
    fi
    
    # 检查DoH支持 - 更宽松的检测
    if curl --help all 2>/dev/null | grep -q "doh-url"; then
        print_success "DoH (DNS over HTTPS): 参数支持已确认"
        
        # 简单测试DoH功能
        echo -n "测试DoH实际功能: "
        local doh_test
        doh_test=$(timeout 8 curl --doh-url https://cloudflare-dns.com/dns-query -sS -I -o /dev/null --connect-timeout 5 --max-time 8 "https://httpbin.org/get" 2>&1)
        local doh_exit=$?
        
        # 只检查致命错误，忽略网络问题
        if echo "$doh_test" | grep -qE "(Unknown option|unrecognized option|or 'curl --manual'|not compiled|requires|unsupported)"; then
            print_warning "DoH功能测试: 不支持"
            return 1
        elif [ $doh_exit -eq 0 ]; then
            print_success "DoH功能测试: 完全正常"
            return 0
        else
            print_warning "DoH功能测试: 可能有网络问题，但功能已启用"
            return 0  # 仍然认为DoH可用
        fi
    else
        print_warning "DoH (DNS over HTTPS): 不支持"
        return 1
    fi
}

# 执行单次测试
single_test() {
    local url="$1"
    local dns_option="$2"
    local doh_url="$3"
    
    local curl_cmd="curl -sS -I -L -o /dev/null --connect-timeout $TIMEOUT --max-time $TIMEOUT"
    
    # 添加DNS配置（仅在支持且指定时）
    if [ -n "$dns_option" ] && [ "$DNS_SERVERS_SUPPORTED" = true ]; then
        curl_cmd="$curl_cmd --dns-servers $dns_option"
    elif [ -n "$dns_option" ] && [ "$DNS_SERVERS_SUPPORTED" = false ]; then
        # 不支持 --dns-servers 时，跳过该测试
        echo "SKIP SKIP SKIP SKIP SKIP"
        return 2
    fi
    
    # 添加DoH配置
    if [ -n "$doh_url" ]; then
        curl_cmd="$curl_cmd --doh-url $doh_url"
    fi
    
    # 添加详细的时间测量
    curl_cmd="$curl_cmd -w '%{time_namelookup} %{time_connect} %{time_starttransfer} %{time_total} %{http_code}'"
    
    # 调试模式：显示实际命令
    if [ "$DEBUG" = true ]; then
        echo >&2 "调试: 执行命令 -> $curl_cmd \"$url\""
    fi
    
    # 创建临时文件来分离输出
    local temp_stdout=$(mktemp)
    local temp_stderr=$(mktemp)
    
    # 执行curl命令
    eval "$curl_cmd \"$url\"" >"$temp_stdout" 2>"$temp_stderr"
    local exit_code=$?
    
    # 读取输出
    local stdout_content=$(cat "$temp_stdout" 2>/dev/null)
    local stderr_content=$(cat "$temp_stderr" 2>/dev/null)
    
    # 清理临时文件
    rm -f "$temp_stdout" "$temp_stderr"
    
    # 调试模式：显示结果
    if [ "$DEBUG" = true ]; then
        echo >&2 "调试: 退出代码=$exit_code"
        echo >&2 "调试: stdout='$stdout_content'"
        echo >&2 "调试: stderr='$stderr_content'"
    fi
    
    # 检查curl是否成功执行
    if [ $exit_code -ne 0 ]; then
        if [ "$DEBUG" = true ]; then
            echo >&2 "调试: curl退出失败，代码=$exit_code"
        fi
        echo "ERROR ERROR ERROR ERROR ERROR"
        return 1
    fi
    
    # 检查stderr是否包含错误信息
    if [ -n "$stderr_content" ] && echo "$stderr_content" | grep -qE "(Unknown option|unrecognized option|or 'curl --manual'|error|failed|timeout|refused|resolve|connect|SSL)"; then
        if [ "$DEBUG" = true ]; then
            echo >&2 "调试: stderr包含错误信息: $stderr_content"
        fi
        echo "ERROR ERROR ERROR ERROR ERROR"
        return 1
    fi
    
    # 处理结果（curl的-w输出通常在stderr中）
    local result=""
    if [ -n "$stderr_content" ]; then
        result=$(echo "$stderr_content" | tail -n1 | tr -d '\r\n' | sed 's/[[:space:]]*$//')
    fi
    
    # 如果stderr为空，尝试stdout
    if [ -z "$result" ] && [ -n "$stdout_content" ]; then
        result=$(echo "$stdout_content" | tail -n1 | tr -d '\r\n' | sed 's/[[:space:]]*$//')
    fi
    
    # 验证结果格式（应该是5个数值，用空格分隔）
    if echo "$result" | grep -qE "^[0-9.]+ +[0-9.]+ +[0-9.]+ +[0-9.]+ +[0-9]+$"; then
        # 进一步验证每个字段都是有效数字
        read dns_time connect_time transfer_time total_time http_code <<< "$result"
        
        if [[ "$dns_time" =~ ^[0-9.]+$ ]] && [[ "$connect_time" =~ ^[0-9.]+$ ]] && \
           [[ "$transfer_time" =~ ^[0-9.]+$ ]] && [[ "$total_time" =~ ^[0-9.]+$ ]] && \
           [[ "$http_code" =~ ^[0-9]+$ ]]; then
            echo "$result"
            return 0
        fi
    fi
    
    # 如果到这里，说明结果格式不正确
    if [ "$DEBUG" = true ]; then
        echo >&2 "调试: 结果格式不正确='$result'"
        echo >&2 "调试: 期望格式: 数字 数字 数字 数字 数字"
    fi
    echo "ERROR ERROR ERROR ERROR ERROR"
    return 1
}

# 执行多次测试并统计
run_tests() {
    local url="$1"
    local config_name="$2"
    local dns_option="$3"
    local doh_url="$4"
    
    # 检查是否应该跳过此配置（注意：系统默认的dns_option应该是空字符串）
    if [ -n "$dns_option" ] && [ "$DNS_SERVERS_SUPPORTED" = false ]; then
        printf "${YELLOW}%-25s %12s %12s %12s %12s %10s${NC}\n" \
            "$config_name" "SKIPPED" "SKIPPED" "SKIPPED" "SKIPPED" "N/A"
        return 0
    fi
    
    local dns_times=()
    local connect_times=()
    local transfer_times=()
    local total_times=()
    local success_count=0
    
    echo -n "  测试 $config_name: "
    
    for ((i=1; i<=TEST_COUNT; i++)); do
        echo -n "."
        local result
        result=$(single_test "$url" "$dns_option" "$doh_url")
        local test_result=$?
        
        if [ $test_result -eq 0 ] && [ "$result" != "ERROR ERROR ERROR ERROR ERROR" ]; then
            read dns_time connect_time transfer_time total_time http_code <<< "$result"
            
            # 验证数据完整性并检查HTTP状态码
            if [[ "$dns_time" =~ ^[0-9.]+$ ]] && [[ "$connect_time" =~ ^[0-9.]+$ ]] && \
               [[ "$transfer_time" =~ ^[0-9.]+$ ]] && [[ "$total_time" =~ ^[0-9.]+$ ]] && \
               [[ "$http_code" =~ ^[0-9]+$ ]]; then
                
                # 检查HTTP状态码范围
                if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 400 ]; then
                    dns_times+=($dns_time)
                    connect_times+=($connect_time)
                    transfer_times+=($transfer_time)
                    total_times+=($total_time)
                    ((success_count++))
                fi
            else
                # 数据不完整或格式错误
                if [ "$DEBUG" = true ]; then
                    echo >&2 "调试: 数据格式错误 - DNS:$dns_time, 连接:$connect_time, 传输:$transfer_time, 总计:$total_time, 状态码:$http_code"
                fi
            fi
        elif [ $test_result -eq 2 ]; then
            # SKIP情况已在函数开头处理
            break
        fi
        
        sleep 0.1  # 短暂间隔
    done
    
    echo " 完成"
    
    if [ $success_count -eq 0 ]; then
        printf "${RED}%-25s %12s %12s %12s %12s %10s${NC}\n" \
            "$config_name" "FAILED" "FAILED" "FAILED" "FAILED" "0%"
        return 1
    fi
    
    # 计算统计值
    local dns_stats=($(calculate_stats "${dns_times[@]}"))
    local connect_stats=($(calculate_stats "${connect_times[@]}"))
    local transfer_stats=($(calculate_stats "${transfer_times[@]}"))
    local total_stats=($(calculate_stats "${total_times[@]}"))
    
    # 计算成功率
    local success_rate=0
    if [ $TEST_COUNT -gt 0 ]; then
        success_rate=$(math_calc "$success_count * 100 / $TEST_COUNT")
    fi
    
    # 根据延迟给出颜色
    local total_avg=${total_stats[0]:-0}
    local color=$GREEN
    if float_compare "$total_avg" ">" "1.0"; then
        color=$YELLOW
    fi
    if float_compare "$total_avg" ">" "3.0"; then
        color=$RED
    fi
    
    # 如果成功率太低，使用红色
    if float_compare "$success_rate" "<" "80"; then
        color=$RED
    fi
    
    printf "${color}%-25s %11.3fms %11.3fms %11.3fms %11.3fms %9.1f%%${NC}\n" \
        "$config_name" \
        $(math_calc "${dns_stats[0]:-0} * 1000") \
        $(math_calc "${connect_stats[0]:-0} * 1000") \
        $(math_calc "${transfer_stats[0]:-0} * 1000") \
        $(math_calc "${total_stats[0]:-0} * 1000") \
        "$success_rate"
    
    # 保存详细统计（可选）
    if [ "$VERBOSE" = true ]; then
        echo "    DNS解析: 平均${dns_stats[0]:-0}s, 最小${dns_stats[1]:-0}s, 最大${dns_stats[2]:-0}s, 标准差${dns_stats[3]:-0}s"
        echo "    连接建立: 平均${connect_stats[0]:-0}s, 最小${connect_stats[1]:-0}s, 最大${connect_stats[2]:-0}s, 标准差${connect_stats[3]:-0}s"
        echo "    首字节: 平均${transfer_stats[0]:-0}s, 最小${transfer_stats[1]:-0}s, 最大${transfer_stats[2]:-0}s, 标准差${transfer_stats[3]:-0}s"
        echo "    总时间: 平均${total_stats[0]:-0}s, 最小${total_stats[1]:-0}s, 最大${total_stats[2]:-0}s, 标准差${total_stats[3]:-0}s"
        echo
    fi
}

# 检查依赖
check_dependencies() {
    local missing_deps=()
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    # 检测数学计算工具
    if ! detect_math_tool; then
        missing_deps+=("bc 或 awk")
        print_error "未找到数学计算工具"
        echo "请安装以下任一工具："
        echo "  • bc (推荐): sudo apt install bc"
        echo "  • awk: 通常系统自带，如无请安装 gawk"
    else
        case "$MATH_TOOL" in
            "bc")
                print_success "数学计算工具: bc (推荐)"
                ;;
            "awk")
                print_warning "数学计算工具: awk (备选方案)"
                print_warning "建议安装 bc 以获得更好的精度: sudo apt install bc"
                ;;
        esac
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "缺少依赖: ${missing_deps[*]}"
        exit 1
    fi
}

# 检查HTTP响应是否成功
check_http_success() {
    local url="$1"
    local timeout="${2:-5}"
    
    # 获取HTTP状态码
    local result
    result=$(curl -sS -L -o /dev/null -w "%{http_code}" --connect-timeout "$timeout" --max-time "$((timeout*2))" "$url" 2>&1)
    local curl_exit=$?
    
    # 检查curl是否成功执行
    if [ $curl_exit -ne 0 ]; then
        return 1
    fi
    
    # 检查结果是否为纯数字（HTTP状态码）
    if [[ "$result" =~ ^[0-9]+$ ]]; then
        local http_code="$result"
        # curl执行成功且HTTP状态码在200-399范围内
        if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 400 ]; then
            return 0
        elif [ "$http_code" -ge 400 ]; then
            # 连接成功但服务器返回错误（如404, 500等）
            return 2
        else
            return 1
        fi
    else
        # 结果包含错误信息而不是纯数字
        return 1
    fi
}

# 快速网络诊断
quick_diagnosis() {
    echo -e "${CYAN}=== 快速网络诊断 ===${NC}"
    
    # 测试基本HTTP连接
    echo -n "测试基本HTTP连接: "
    if check_http_success "http://httpbin.org/get" 5; then
        echo -e "${GREEN}成功${NC}"
    else
        echo -e "${RED}失败${NC}"
        echo -e "${RED}基本网络连接有问题，请检查网络设置${NC}"
        return 1
    fi
    
    # 测试HTTPS连接
    echo -n "测试HTTPS连接: "
    local https_result
    https_result=$(check_http_success "https://httpbin.org/get" 5; echo $?)
    case $https_result in
        0)
            echo -e "${GREEN}成功${NC}"
            ;;
        2)
            echo -e "${YELLOW}连接成功但服务器返回错误${NC}"
            ;;
        *)
            echo -e "${RED}失败${NC}"
            echo -e "${RED}HTTPS连接有问题，可能是证书或网络问题${NC}"
            ;;
    esac
    
    # 测试DNS解析
    echo -n "测试DNS解析: "
    local dns_success=false
    local test_domains=("www.baidu.com" "httpbin.org" "www.bing.com")
    local success_count=0
    
    for domain in "${test_domains[@]}"; do
        if check_http_success "https://$domain/" 3; then
            ((success_count++))
            dns_success=true
        fi
    done
    
    if [ "$dns_success" = true ]; then
        echo -e "${GREEN}成功 ($success_count/${#test_domains[@]} 域名可访问)${NC}"
    else
        echo -e "${RED}失败${NC}"
        echo -e "${RED}DNS解析存在问题${NC}"
    fi
    
    echo
}

# 主函数
main() {
    print_header
    
    # 检查依赖
    check_dependencies
    
    # 快速网络诊断
    quick_diagnosis
    
    # 特别测试as.target的基本连通性
    echo -e "${CYAN}=== 测试主要目标 as.target ===${NC}"
    echo -n "基本连通性: "
    if check_http_success "https://as.target" 8; then
        echo -e "${GREEN}成功 ✓${NC}"
    else
        echo -e "${RED}失败 ✗${NC}"
        echo -e "${YELLOW}警告: as.target 可能暂时无法访问，测试结果可能受影响${NC}"
    fi
    echo
    
    # 如果指定了自定义URL，只测试该URL
    if [ -n "$CUSTOM_URL" ]; then
        echo -e "${CYAN}=== 测试自定义URL ===${NC}"
        echo "测试URL: $CUSTOM_URL"
        # 这里可以添加自定义URL的测试逻辑
        return 0
    fi
    
    # 检查curl功能
    local doh_supported
    check_curl_features
    doh_supported=$?
    
    # 显示使用的数学工具
    echo "数学计算工具: $MATH_TOOL"
    
    # 显示DNS测试提示
    if [ "$DNS_SERVERS_SUPPORTED" = false ]; then
        echo
        print_warning "由于curl不支持 --dns-servers 参数，将跳过自定义DNS服务器测试"
        print_warning "只测试系统默认DNS和DoH配置"
        echo
    fi
    echo
    
    # 测试每个URL
    for url in "${urls[@]}"; do
        print_section "测试 URL: $url"
        echo "每个配置测试 $TEST_COUNT 次，计算平均值..."
        echo
        
        # 打印表头
        printf "${WHITE}%-25s %12s %12s %12s %12s %10s${NC}\n" \
            "DNS配置" "DNS解析" "连接建立" "首字节" "总时间" "成功率"
        printf "%-25s %12s %12s %12s %12s %10s\n" \
            "-------------------------" "------------" "------------" "------------" "------------" "----------"
        
        # 测试传统DNS配置
        for config_name in "${!dns_configs[@]}"; do
            run_tests "$url" "$config_name" "${dns_configs[$config_name]}" ""
        done
        
        # 测试DoH配置（如果支持）
        if [ $doh_supported -eq 0 ]; then
            echo
            printf "${PURPLE}%-25s %12s %12s %12s %12s %10s${NC}\n" \
                "DoH配置" "DNS解析" "连接建立" "首字节" "总时间" "成功率"
            printf "%-25s %12s %12s %12s %12s %10s\n" \
                "-------------------------" "------------" "------------" "------------" "------------" "----------"
            
            for config_name in "${!doh_configs[@]}"; do
                run_tests "$url" "$config_name" "" "${doh_configs[$config_name]}"
            done
        fi
        
        echo
        echo "---"
        echo
    done
    
    # 输出说明
    echo -e "${WHITE}说明:${NC}"
    echo "• DNS解析: 域名解析耗时"
    echo "• 连接建立: TCP连接建立耗时"  
    echo "• 首字节: 从请求发送到接收首字节的耗时"
    echo "• 总时间: 完整请求的总耗时"
    echo "• 成功率: 成功请求的百分比"
    echo
    echo -e "${WHITE}状态说明:${NC}"
    echo -e "${GREEN}绿色${NC}: 延迟良好 (<1s)"
    echo -e "${YELLOW}黄色${NC}: 延迟一般 (1-3s) 或 SKIPPED (功能不支持)"
    echo -e "${RED}红色${NC}: 延迟较高 (>3s) 或 FAILED (连接失败)"
    echo -e "${PURPLE}紫色${NC}: DoH配置"
    echo
    
    if [ "$DNS_SERVERS_SUPPORTED" = false ]; then
        echo -e "${WHITE}注意事项:${NC}"
        echo "• 自定义DNS服务器测试被跳过，因为您的curl版本不支持 --dns-servers 参数"
        echo "• 当前配置的DNS服务器: 腾讯DNS, 阿里DNS"
        echo "• 要启用完整DNS测试功能，请升级curl版本或重新编译带有c-ares支持的curl"
        echo
    fi
    
    echo -e "${WHITE}💡 关于测试配置:${NC}"
    echo "• as.target - Claude Relay Service (Claude API中继服务)"
    echo "• 系统默认 - 使用系统配置的DNS服务器"
    echo "• 腾讯DNS (120.53.53.102) - 腾讯云DNS服务器"
    echo "• 阿里DNS (223.5.5.5) - 阿里云DNS服务器"
    echo "• Aliyun DoH - 阿里云的DNS over HTTPS服务"  
    echo "• NovaXNS - 专业的DoH服务提供商"
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--debug)
            DEBUG=true
            shift
            ;;
        -c|--count)
            TEST_COUNT="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -u|--url)
            CUSTOM_URL="$2"
            shift 2
            ;;
        --test-original)
            TEST_ORIGINAL=true
            shift
            ;;
        --no-color)
            ENABLE_COLOR=false
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  -v, --verbose     显示详细统计信息"
            echo "  -c, --count N     每个配置的测试次数 (默认: 5)"
            echo "  -t, --timeout N   超时时间，秒 (默认: 10)"
            echo "  --no-color        禁用彩色输出"
            echo "  -d, --debug       显示详细调试信息"
            echo "  -u, --url URL     测试指定URL"
            echo "  --test-original   测试主要目标 as.target"
            echo "  -h, --help        显示此帮助信息"
            exit 0
            ;;
        *)
            print_error "未知选项: $1"
            exit 1
            ;;
    esac
done

# 执行主函数
main