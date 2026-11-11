//! Grammaire CLI fermée des lectures humaines natives.
use bridget_transport::protocol::{
    HumanLineageAction as A, HumanLineageError as E, HumanLineageRequest,
};
use std::collections::BTreeMap;
use std::io::{Read, Write};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ComposerInput {
    request_id: String,
    expected_round: u32,
    operation: bridget_transport::protocol::HumanComposeOperation,
}

#[cfg(test)]
fn parse(args: &[String]) -> Result<HumanLineageRequest, E> {
    parse_input(args, &mut std::io::empty())
}

fn parse_input(args: &[String], input: &mut dyn Read) -> Result<HumanLineageRequest, E> {
    let command = args.first().ok_or(E::InvalidRequest)?;
    let mut flags = BTreeMap::new();
    let mut i = 1;
    while i < args.len() {
        let key = args[i].as_str();
        if !key.starts_with("--") || flags.contains_key(key) {
            return Err(E::InvalidRequest);
        }
        let val = if matches!(key, "--json" | "--follow" | "--input-stdin") {
            String::new()
        } else {
            i += 1;
            let val = args.get(i).ok_or(E::InvalidRequest)?;
            if val.starts_with("--") {
                return Err(E::InvalidRequest);
            }
            val.clone()
        };
        flags.insert(key, val);
        i += 1;
    }
    if flags.remove("--json").is_none() {
        return Err(E::InvalidRequest);
    }
    let t3_thread_id = flags.remove("--t3-thread").ok_or(E::InvalidRequest)?;
    let project_root = flags.remove("--project-root").ok_or(E::InvalidRequest)?;
    if t3_thread_id.is_empty()
        || t3_thread_id.len() > 2048
        || t3_thread_id
            .chars()
            .any(bridget_core::is_disallowed_control)
        || project_root.len() > 4096
        || !std::path::Path::new(&project_root).is_absolute()
    {
        return Err(E::InvalidRequest);
    }
    let number = |value: Option<String>, default: u64, max: u64, min: u64| -> Result<u64, E> {
        let value = match value {
            Some(v) => v.parse::<u64>().map_err(|_| E::InvalidRequest)?,
            None => default,
        };
        if value < min || value > max {
            return Err(E::InvalidRequest);
        }
        Ok(value)
    };
    let uuid = |value: Option<String>| -> Result<String, E> {
        let v = value.ok_or(E::InvalidRequest)?;
        if crate::threads::canonical_uuid(&v).as_deref() != Some(v.as_str()) {
            return Err(E::InvalidRequest);
        }
        Ok(v)
    };
    let request = match command.as_str() {
        "compose" => {
            let task_id = uuid(flags.remove("--task"))?;
            if flags.remove("--input-stdin").is_none() || !flags.is_empty() {
                return Err(E::InvalidRequest);
            }
            // JSON escaping may grow a 64-Kio body sixfold. Bound bytes before parsing.
            const MAX_INPUT: u64 = 6 * 65536 + 4096;
            let mut body = Vec::new();
            input
                .take(MAX_INPUT + 1)
                .read_to_end(&mut body)
                .map_err(|_| E::InvalidRequest)?;
            if body.len() as u64 > MAX_INPUT {
                return Err(E::InvalidRequest);
            }
            let value: ComposerInput =
                serde_json::from_slice(&body).map_err(|_| E::InvalidRequest)?;
            if crate::threads::canonical_uuid(&value.request_id).as_deref()
                != Some(value.request_id.as_str())
                || !value.operation.bounded(value.expected_round)
            {
                return Err(E::InvalidRequest);
            }
            A::Compose {
                task_id,
                request_id: value.request_id,
                expected_round: value.expected_round,
                operation: value.operation,
            }
        }
        "watch" => A::Watch,
        "cancel" => A::Cancel {
            task_id: uuid(flags.remove("--task"))?,
            request_id: uuid(flags.remove("--request-id"))?,
        },
        "inspect" => match flags.remove("--action").as_deref() {
            Some("list") => A::List {
                limit: number(flags.remove("--limit"), 50, 100, 1)? as u32,
                cursor: flags.remove("--cursor"),
            },
            Some("show") => A::Show {
                task_id: uuid(flags.remove("--task"))?,
                offset: number(flags.remove("--offset"), 0, 256 * 1024, 0)? as u32,
                limit: number(flags.remove("--limit"), 16384, 16384, 1)? as u32,
            },
            Some("journal") => A::Journal {
                task_id: uuid(flags.remove("--task"))?,
                after_seq: number(
                    flags.remove("--after-seq"),
                    0,
                    bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ,
                    0,
                )?,
                limit: number(flags.remove("--limit"), 50, 100, 1)? as u32,
                follow: flags.remove("--follow").is_some(),
            },
            _ => return Err(E::InvalidRequest),
        },
        _ => return Err(E::InvalidRequest),
    };
    let version = match flags.remove("--response-version") {
        Some(value) if matches!(&request, A::List { .. } | A::Show { .. }) => {
            value.parse::<u16>().map_err(|_| E::InvalidRequest)?
        }
        Some(_) => return Err(E::InvalidRequest),
        None => 1,
    };
    if !flags.is_empty() {
        return Err(E::InvalidRequest);
    }
    if let A::List {
        cursor: Some(c), ..
    } = &request
        && (c.len() > 2048 || c.chars().any(char::is_control))
    {
        return Err(E::InvalidRequest);
    }
    let request = HumanLineageRequest {
        version,
        t3_thread_id,
        project_root,
        request,
    };
    request.validate_response_version()?;
    Ok(request)
}

