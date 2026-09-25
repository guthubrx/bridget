//! Points de synchronisation réservés aux bancs de crash inter-processus.
//!
//! Ce module n'est compilé qu'avec la feature `test-support`. Sans le
//! répertoire explicitement fourni par le banc, chaque point est un no-op.

use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

pub const DIRECTORY_ENV: &str = "BRIDGET_TEST_SYNC_DIR";

/// Publie un jalon, puis attend que le banc ouvre la FIFO correspondante.
///
/// Le banc tue le processus dès l'observation du jalon. Le seul lecteur de la
/// FIFO est ce processus ; une absence de FIFO désactive donc ce point sans
/// modifier le comportement du daemon.
pub fn checkpoint(point: &str) {
    let Some(root) = std::env::var_os(DIRECTORY_ENV) else {
        return;
    };
    let root = PathBuf::from(root);
    if fs::write(root.join(format!("{point}.ready")), b"ready\n").is_err() {
        return;
    }
    let Ok(mut gate) = File::open(root.join(format!("{point}.fifo"))) else {
        return;
    };
    let mut token = [0_u8; 1];
    let _ = gate.read(&mut token);
}
