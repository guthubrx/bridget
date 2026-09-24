#[path = "src/build_identity.rs"]
mod build_identity;

fn main() {
    println!("cargo:rerun-if-env-changed=BRIDGET_BUILD_ID");
    // Session 116 : le chemin est lu à l'EXÉCUTION du script, pas à sa
    // compilation. `env!` figeait celui du premier arbre qui avait compilé ce
    // script ; réutilisé tel quel par un autre worktree partageant le même
    // répertoire de compilation, il interrogeait le Git de ce premier arbre.
    // L'identifiant de build est ainsi resté celui de la session 105 du
    // 17/09, et l'alerte de daemon périmé ne pouvait plus se déclencher.
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("fourni par Cargo");
    let root = std::path::Path::new(&manifest_dir).join("../..");
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
