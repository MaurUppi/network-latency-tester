use std::process::Command;
use std::env;

fn main() {
    // Set build-time environment variables
    set_build_info();
    
    // Set platform-specific configurations
    set_platform_config();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

fn set_build_info() {
    // Get build timestamp
    let build_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    
    // Get git commit hash if available
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output() 
    {
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_COMMIT={}", commit);
        }
    }
    
    // Get git branch if available
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_BRANCH={}", branch);
        }
    }
    
    // Set version information
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=VERSION={}", version);
    
    // Set target triple
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=TARGET_TRIPLE={}", target);
}

fn set_platform_config() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    match target_os.as_str() {
        "windows" => {
            // Windows-specific configurations
            println!("cargo:rustc-cfg=windows_platform");
            
            // Link Windows networking libraries
            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=winmm");
        },
        "macos" => {
            // macOS-specific configurations
            println!("cargo:rustc-cfg=macos_platform");
            
            // Link macOS system frameworks
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=SystemConfiguration");
        },
        "linux" => {
            // Linux-specific configurations
            println!("cargo:rustc-cfg=linux_platform");
            
            // Link Linux networking libraries
            println!("cargo:rustc-link-lib=dl");
        },
        _ => {
            println!("cargo:rustc-cfg=unix_platform");
        }
    }
    
    match target_arch.as_str() {
        "x86_64" => println!("cargo:rustc-cfg=arch_x86_64"),
        "aarch64" => println!("cargo:rustc-cfg=arch_aarch64"),
        _ => {}
    }
    
    // Set optimization flags for release builds
    if env::var("PROFILE").unwrap_or_default() == "release" {
        println!("cargo:rustc-cfg=release_build");
    }
}

// Helper function to check if a command exists
#[allow(dead_code)]
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}