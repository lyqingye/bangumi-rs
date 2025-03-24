use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // 指示cargo在源文件改变时重新运行此脚本
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // 获取编译模式（debug或release）
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // 初始化RUSTFLAGS
    let mut rustflags = String::new();

    // 获取cargo manifest路径
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取CARGO_MANIFEST_DIR");
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");

    // 读取和解析Cargo.toml中的rustc-args
    if let Ok(cargo_toml_content) = fs::read_to_string(cargo_toml_path) {
        if let Ok(cargo_toml) = toml::from_str::<toml::Value>(&cargo_toml_content) {
            // 获取rustc参数
            if let Some(metadata) = cargo_toml.get("package").and_then(|p| p.get("metadata")) {
                if let Some(rustc_args) = metadata.get("rustc-args") {
                    let profile_key = if profile == "release" {
                        "release"
                    } else {
                        "dev"
                    };

                    if let Some(args) = rustc_args.get(profile_key).and_then(|a| a.as_array()) {
                        // 收集所有参数
                        let mut rustc_flags_vec = Vec::new();
                        for arg in args {
                            if let Some(arg_str) = arg.as_str() {
                                rustc_flags_vec.push(arg_str);
                            }
                        }

                        // 组合所有参数
                        rustflags = rustc_flags_vec.join(" ");
                    }
                }
            }
        }
    }

    // 添加默认的CPU架构优化
    if !rustflags.contains("target-cpu=") {
        if !rustflags.is_empty() {
            rustflags.push(' ');
        }
        rustflags.push_str("-C target-cpu=native");
    }

    // 在release模式下应用PGO（如果支持）
    if profile == "release" {
        if let Some(pgo_flags) = attempt_pgo_optimization() {
            if !rustflags.is_empty() {
                rustflags.push(' ');
            }
            rustflags.push_str(&pgo_flags);
        }
    }

    // 设置最终的RUSTFLAGS（只设置一次）
    if !rustflags.is_empty() {
        println!("cargo:rustc-env=RUSTFLAGS={}", rustflags);
    }
}

// 尝试应用PGO优化(Profile Guided Optimization)，返回PGO标志而不是直接设置RUSTFLAGS
fn attempt_pgo_optimization() -> Option<String> {
    // 检查是否支持PGO
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());

    if let Some(version) = rustc_version {
        if version.contains("nightly") {
            println!("cargo:warning=检测到nightly编译器，尝试启用PGO优化");

            // 返回PGO标志而不是直接设置RUSTFLAGS
            let pgo_flags = "-Cprofile-generate=./pgo-data";
            println!(
                "cargo:warning=已配置PGO生成模式。编译并运行程序以生成性能分析数据，然后重新编译以使用这些数据。"
            );

            return Some(pgo_flags.to_string());
        } else {
            println!("cargo:warning=未检测到nightly编译器，跳过PGO优化");
        }
    }

    None
}
