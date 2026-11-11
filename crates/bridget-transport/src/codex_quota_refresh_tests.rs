use super::*;

fn session(fail_refresh: bool) -> (CodexAppServerTransport, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("bridget-quota261-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let script = r#"
import json, os, sys
reads = 0
for line in sys.stdin:
    request = json.loads(line)
    method = request.get('method')
    with open(os.environ['QUOTA_TRACE'], 'a') as trace:
        trace.write(method + '\n')
    if 'id' not in request:
        continue
    error = None
    if method == 'initialize':
        result = {'userAgent':'fake','codexHome':'/tmp','platformFamily':'unix','platformOs':'linux'}
    elif method == 'thread/start':
        result = {'thread':{'id':'quota-native'},'model':'gpt-6.1-sol','reasoningEffort':'high'}
    elif method == 'account/rateLimits/read':
        reads += 1
        if reads > 1 and os.environ['QUOTA_FAIL'] == '1':
            error = {'code':-32000,'message':'unavailable'}
        used = 100 if reads != 2 else 0
        result = {'rateLimits':{'primary':{'usedPercent':used,'windowDurationMins':10080,'resetsAt':2000000000}}}
    else:
        result = {}
    response = {'id':request['id']}
    response['error' if error else 'result'] = error if error else result
    print(json.dumps(response), flush=True)
"#;
    let options = CodexAppServerOptions {
        command: "/usr/bin/python3".into(),
        args: vec!["-u".into(), "-c".into(), script.into()],
        queue_capacity: 8,
        notify_timeout_secs: 60,
        model: None,
        permissions: "deny".into(),
        child_requests: false,
        provider_observation: None,
        thread_bootstrap: CodexThreadBootstrap::Start,
        dynamic_tool_handler: None,
    };
    let transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "QUOTA_TRACE".into(),
                root.join("methods").to_string_lossy().into_owned(),
            ),
            (
                "QUOTA_FAIL".into(),
                if fail_refresh { "1" } else { "0" }.into(),
            ),
        ],
        false,
    )
    .unwrap();
    (transport, root)
}

fn quota(events: &[ManagedEvent]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            ManagedEventKind::RateLimitObserved { used_percent, .. } => *used_percent,
            _ => None,
        })
        .collect()
}

fn due(transport: &CodexAppServerTransport) {
    *transport.quota_read_at.lock().unwrap() = Instant::now() - RATE_LIMIT_REFRESH_INTERVAL;
}

#[test]
fn spec261_idle_quota_reads_reset_on_same_process_without_turn() {
    let (transport, root) = session(false);
    assert_eq!(quota(&transport.drain_events()), vec![100]);
    let pid = transport.process_id();
    due(&transport);
    assert_eq!(quota(&transport.drain_events()), vec![0]);
    assert!(quota(&transport.drain_events()).is_empty());
    due(&transport);
    transport.busy.store(true, Ordering::SeqCst);
    assert!(quota(&transport.drain_events()).is_empty());
    transport.busy.store(false, Ordering::SeqCst);
    assert_eq!(quota(&transport.drain_events()), vec![100]);
    assert_eq!(transport.process_id(), pid);
    transport.stop();
    let methods = std::fs::read_to_string(root.join("methods")).unwrap();
    assert_eq!(
        methods
            .lines()
            .filter(|m| *m == "account/rateLimits/read")
            .count(),
        3
    );
    assert_eq!(methods.lines().filter(|m| *m == "thread/start").count(), 1);
    assert!(!methods.contains("turn/start"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec261_failed_refresh_never_invents_available_quota_or_retries_immediately() {
    let (transport, root) = session(true);
    assert_eq!(quota(&transport.drain_events()), vec![100]);
    due(&transport);
    assert!(quota(&transport.drain_events()).is_empty());
    assert!(quota(&transport.drain_events()).is_empty());
    transport.stop();
    let methods = std::fs::read_to_string(root.join("methods")).unwrap();
    assert_eq!(
        methods
            .lines()
            .filter(|m| *m == "account/rateLimits/read")
            .count(),
        2
    );
    assert!(!methods.contains("turn/start"));
    std::fs::remove_dir_all(root).unwrap();
}
