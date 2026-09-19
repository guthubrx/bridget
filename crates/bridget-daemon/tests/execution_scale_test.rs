use bridget_daemon::ExecutionStore;
use std::time::{Duration, Instant};

/// Seuil de régression volontairement large: ce témoin protège la forme O(n)
/// de la projection sur un volume fixe, sans transformer un test CI en microbench instable.
#[test]
fn projection_execution_reste_bornee_sur_256_agents() {
    let store = ExecutionStore::open_in_memory().expect("magasin mémoire");
    let started = Instant::now();
    for index in 0..256 {
        store
            .record_starting(
                &format!("submission-{index}"),
                &format!("execution-{index}"),
                &format!("agent-{index}"),
                1_000 + index as i64,
            )
            .expect("démarrage");
    }
    let summaries = store.agent_execution_summaries().expect("projection");
    let elapsed = started.elapsed();

    assert_eq!(summaries.len(), 256);
    assert!(
        elapsed < Duration::from_secs(3),
        "projection 256 agents trop lente: {elapsed:?}"
    );
}