pub(super) fn run(args: &[String]) {
    let mut out = std::io::stdout().lock();
    let mut emit = |value: &serde_json::Value| -> std::io::Result<()> {
        if crate::cli_table::active() {
            crate::cli_table::capture(value.to_string(), true);
            return Ok(());
        }
        serde_json::to_writer(&mut out, value)?;
        writeln!(out)?;
        out.flush()
    };
    let request = match parse_input(args, &mut std::io::stdin().lock()) {
        Ok(r) => r,
        Err(e) => {
            let _ = emit(&e.result());
            crate::cli_table::exit(2)
        }
    };
    let namespace = match crate::environment::Namespace::from_environment() {
        Ok(n) => n,
        Err(_) => {
            let _ = emit(&E::BindingUnavailable.result());
            crate::cli_table::exit(3)
        }
    };
    let mut refused = false;
    // `--response-version` est une demande explicite : la CLI relaye donc le
    // refus du maître dans cette version sans négociation cachée. Le Reader T3
    // effectue seul les tentatives de compatibilité et son staging homogène.
    // La v4 transporte notamment les faits figés et leurs inconnues tels quels :
    // la CLI ne les enrichit jamais avec une mesure ou une horloge locale.
    let result = crate::communication::client::human_lineage(
        &namespace.socket,
        request,
        |value| {
            refused |= value["status"] == "error";
            emit(value)
        },
        || {
            let mut fd = libc::pollfd {
                fd: libc::STDOUT_FILENO,
                events: libc::POLLOUT,
                revents: 0,
            };
            // SAFETY: `fd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            unsafe {
                libc::poll(&mut fd, 1, 0);
            }
            fd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0
        },
    );
    if let Err(error) = result {
        let closed = matches!(
            error,
            crate::communication::client::ClientError::Technical {
                code: "watch_output_closed",
                ..
            }
        );
        if !closed {
            let _ = emit(&E::BindingUnavailable.result());
        }
        crate::cli_table::exit(if closed { 0 } else { 3 });
    }
    if refused {
        crate::cli_table::exit(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn composer_args() -> Vec<String> {
        [
            "compose",
            "--json",
            "--t3-thread",
            "parent",
            "--project-root",
            "/project",
            "--task",
            "25000000-0000-4000-8000-000000000001",
            "--input-stdin",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[test]
    fn spec258_cli_compose_stdin_ferme_et_borne() {
        for (round, operation) in [
            (0, serde_json::json!({"kind":"read"})),
            (
                3,
                serde_json::json!({"kind":"send","body":"a\nb","intent":"steer"}),
            ),
            (
                3,
                serde_json::json!({"kind":"select_runtime","model":"m","effort":null}),
            ),
            (3, serde_json::json!({"kind":"interrupt"})),
        ] {
            let bytes = serde_json::to_vec(
                &serde_json::json!({"request_id":"25000000-0000-4000-8000-000000000002",
                "expected_round":round,"operation":operation}),
            )
            .unwrap();
            let parsed = parse_input(&composer_args(), &mut bytes.as_slice()).unwrap();
            assert!(
                matches!(parsed.request, A::Compose{expected_round,..} if expected_round==round)
            );
        }
        for raw in [
            "{}".to_owned(),
            "x".repeat(6*65536+4097),
            serde_json::json!({"request_id":"25000000-0000-4000-8000-000000000002","expected_round":0,"operation":{"kind":"interrupt"}}).to_string(),
            serde_json::json!({"request_id":"25000000-0000-4000-8000-000000000002","expected_round":0,"operation":{"kind":"read"},"agent":"forged"}).to_string(),
        ] { assert!(parse_input(&composer_args(),&mut raw.as_bytes()).is_err()); }
        let mut args = composer_args();
        args.push("--request-id".into());
        args.push("forged".into());
        assert!(parse_input(&args, &mut std::io::empty()).is_err());
    }

    fn request_args(action: &str, extra: &[&str]) -> Vec<String> {
        let mut args: Vec<String> = [
            "inspect",
            "--json",
            "--t3-thread",
            "thread-162",
            "--project-root",
            "/project",
            "--action",
            action,
        ]
        .into_iter()
        .map(String::from)
        .collect();
        args.extend(extra.iter().map(|arg| (*arg).to_owned()));
        args
    }

    #[test]
    fn native162_lineage_cli_defaults_to_legacy_projection() {
        let request = parse(&request_args("list", &[])).unwrap();
        assert_eq!(request.version, 1);
        assert!(matches!(
            request.request,
            A::List {
                limit: 50,
                cursor: None
            }
        ));
        assert_eq!(
            request.request.capability(),
            bridget_transport::protocol::ClientCapability::HumanLineageViewV1
        );
    }

    #[test]
    fn spec163_lineage_cli_negocie_v3_seulement_pour_list_et_show() {
        for (wire, version) in [("2", 2), ("3", 3), ("4", 4)] {
            let list = parse(&request_args("list", &["--response-version", wire])).unwrap();
            let show = parse(&request_args(
                "show",
                &[
                    "--task",
                    "00000000-0000-4000-8000-000000000162",
                    "--response-version",
                    wire,
                ],
            ))
            .unwrap();
            assert_eq!(list.version, version);
            assert_eq!(show.version, version);
        }
    }

    #[test]
    fn spec163_lineage_cli_refuse_v3_hors_des_lectures_paginees() {
        for version in ["1", "2", "3", "4"] {
            let journal = request_args(
                "journal",
                &[
                    "--task",
                    "00000000-0000-4000-8000-000000000162",
                    "--response-version",
                    version,
                ],
            );
            assert!(matches!(parse(&journal), Err(E::InvalidRequest)));
            for command in ["watch", "cancel"] {
                let mut args: Vec<String> = [
                    command,
                    "--json",
                    "--t3-thread",
                    "thread-162",
                    "--project-root",
                    "/project",
                    "--response-version",
                    version,
                ]
                .into_iter()
                .map(String::from)
                .collect();
                if command == "cancel" {
                    args.extend(
                        [
                            "--task",
                            "00000000-0000-4000-8000-000000000162",
                            "--request-id",
                            "00000000-0000-4000-8000-000000000163",
                        ]
                        .map(String::from),
                    );
                }
                assert!(matches!(parse(&args), Err(E::InvalidRequest)));
            }
        }
    }

    #[test]
    fn native162_lineage_cli_refuses_malformed_or_duplicate_version() {
        for value in ["-1", "65536", "two"] {
            assert!(matches!(
                parse(&request_args("list", &["--response-version", value])),
                Err(E::InvalidRequest)
            ));
        }
        assert!(matches!(
            parse(&request_args(
                "list",
                &["--response-version", "2", "--response-version", "1",]
            )),
            Err(E::InvalidRequest)
        ));
    }
}
