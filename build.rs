use std::env;


fn main() {
    // 输出基本构建信息，不依赖vergen
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", chrono::Utc::now().to_rfc3339());
    println!("cargo:rustc-env=PROJECT_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=PROJECT_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=PROJECT_AUTHORS={}", env!("CARGO_PKG_AUTHORS"));
    println!("cargo:rustc-env=PROJECT_DESCRIPTION={}", env!("CARGO_PKG_DESCRIPTION"));
    println!("cargo:rustc-env=PROJECT_HOMEPAGE={}", env!("CARGO_PKG_HOMEPAGE", ""));
}
