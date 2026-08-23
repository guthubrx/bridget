#[path = "src/build_identity.rs"]
mod build_identity;

fn main() {
    println!("cargo:rerun-if-env-changed=BRIDGET_BUILD_ID");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in build_identity::invalidation_paths(&root) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let build_id = std::env::var("BRIDGET_BUILD_ID")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| build_identity::current_build_id(&root))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BRIDGET_BUILD_ID={build_id}");
}
