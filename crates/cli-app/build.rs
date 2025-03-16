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

    // 获取cargo manifest路径
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取CARGO_MANIFEST_DIR");
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");

    // 读取和解析Cargo.toml
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
                        let mut rustc_flags = Vec::new();
                        for arg in args {
                            if let Some(arg_str) = arg.as_str() {
                                rustc_flags.push(arg_str);
                            }
                        }

                        // 组合所有参数并将其设置为RUSTFLAGS环境变量
                        let rustflags = rustc_flags.join(" ");
                        println!("cargo:rustc-env=RUSTFLAGS={}", rustflags);

                        // 打印使用的标志，便于调试
                        println!("cargo:warning=使用编译标志: {}", rustflags);
                    }
                }
            }
        }
    }

    // 在release模式下应用PGO（如果支持）
    if profile == "release" {
        attempt_pgo_optimization();
    }

    // 设置RUSTFLAGS以使用jemalloc
    println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=native");
}

// 尝试应用PGO优化(Profile Guided Optimization)
fn attempt_pgo_optimization() {
    // 检查是否支持PGO
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());

    if let Some(version) = rustc_version {
        if version.contains("nightly") {
            println!("cargo:warning=检测到nightly编译器，尝试启用PGO优化");

            // 启用PGO
            println!("cargo:rustc-env=RUSTFLAGS=-Cprofile-generate=./pgo-data");
            println!("cargo:warning=已配置PGO生成模式。编译并运行程序以生成性能分析数据，然后重新编译以使用这些数据。");
        } else {
            println!("cargo:warning=未检测到nightly编译器，跳过PGO优化");
        }
    }
}
