//! Compatibilité des données conservées, séparée des versions des fournisseurs.
//! Le lecteur installé impose son plancher à tout successeur, même sans scan des
//! stores : une racine custom oubliée ne doit jamais être assimilée à « vide ».
use crate::cli_table::table_println as println;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;

const WORKTREE: &str = "workspace.worktree.v1";
const COMMIT_VIEW:&str="workspace.commit_view.v1";
const COMMIT_VIEW_ORIGIN_ROOT:&str="workspace.commit_view.origin_root.v2";
// Le lecteur comprend préparation partielle, fichier en cours, alias198 et
// conservation compacte, même lorsque la création publique reste fermée.
// Les contre-épreuves opérationnelles sont dans remote_worker::tests et
// workspace_commit_view::tests ; ce plancher ne dépend pas du drapeau créateur.
// Le reçu garde son schéma, mais le lecteur doit aussi savoir re-projeter
// un parent en checkout lié vers la racine de A lors d'une lecture omise.
// Le lecteur A accepte le JSON V1 mais ne sait pas effectuer cette relecture.
// Le choix figure dans AutoV2, Task et Prepare et doit survivre aux reprises.
/// Accepter le JSON sans appliquer ce choix n'est pas un lecteur compatible.
const MODEL_OPTIONS:&str="delegation.model_options.v1";
const PROVIDER_ACCOUNTS: &str = "launch.provider_account.v1";
const WORKER_CAPACITY: &str = "worker.store.capacity.v2";
const SPECIALIST_REQUESTS: &str = "specialists.requests.v1";
// Les prepared222 ne peuvent être remis que par leur autorité épinglée.
// Un ancien drain qui ignore cette garde ne constitue pas un lecteur compatible.
const FORMATS: &[&str] = &[SPECIALIST_REQUESTS,WORKER_CAPACITY,WORKTREE,COMMIT_VIEW,COMMIT_VIEW_ORIGIN_ROOT,MODEL_OPTIONS,PROVIDER_ACCOUNTS];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    version: u8,
    nonce: String,
    image_sha256: String,
    reads: Vec<String>,
    writes: Vec<String>,
}
fn hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn identifiers(values: &[String], empty: bool) -> bool {
    (empty || !values.is_empty())
        && values.len() <= 16
        && values.iter().all(|s| {
            !s.is_empty()
                && s.len() <= 64
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._".contains(&b))
        })
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
fn validate(bytes: &[u8], nonce: &str, image: &str) -> bool {
    if bytes.len() > 8192 || !hex(nonce, 32) || !hex(image, 64) {
        return false;
    }
    let Ok(report) = serde_json::from_slice::<Report>(bytes) else {
        return false;
    };
    report.version == 1
        && report.nonce == nonce
        && report.image_sha256 == image
        && identifiers(&report.reads, false)
        && identifiers(&report.writes, true)
        && FORMATS.iter().all(|f| report.reads.iter().any(|s| s == f))
        && report.writes.iter().all(|f| report.reads.contains(f))
}
fn report(nonce: &str) -> Result<Report, String> {
    if !hex(nonce, 32) {
        return Err("storage_compatibility_arguments_invalid".into());
    }
    let path = std::env::current_exe().map_err(|_| "storage_image_unavailable")?;
    let mut file = std::fs::File::open(path).map_err(|_| "storage_image_unavailable")?;
    let metadata = file.metadata().map_err(|_| "storage_image_unavailable")?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 {
        return Err("storage_image_unavailable".into());
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        if std::time::Instant::now() >= deadline {
            return Err("storage_image_unavailable".into());
        }
        let n = file
            .read(&mut buffer)
            .map_err(|_| "storage_image_unavailable")?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(Report {
        version: 1,
        nonce: nonce.into(),
        image_sha256: format!("{:x}", hash.finalize()),
        reads: FORMATS.iter().map(|f| (*f).into()).collect(),
        writes: FORMATS.iter().map(|f| (*f).into()).collect(),
    })
}
fn execute(args: &[String]) -> Result<serde_json::Value, String> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["report", "--nonce", nonce] => {
            serde_json::to_value(report(nonce)?).map_err(|_| "storage_report_unavailable".into())
        }
        ["validate-report", "--nonce", nonce, "--sha256", image] => {
            let mut bytes = Vec::new();
            std::io::stdin()
                .lock()
                .take(8193)
                .read_to_end(&mut bytes)
                .map_err(|_| "storage_report_unavailable")?;
            if !validate(&bytes, nonce, image) {
                return Err("storage_reader_incompatible".into());
            }
            Ok(serde_json::json!({"valid":true}))
        }
        ["policy", "--json"] => Ok(serde_json::json!({"version":1,"required_reads":FORMATS,
            "scope":"all_formats_this_controller_can_persist","inventory_scanned":false})),
        _ => Err("storage_compatibility_arguments_invalid".into()),
    }
}
pub(crate) fn run(args: &[String]) -> ! {
    match execute(args) {
        Ok(value) => {
            println!("{value}");
            crate::cli_table::exit(0)
        }
        Err(code) => {
            println!(
                "{}",
                serde_json::json!({"status":"refused","code":code,
                "message":"Cette vérification de lecture des données n’a pas abouti. Aucun service n’a été arrêté.",
                "recovery":"Conserver le binaire actif, les tâches et leurs reçus. Faire vérifier le candidat et son rapport de formats par le responsable Bridget avant une bascule ou un retour. Si le candidat ne sait pas lire ces formats, demander une version compatible ; ne pas effacer les données pour forcer le passage. Ce refus concerne ce candidat. Les autres travaux autorisés suivent leur propre état."})
            );
            crate::cli_table::exit(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::json!({"version":1,"nonce":"a".repeat(32),"image_sha256":"b".repeat(64),
            "reads":FORMATS,"writes":FORMATS})
    }
    #[test]
    fn spec221_c19_origin_root_floor_refuses_reader_a_despite_unchanged_json_schema() {
        let mut old=fixture();
        old["reads"]=serde_json::json!([WORKTREE,COMMIT_VIEW]);
        old["writes"]=old["reads"].clone();
        assert!(!validate(&serde_json::to_vec(&old).unwrap(),&"a".repeat(32),&"b".repeat(64)));
        assert!(validate(&serde_json::to_vec(&fixture()).unwrap(),&"a".repeat(32),&"b".repeat(64)));
    }

    #[test]
    fn spec222_storage_floor_refuses_reader_without_specialist_prepared_authority() {
        let mut old = fixture();
        old["reads"].as_array_mut().unwrap().retain(|v|v != SPECIALIST_REQUESTS);
        old["writes"] = old["reads"].clone();
        assert!(!validate(&serde_json::to_vec(&old).unwrap(), &"a".repeat(32), &"b".repeat(64)));
        assert!(validate(&serde_json::to_vec(&fixture()).unwrap(), &"a".repeat(32), &"b".repeat(64)));
    }

    #[test]
    fn spec221_result_capacity_floor_refuses_legacy_16mib_reader() {
        let mut old=fixture();
        old["reads"].as_array_mut().unwrap().retain(|v|v!=WORKER_CAPACITY);
        old["writes"]=old["reads"].clone();
        assert!(!validate(&serde_json::to_vec(&old).unwrap(),&"a".repeat(32),&"b".repeat(64)));
        assert!(validate(&serde_json::to_vec(&fixture()).unwrap(),&"a".repeat(32),&"b".repeat(64)));
    }

    #[test]
    fn spec221_c05_storage_floor_refuses_a_reader_that_would_ignore_the_choice() {
        let mut old=fixture();
        old["reads"]=serde_json::json!([WORKTREE,COMMIT_VIEW,COMMIT_VIEW_ORIGIN_ROOT]);
        old["writes"]=old["reads"].clone();
        assert!(!validate(&serde_json::to_vec(&old).unwrap(),&"a".repeat(32),&"b".repeat(64)));
        assert!(validate(&serde_json::to_vec(&fixture()).unwrap(),&"a".repeat(32),&"b".repeat(64)));
    }

    #[test]
    fn spec221_storage_reader_report_is_bound_to_nonce_image_and_exact_shape() {
        let original = fixture();
        let nonce = "a".repeat(32);
        let image = "b".repeat(64);
        assert!(validate(
            &serde_json::to_vec(&original).unwrap(),
            &nonce,
            &image
        ));
        for (key, bad) in [
            ("version", serde_json::json!(2)),
            ("nonce", serde_json::json!("c".repeat(32))),
            ("image_sha256", serde_json::json!("c".repeat(64))),
            ("reads", serde_json::json!([])),
            ("reads", serde_json::json!([WORKTREE, WORKTREE])),
            ("reads", serde_json::json!(["other.format"])),
            ("writes", serde_json::json!(["future.format"])),
            ("private", serde_json::json!("PRIVATE")),
        ] {
            let mut altered = original.clone();
            altered[key] = bad;
            assert!(
                !validate(&serde_json::to_vec(&altered).unwrap(), &nonce, &image),
                "{key}"
            );
        }
        assert!(!validate(&vec![b' '; 8193], &nonce, &image));
        // Une déclaration additive ne doit pas casser un ancien validateur
        // si elle conserve les formats qu'il exige.
        let mut future = original.clone();
        future["reads"].as_array_mut().unwrap().push(serde_json::json!("future.format"));
        assert!(validate(
            &serde_json::to_vec(&future).unwrap(),
            &nonce,
            &image
        ));
    }
    #[test]
    fn spec221_storage_floor_keeps_commit_view_for_reader_even_without_creator() {
        use bridget_transport::protocol::WorkspaceRequest;
        assert!(
            serde_json::from_value::<WorkspaceRequest>(serde_json::json!({"strategy":"worktree"}))
                .is_ok()
        );
        assert!(FORMATS.contains(&COMMIT_VIEW));
        // Le plancher ne dépend pas du créateur : le jalon lecteur seul a
        // exercé ce même contrat avant l'ouverture du candidat de création.
        let old=serde_json::json!({"version":1,"nonce":"a".repeat(32),"image_sha256":"b".repeat(64),
            "reads":[WORKTREE],"writes":[WORKTREE]});
        assert!(!validate(&serde_json::to_vec(&old).unwrap(),&"a".repeat(32),&"b".repeat(64)));
        assert!(
            serde_json::from_value::<WorkspaceRequest>(
                serde_json::json!({"strategy":"commit_view"})
            )
            .is_ok()
        );
        let value = execute(&["policy".into(), "--json".into()]).unwrap();
        assert_eq!(value["inventory_scanned"], false);
        assert_eq!(value["required_reads"], serde_json::json!([WORKTREE,COMMIT_VIEW,COMMIT_VIEW_ORIGIN_ROOT,MODEL_OPTIONS,PROVIDER_ACCOUNTS]));
    }
}
