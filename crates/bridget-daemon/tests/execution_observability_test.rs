use bridget_daemon::{
    daemon::Metrics,
    ui::{UiAlertThresholdsV1, execution_alerts},
};
use std::sync::atomic::Ordering;

#[test]
fn metriques_execution_sont_compteurs_bornes_et_sans_corpus() {
    let metrics = Metrics::new();
    metrics.record_execution_admitted();
    metrics.record_execution_started(12);
    metrics.record_execution_transition_rejected();
    metrics.record_execution_delivery_failure();
    metrics.record_execution_queue_saturated();

    assert_eq!(metrics.execution_admitted.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.execution_started.load(Ordering::Relaxed), 1);
    assert_eq!(
        metrics
            .execution_start_latency_samples
            .load(Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics
            .execution_start_latency_secs_total
            .load(Ordering::Relaxed),
        12
    );
    assert_eq!(
        metrics
            .execution_transition_rejected
            .load(Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics.execution_delivery_failures.load(Ordering::Relaxed),
        1
    );
    assert_eq!(metrics.execution_queue_saturated.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.errors.load(Ordering::Relaxed), 2);

    let source = include_str!("../src/daemon.rs");
    let definition = source
        .split("pub struct Metrics")
        .nth(1)
        .and_then(|rest| rest.split("impl Default for Metrics").next())
        .expect("definition Metrics");
    assert!(!definition.contains("HashMap"));
    assert!(!definition.contains("String"));
}

#[test]
fn alertes_couvrent_vieillissement_sans_progres_autorisation_et_saturation() {
    let thresholds = UiAlertThresholdsV1::default();
    assert_eq!(thresholds.stale_message_secs, 15 * 60);
    assert_eq!(thresholds.stalled_turn_secs, 5 * 60);
    assert_eq!(thresholds.stalled_approval_secs, 5 * 60);
    assert_eq!(thresholds.saturated_queue_depth, 10);

    let alerts = execution_alerts(
        Some("running"),
        Some("waiting_approval"),
        Some(thresholds.stalled_turn_secs),
        thresholds.saturated_queue_depth,
        Some(thresholds.stale_message_secs),
    );
    assert_eq!(
        alerts,
        vec![
            "stale_message",
            "stalled_turn",
            "stalled_approval",
            "queue_saturated",
        ]
    );
}
