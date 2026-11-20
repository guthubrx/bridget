use bridget_daemon::cli;
use std::io::Write;

fn init_logging() {
    let json_mode = std::env::var("BRIDGET_LOG_FORMAT")
        .map(|v| v.trim().eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    if json_mode {
        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let line = serde_json::json!({
                    "ts_ms": ts,
                    "level": record.level().as_str(),
                    "target": record.target(),
                    "msg": record.args().to_string(),
                });
                writeln!(buf, "{line}")
            })
            .init();
    } else {
        env_logger::init();
    }
}

fn main() {
    init_logging();
    cli::run();
}
