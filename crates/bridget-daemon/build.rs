use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=BRIDGET_BUILD_ID");
    if let Some(head) = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-path", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
    {
        println!("cargo:rerun-if-changed={}", head.trim());
    }
    let build_id = std::env::var("BRIDGET_BUILD_ID").ok().filter(|value| !value.is_empty()).or_else(|| {
        Command::new("git")
            .args(["rev-parse", "--short=12", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }).unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BRIDGET_BUILD_ID={build_id}");
}
