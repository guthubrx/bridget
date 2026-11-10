//! Présentation native225 : une seule exécution, mêmes données et mêmes droits.
//! Le CLI collecte sa propre sortie dans le même processus, sans redirection de
//! descripteur ni sous-processus CLI. Hors --table les macros restent identiques.
use serde_json::{Value, json};
use std::cell::RefCell;
use std::io::{self, IsTerminal, Read, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MAX_CAPTURE: usize = 16 * 1024 * 1024;
thread_local! { static OUTPUT: RefCell<Option<Context>> = const { RefCell::new(None) }; }
struct Context {
    family: String,
    buffer: String,
    raw: bool,
    width: usize,
    color: bool,
}
pub(crate) struct Guard(Option<Context>);
impl Drop for Guard {
    fn drop(&mut self) {
        flush();
        OUTPUT.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
pub(crate) fn active() -> bool {
    OUTPUT.with(|s| s.borrow().as_ref().is_some_and(|v| !v.raw))
}
pub(crate) fn capture(mut text: String, newline: bool) {
    if newline {
        text.push('\n');
    }
    let fallback = OUTPUT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(ctx) = slot.as_mut() else {
            return Some(text);
        };
        if ctx.raw {
            return Some(text);
        }
        if ctx.buffer.len().saturating_add(text.len()) > MAX_CAPTURE {
            ctx.raw = true;
            let mut original = std::mem::take(&mut ctx.buffer);
            original.push_str(&text);
            return Some(original);
        }
        ctx.buffer.push_str(&text);
        None
    });
    if let Some(original) = fallback {
        eprintln!(
            "Tableau trop volumineux : sortie originale conservée, sans rejouer la commande."
        );
        write_output(original.as_bytes());
    }
}
/// Sortie d'un helper natif déjà exécuté, avec la même borne que les macros.
pub(crate) fn capture_reader(mut reader: impl io::Read) -> io::Result<()> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(MAX_CAPTURE as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_CAPTURE || std::str::from_utf8(&bytes).is_err() {
        flush();
        OUTPUT.with(|s| {
            if let Some(ctx) = s.borrow_mut().as_mut() {
                ctx.raw = true;
            }
        });
        eprintln!("Sortie non tabulable ou trop volumineuse : contenu original conservé.");
        let mut out = io::stdout().lock();
        out.write_all(&bytes)?;
        io::copy(&mut reader, &mut out)?;
    } else {
        capture(String::from_utf8(bytes).expect("UTF-8 contrôlé"), false);
    }
    Ok(())
}
pub(crate) fn flush() {
    let output = OUTPUT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let ctx = slot.as_mut()?;
        if ctx.raw || ctx.buffer.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut ctx.buffer);
        Some(render(&ctx.family, &text, ctx.width, ctx.color))
    });
    if let Some(output) = output {
        write_output(output.as_bytes());
    }
}
fn write_output(bytes: &[u8]) {
    let mut out = io::stdout().lock();
    if let Err(error) = out.write_all(bytes).and_then(|()| out.flush()) {
        eprintln!(
            "Sortie du tableau interrompue : {error}. Conserver les références déjà reçues ; ne pas recréer une opération d'issue inconnue."
        );
        std::process::exit(1);
    }
}
pub(crate) fn exit(code: i32) -> ! {
    flush();
    std::process::exit(code)
}
macro_rules! table_println {
    () => { if $crate::cli_table::active() {$crate::cli_table::capture(String::new(),true)}else{std::println!()} };
    ($($arg:tt)*) => { if $crate::cli_table::active() {$crate::cli_table::capture(format!($($arg)*),true)}else{std::println!($($arg)*)} };
}
macro_rules! table_print {
    ($($arg:tt)*) => { if $crate::cli_table::active() {$crate::cli_table::capture(format!($($arg)*),false)}else{std::print!($($arg)*)} };
}
pub(crate) use table_print;
pub(crate) use table_println;

// Le registre est aussi l'inventaire des raccords : les flux et moteurs restent
// hors de ce parcours. Les valeurs d'options et tout ce qui suit -- sont intacts.
const FAMILIES: &[&str] = &[
    "agents",
    "who",
    "specialists",
    "task",
    "control",
    "requests",
    "inbox",
    "queue",
    "ledger",
    "journal",
    "events",
    "wake",
    "thread",
    "handoff",
    "artifact",
    "delegate",
    "auto-delegate",
    "spawn",
    "runtime",
    "identity",
    "domain",
    "dnd",
    "guichet",
    "questions",
    "reaper",
    "reprise",
    "status",
    "accounts",
    "clients",
    "formats",
    "mobile",
    "updates",
    "federate",
    "lineage",
    "worker",
    "send",
    "reply",
    "cancel",
    "stop",
    "relaunch",
    "decommission",
    "adopt-stopped",
    "rename",
    "discover",
    "delegate-grant",
];
const BOOLEANS: &[&str] = &[
    "--dormant",
    "--local-only",
    "--synchronize",
    "--all",
    "--global",
    "--include-stopped",
    "--json",
    "--help",
    "-h",
    "--follow",
    "--apply",
    "--dry-run",
    "--observed",
    "--refresh-models",
    "--force",
    "--verbose",
    "--reply",
    "--yes",
    "--resume-on-limit",
    "--persistent",
    "--no-persistent",
    "--json-stdin",
    "--history",
    "--cancel-active",
    "--once",
    "--sync",
    "--compact",
    "--include-dead",
    "--reset",
    "--remove",
    "--revoke",
];
#[derive(Debug)]
pub(crate) struct Arguments {
    pub args: Vec<String>,
    pub family: Option<String>,
}
pub(crate) fn normalize(mut args: Vec<String>) -> Result<Arguments, String> {
    let mut table = false;
    while args.get(1).is_some_and(|v| v == "--table") {
        args.remove(1);
        table = true;
    }
    let Some(root) = args.get(1).cloned() else {
        return if table {
            Err("--table exige une commande Bridget.".into())
        } else {
            Ok(Arguments { args, family: None })
        };
    };
    if !FAMILIES.contains(&root.as_str()) {
        return if table {
            Err(format!(
                "--table ne s'applique pas à {root} ; aucun programme n'a été lancé."
            ))
        } else {
            Ok(Arguments { args, family: None })
        };
    }
    // Un unique texte libre n'est pas une option d'affichage.
    if !table && matches!(root.as_str(), "reply" | "rename" | "domain") && args.len() == 3 {
        return Ok(Arguments { args, family: None });
    }
    let mut clean = args[..2].to_vec();
    let mut i = 2;
    let mut explicit_json = false;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            clean.extend_from_slice(&args[i..]);
            break;
        }
        if arg == "--table" {
            table = true;
            i += 1;
            continue;
        }
        if matches!(root.as_str(), "send" | "reply") && !arg.starts_with('-') {
            // Le corps reste exactement celui reçu. --table après son premier mot
            // ne devient pas une permission implicite d'envoyer un texte modifié.
            clean.extend_from_slice(&args[i..]);
            break;
        }
        if arg == "--json" {
            explicit_json = true;
        }
        clean.push(arg.clone());
        if arg.starts_with('-')
            && !arg.contains('=')
            && !BOOLEANS.contains(&arg.as_str())
            && i + 1 < args.len()
        {
            i += 1;
            clean.push(args[i].clone());
        }
        i += 1;
    }
    if !table {
        return Ok(Arguments { args, family: None });
    }
    if explicit_json {
        return Err("--table et --json sont deux formats distincts ; choisissez-en un. Aucun appel n'a été transmis.".into());
    }
    let op = clean.get(2).cloned().unwrap_or_default();
    if (root == "worker" && op != "diagnose")
        || (root == "agents" && matches!(op.as_str(),"sleep"|"wake"))
        || (root == "mobile" && matches!(op.as_str(), "pair" | "serve"))
        || (root == "updates" && matches!(op.as_str(), "watch" | "once" | "setup"))
        || (root == "thread" && op == "watch")
        || (root == "lineage" && op == "watch")
        || clean.iter().any(|v| v == "--follow")
        || (root == "artifact" && matches!(op.as_str(), "read" | "publish"))
        || (root == "federate" && op != "status")
    {
        return Err("--table concerne les vues finies ; ce flux, ce contenu ou cette opération interactive conserve son format natif. Aucun appel n'a été transmis.".into());
    }
    let family = if matches!(root.as_str(), "who" | "discover") {
        "agents".to_owned()
    } else {
        root.clone()
    };

    // Chaque ajout correspond à une option déjà acceptée par CE parseur.
    let json = root == "agents"
        || matches!(
            family.as_str(),
            "specialists"
                | "task"
                | "requests"
                | "handoff"
                | "delegate"
                | "auto-delegate"
                | "reaper"
                | "lineage"
        )
        || family == "ledger" && matches!(op.as_str(), "search" | "read")
        || family == "control" && op == "delegations"
        || family == "clients" && op == "check"
        || family == "formats" && op == "policy"
        || family == "thread" && op == "inspect";
    if json && !matches!(root.as_str(), "who" | "discover") {
        clean.push("--json".into());
    }
    Ok(Arguments {
        args: clean,
        family: Some(family),
    })
}
pub(crate) fn prepare(args: Vec<String>) -> Result<(Vec<String>, Guard), String> {
    let parsed = normalize(args)?;
    let previous = if let Some(family) = parsed.family {
        let tty = io::stdout().is_terminal();
        let width = terminal_width();
        let color = tty
            && std::env::var_os("NO_COLOR").is_none()
            && std::env::var("TERM").is_ok_and(|v| v != "dumb");
        OUTPUT.with(|s| {
            s.borrow_mut().replace(Context {
                family,
                buffer: String::new(),
                raw: false,
                width,
                color,
            })
        })
    } else {
        None
    };
    Ok((parsed.args, Guard(previous)))
}
fn terminal_width() -> usize {
    let mut winsize: libc::winsize = unsafe { std::mem::zeroed() };
    let width = if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) } == 0
        && winsize.ws_col > 0
    {
        winsize.ws_col as usize
    } else {
        std::env::var("COLUMNS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120)
    };
    width.clamp(32, 240)
}
fn at<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(v, |v, k| v.get(k))
}
fn pick(v: &Value, paths: &str) -> Value {
    paths
        .split('|')
        .find_map(|p| at(v, p).filter(|v| !v.is_null() && **v != json!("")))
        .cloned()
        .unwrap_or(Value::Null)
}
fn text(v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::String(v) => v.clone(),
        Value::Bool(v) => if *v { "Oui" } else { "Non" }.into(),
        Value::Array(v) => v.iter().map(text).collect::<Vec<_>>().join(", "),
        Value::Object(v) => v
            .iter()
            .map(|(k, v)| format!("{k}: {}", text(v)))
            .collect::<Vec<_>>()
            .join(" ; "),
        _ => v.to_string(),
    }
}
fn clean(v: &str) -> String {
    // Retirer une séquence CSI complète, pas seulement son ESC.
    let mut plain = String::new();
    let mut chars = v.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for x in chars.by_ref() {
                    if ('@'..='~').contains(&x) {
                        break;
                    }
                }
            }
            continue;
        }
        plain.push(c);
    }
    plain
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .filter(|c| !bridget_core::is_disallowed_control(*c))
        .collect()
}
fn state(v: &Value) -> String {
    if pick(v, "specialist.enabled|enabled") == json!(false) {
        return "Désactivé".into();
    }
    let s = pick(
        v,
        "presence.state|agent.state|state|status|stage|execution_state|submission_state|delivery_status",
    );
    match s.as_str() {
        Some("connected") => "Connecté".into(),
        Some("busy") => "Occupé".into(),
        Some("working") => "En cours".into(),
        Some("result_available") => "Résultat disponible".into(),
        Some("starting") => "Démarrage".into(),
        Some("failed" | "error") => "Échec".into(),
        Some("cancelled") => "Annulé".into(),
        Some("stopped") => "Arrêté".into(),
        Some("disconnected") => "Déconnecté".into(),
        Some("pending") => "En attente".into(),
        Some("refused") => "Refusé".into(),
        Some("not_observed") => "Non observé".into(),
        Some("stale") => "Périmé".into(),
        Some("not_reported") => "Non signalé".into(),
        Some("up_to_date") => "À jour".into(),
        Some("absent" | "missing") => "Absent".into(),
        None => "Non observé".into(),
        _ => text(&s),
    }
}
#[derive(Clone, Copy)]
struct Column(&'static str, &'static str);
fn columns(family: &str, rows: &[&Value]) -> Vec<Column> {
    let first = rows.first().copied().unwrap_or(&Value::Null);
    use Column as C;
    if first.get("specialist").is_some() || (family == "specialists" && rows.is_empty()) {
        return vec![
            C("NOM HUMAIN", "display_name|specialist.agent_id"),
            C("RÔLE", "@roles"),
            C("PROJET", "@projects"),
            C("HÔTE", "presence.host"),
            C("ÉTAT", "@state"),
        ];
    }
    if family == "specialists"
        && (first.get("profile_id").is_some() || first.get("definition").is_some())
    {
        return vec![
            C(
                "PROFIL",
                "name|definition.name|definition.definition.name|profile_id",
            ),
            C(
                "RÔLES",
                "roles|definition.roles|definition.definition.roles",
            ),
            C("VERSION", "revision|definition.revision"),
            C("APPROBATION", "approved|approved_ref"),
        ];
    }
    if first.get("memory_id").is_some() {
        return vec![
            C("RÉFÉRENCE", "memory_id"),
            C("TYPE", "kind"),
            C("SOUVENIR", "text"),
            C("PROJET", "project_id"),
            C("PARTAGÉ", "shared"),
            C("VERSION", "revision"),
        ];
    }
    match family {
        "agents" => vec![
            C("NOM HUMAIN", "agent.display_name|display_name|agent_id"),
            C("RÔLE", "agent.domain|domain|agent.agent_type|agent_type"),
            C("PROJET", "communication_project.root|project_id|project"),
            C("HÔTE", "agent.host|host"),
            C("ÉTAT", "@state"),
            C("DORMANT", "tracking.dormant"),
        ],
        "task" | "control" | "auto-delegate" | "spawn" => vec![
            C("RÉFÉRENCE", "task_id|id|reference"),
            C(
                "AGENT",
                "display_name|child_display_name|child_agent_id|child|owner",
            ),
            C("PROJET", "project_id|project.root"),
            C("HÔTE", "remote_placement.machine_id|machine_id|host"),
            C("ÉTAT", "@state"),
            C("RONDE", "round|round_no"),
        ],
        "accounts" => vec![
            C("CONNEXION", "connection_id|id"),
            C("MOTEUR", "engine"),
            C("HÔTE", "machine_id|host"),
            C("ACTIVÉ", "enabled"),
            C("AUTHENTIFICATION", "auth"),
            C("GÉRÉ", "managed"),
        ],
        "delegate" => vec![
            C("HÔTE", "machine_id|host"),
            C("CONNEXION", "connection_id"),
            C("MOTEUR", "engine"),
            C("MODÈLE", "model"),
            C("EFFORTS", "supported_efforts|efforts"),
            C(
                "OBSERVATION",
                "observation.status|eligibility|status|entitlement",
            ),
        ],
        "updates" => vec![
            C("HÔTE", "host|host_id|machine_id|_key"),
            C("OBSERVÉ À", "observed_at"),
            C("INSTALLÉ", "installed.version|installed|version|build_id"),
            C(
                "DISPONIBLE",
                "available.version|available|available_version",
            ),
            C("ÉTAT", "@state"),
            C("ACTION", "next_action"),
        ],
        "inbox" => vec![
            C("RÉFÉRENCE", "id"),
            C("TYPE", "kind"),
            C("ÉTAT", "@state"),
            C("OCCURRENCES", "occurrences"),
            C("CHOIX", "options"),
            C("CONTEXTE", "context"),
        ],
        "requests" | "ledger" => vec![
            C("RÉFÉRENCE", "request_id|id|message_id"),
            C(
                "ÉMETTEUR",
                "sender_display_name|from_display_name|sender|from",
            ),
            C(
                "DESTINATAIRE",
                "target_display_name|to_display_name|target|to",
            ),
            C("RÉSUMÉ", "body|summary|message"),
            C("ÉTAT", "@state"),
            C("DATE", "ts|created_at|issued_at"),
        ],
        "questions" => vec![
            C("RÉFÉRENCE", "request_id|id"),
            C("AGENT", "agent_display_name|agent_id|owner"),
            C("QUESTION", "question|text|title|prompt"),
            C("ÉTAT", "@state"),
            C("DATE", "created_at"),
        ],
        "queue" => vec![
            C("POSITION", "position|index|priority"),
            C("RÉFÉRENCE", "request_id|message_id|id"),
            C("ÉMETTEUR", "sender_display_name|sender|from"),
            C("RÉSUMÉ", "body|task|text"),
            C("ÉTAT", "@state"),
        ],
        "wake" => vec![
            C("RÉFÉRENCE", "schedule_id|id"),
            C("NOM", "label|name"),
            C("CIBLE", "target.agent_id|target|agent_id"),
            C("CADENCE", "cadence|schedule"),
            C("ÉTAT", "@state"),
            C("PROCHAINE", "next_at|next_wake_at"),
        ],
        "thread" | "lineage" | "handoff" => vec![
            C("RÉFÉRENCE", "thread_id|task_id|id|ref"),
            C("TITRE", "title|label|summary"),
            C("RESPONSABLE", "owner_display_name|owner|agent_id"),
            C("PROJET", "project_id|project.root"),
            C("ÉTAT", "@state"),
            C("DATE", "created_at|updated_at"),
        ],
        "artifact" => vec![
            C("RÉFÉRENCE", "artifact_id|id|ref"),
            C("NOM", "name|label|title"),
            C("TYPE", "mime_type|kind"),
            C("TAILLE", "bytes|size"),
            C("PROJET", "project_id"),
            C("DATE", "created_at"),
        ],
        "clients" => vec![
            C("PROGRAMME", "program|name|engine"),
            C("HÔTE", "host|machine_id"),
            C("VERSION", "version|observed_version"),
            C("MINIMUM", "minimum_version|required_version"),
            C("VERDICT", "verdict|status"),
            C("ACTION", "next_action|recovery"),
        ],
        "events" | "journal" => vec![
            C("SÉQUENCE", "seq|sequence|id"),
            C("TYPE", "event|kind|type"),
            C("AGENT", "agent_id|agent"),
            C("RÉSUMÉ", "payload.text|payload.body|text|message|summary"),
            C("DATE", "at|ts|observed_at"),
        ],
        "mobile" => vec![
            C("APPAREIL", "name|display_name|device_id|id"),
            C("ÉTAT", "@state"),
            C("PORTÉES", "scopes|capabilities"),
            C("VERSION", "version|app_version"),
            C("DERNIÈRE VUE", "last_seen_at|updated_at"),
        ],
        _ => vec![],
    }
}
fn cell(row: &Value, path: &str) -> String {
    match path {
        "@state" => state(row),
        "@roles" => row["roles"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|r| text(&pick(r, "label|role_id")))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "—".into()),
        "@projects" => {
            if row["specialist"]["scope"]["transversal"] == true {
                "Tous les projets".into()
            } else {
                text(&row["specialist"]["scope"]["projects"])
            }
        }
        _ => text(&pick(row, path)),
    }
}
fn color_cell(s: &str, color: bool) -> String {
    if !color {
        return s.into();
    }
    let c = if matches!(s, "Connecté" | "ok" | "compatible" | "À jour") {
        "32"
    } else if matches!(s, "Occupé" | "En attente" | "En cours" | "Périmé") {
        "33"
    } else if matches!(s, "Échec" | "Refusé" | "Absent" | "Déconnecté") {
        "31"
    } else if matches!(s, "Désactivé" | "Non observé" | "—") {
        "2"
    } else {
        return s.into();
    };
    format!("\x1b[{c}m{s}\x1b[0m")
}
fn wrap(s: &str, width: usize) -> Vec<String> {
    let s = clean(s);
    let mut lines = vec![];
    let mut line = String::new();
    let mut used = 0;
    for c in s.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if used + cw > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        line.push(c);
        used += cw;
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}
fn table(headers: &[String], rows: &[Vec<String>], width: usize, color: bool) -> String {
    if headers.is_empty() {
        return String::new();
    }
    let n = headers.len();
    // Sur terminal très étroit, une fiche nom/valeur garde toutes les colonnes.
    if width < 5 * n + 1 && n > 2 {
        let mut out = String::new();
        for (i, row) in rows.iter().enumerate() {
            out.push_str(&format!("{}\n", i + 1));
            let cells = headers
                .iter()
                .zip(row)
                .map(|(h, v)| vec![h.clone(), v.clone()])
                .collect::<Vec<_>>();
            out.push_str(&table(
                &["CHAMP".into(), "VALEUR".into()],
                &cells,
                width,
                color,
            ));
        }
        if rows.is_empty() {
            out.push_str("Aucune entrée.\n");
        }
        return out;
    }
    let budget = width.saturating_sub(3 * n + 1).max(n);
    let mut widths = (0..n)
        .map(|i| {
            headers[i]
                .width()
                .max(
                    rows.iter()
                        .filter_map(|r| r.get(i))
                        .map(|s| clean(s).width())
                        .max()
                        .unwrap_or(1),
                )
                .max(2)
        })
        .collect::<Vec<_>>();
    while widths.iter().sum::<usize>() > budget {
        let i = (0..n).max_by_key(|&i| widths[i]).unwrap();
        if widths[i] <= 2 {
            break;
        }
        widths[i] -= 1;
    }
    let mut out = String::new();
    let border = |left: char, join: char, right: char| -> String {
        format!(
            "{}{}{}\n",
            left,
            widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join(&join.to_string()),
            right
        )
    };
    out.push_str(&border('╭', '┬', '╮'));
    let mut row = |cells: &[String], header: bool| {
        let chunks = cells
            .iter()
            .enumerate()
            .map(|(i, s)| wrap(s, widths[i]))
            .collect::<Vec<_>>();
        let height = chunks.iter().map(Vec::len).max().unwrap_or(1);
        for line in 0..height {
            out.push('│');
            for (i, chunks) in chunks.iter().enumerate() {
                let value = chunks.get(line).map(String::as_str).unwrap_or("");
                let rendered = if header && color {
                    format!("\x1b[1;36m{value}\x1b[0m")
                } else {
                    color_cell(value, color)
                };
                out.push_str(&format!(
                    " {rendered}{} │",
                    " ".repeat(widths[i].saturating_sub(value.width()))
                ));
            }
            out.push('\n');
        }
    };
    row(headers, true);
    out.push_str(&border('├', '┼', '┤'));
    for cells in rows {
        let chunks = cells
            .iter()
            .enumerate()
            .map(|(i, s)| wrap(s, widths[i]))
            .collect::<Vec<_>>();
        for line in 0..chunks.iter().map(Vec::len).max().unwrap_or(1) {
            out.push('│');
            for (i, chunks) in chunks.iter().enumerate() {
                let value = chunks.get(line).map(String::as_str).unwrap_or("");
                out.push_str(&format!(
                    " {}{} │",
                    color_cell(value, color),
                    " ".repeat(widths[i].saturating_sub(value.width()))
                ));
            }
            out.push('\n');
        }
    }
    out.push_str(&border('╰', '┴', '╯'));
    if rows.is_empty() {
        out.push_str("Aucune entrée dans la portée demandée.\n");
    }
    out
}
fn render_rows(family: &str, values: &[Value], width: usize, color: bool) -> String {
    let rows = values.iter().collect::<Vec<_>>();
    let cols = columns(family, &rows);
    // Schéma nouveau/inconnu : aucune donnée supprimée par une fausse table vide.
    if cols.is_empty()
        || (!rows.is_empty()
            && cols.iter().all(|c| {
                rows.iter()
                    .all(|v| cell(v, c.1) == "—" || cell(v, c.1) == "Non observé")
            }))
    {
        return table(
            &["CHAMP".into(), "VALEUR".into()],
            &values
                .iter()
                .flat_map(|v| match v.as_object() {
                    Some(obj) => obj
                        .iter()
                        .map(|(k, v)| vec![k.clone(), text(v)])
                        .collect::<Vec<_>>(),
                    None => vec![vec!["Valeur".into(), text(v)]],
                })
                .collect::<Vec<_>>(),
            width,
            color,
        );
    }
    // Une référence déjà publiée n'est jamais inventée depuis une observation.

    table(
        &cols.iter().map(|c| c.0.into()).collect::<Vec<_>>(),
        &rows
            .iter()
            .map(|r| cols.iter().map(|c| cell(r, c.1)).collect())
            .collect::<Vec<_>>(),
        width,
        color,
    )
}
fn sections(family: &str, value: &Value, width: usize, color: bool) -> String {
    if let Some(rows) = value.as_array() {
        return render_rows(family, rows, width, color);
    }
    let Some(object) = value.as_object() else {
        return table(&["VALEUR".into()], &[vec![text(value)]], width, color);
    };
    // Les fiches spécialisées conservent leur contexte et leurs blocs longs.
    if object.contains_key("specialist") && family == "specialists" {
        let mut out = render_rows("specialists", std::slice::from_ref(value), width, color);
        let details = object
            .iter()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "specialist" | "display_name" | "roles" | "presence"
                )
            })
            .map(|(key, v)| vec![key.clone(), text(v)])
            .collect::<Vec<_>>();
        if !details.is_empty() {
            out.push_str(&table(
                &["DÉTAIL".into(), "VALEUR".into()],
                &details,
                width,
                color,
            ));
        }
        return out;
    }
    if object.contains_key("task_id")
        || object.contains_key("connection_id")
        || object.contains_key("agent")
    {
        let mut out = render_rows(family, std::slice::from_ref(value), width, color);
        out.push_str(&table(
            &["DÉTAIL".into(), "VALEUR".into()],
            &object
                .iter()
                .map(|(k, v)| vec![k.clone(), text(v)])
                .collect::<Vec<_>>(),
            width,
            color,
        ));
        return out;
    }
    let mut out = String::new();
    let mut scalar = vec![];
    for (key, v) in object {
        // Options de formulaire spécialisées : utiles en JSON, pas dans une liste.
        if family == "specialists"
            && matches!(
                key.as_str(),
                "caller_projects" | "launch_options" | "project_options" | "presence_scope"
            )
        {
            continue;
        }
        if key == "version" && v.is_number() {
            continue;
        }
        if key == "status" && matches!(v.as_str(), Some("ok" | "read" | "preview")) {
            continue;
        }

        match v {
            Value::Array(_) | Value::Object(_) if !v.is_null() => {
                if !out.is_empty() {
                    out.push('\n');
                }
                if !matches!(key.as_str(), "data" | "items" | "result") {
                    out.push_str(&format!("{}\n", wrap(key, width).join("\n")));
                }
                if key == "hosts" && v.is_object() {
                    let rows = v
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(host, row)| {
                            let mut row = row.clone();
                            if let Some(r) = row.as_object_mut() {
                                r.insert("_key".into(), json!(host));
                            }
                            row
                        })
                        .collect::<Vec<_>>();
                    out.push_str(&render_rows("updates", &rows, width, color));
                } else {
                    out.push_str(&sections(family, v, width, color));
                }
            }
            Value::Null if key == "next_cursor" => {}
            _ => scalar.push(vec![key.clone(), text(v)]),
        }
    }
    if out.is_empty() && scalar.is_empty()
        && let Some(status) = object.get("status") {
            scalar.push(vec!["ÉTAT".into(), text(status)]);
        }
    if !scalar.is_empty() {
        out.push_str(&table(
            &["CHAMP".into(), "VALEUR".into()],
            &scalar,
            width,
            color,
        ));
    }
    if out.is_empty() {
        out.push_str(&render_rows(family, &[], width, color));
    }
    out
}
pub(crate) fn render(family: &str, input: &str, width: usize, color: bool) -> String {
    let width = width.clamp(16, 240);
    if let Ok(value) = serde_json::from_str::<Value>(input) {
        return sections(family, &value, width, color);
    }
    // Plusieurs réponses bornées (inspection/journal) restent dans leur ordre.
    let lines = input
        .lines()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>();
    if !lines.is_empty()
        && lines
            .iter()
            .all(|line| serde_json::from_str::<Value>(line).is_ok())
    {
        return lines
            .iter()
            .map(|line| {
                sections(
                    family,
                    &serde_json::from_str::<Value>(line).unwrap(),
                    width,
                    color,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    if lines
        .iter()
        .any(|line| serde_json::from_str::<Value>(line).is_ok())
    {
        return lines
            .iter()
            .map(|line| match serde_json::from_str::<Value>(line) {
                Ok(value) => sections(family, &value, width, color),
                Err(_) => table(
                    &["INFORMATION".into()],
                    &[vec![(*line).into()]],
                    width,
                    color,
                ),
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    let rows = lines
        .iter()
        .map(|line| {
            if matches!(
                family,
                "status"
                    | "control"
                    | "runtime"
                    | "identity"
                    | "domain"
                    | "dnd"
                    | "spawn"
                    | "stop"
                    | "relaunch"
                    | "decommission"
                    | "adopt-stopped"
            ) {
                let (key, value) = line.split_once(':').unwrap_or(("Information", line));
                vec![key.trim().into(), value.trim().into()]
            } else {
                vec!["Ligne".into(), (*line).into()]
            }
        })
        .collect::<Vec<_>>();
    table(&["INDICATEUR".into(), "VALEUR".into()], &rows, width, color)
}

#[cfg(test)]
mod tests225 {
    use super::*;
    #[test]
    fn tracking230_table_n_envoie_pas_une_mutation_et_accepte_historique() {
        assert!(normalize(args(&["bridget","agents","sleep","23000000-0000-4000-8000-000000000001","--table"])).is_err());
        let history=normalize(args(&["bridget","--table","agents","--dormant","--global"])).unwrap();
        assert_eq!(history.args,args(&["bridget","agents","--dormant","--global","--json"]));
    }
    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|v| (*v).into()).collect()
    }
    #[test]
    fn table225_global_positions_values_and_protocols() {
        let a = normalize(args(&["bridget", "--table", "specialists", "list"])).unwrap();
        let b = normalize(args(&["bridget", "specialists", "list", "--table"])).unwrap();
        assert_eq!(a.args, b.args);
        assert_eq!(a.family, b.family);
        assert_eq!(a.args, args(&["bridget", "specialists", "list", "--json"]));
        let a = normalize(args(&["bridget", "task", "continue", "--task", "--table"])).unwrap();
        assert!(a.family.is_none());
        assert_eq!(
            a.args,
            args(&["bridget", "task", "continue", "--task", "--table"])
        );
        assert!(normalize(args(&["bridget", "--table", "agents", "--json"])).is_err());
        assert!(normalize(args(&["bridget", "--table", "mcp"])).is_err());
        assert!(normalize(args(&["bridget", "--table", "thread", "watch"])).is_err());
        let p = normalize(args(&["bridget", "codex", "--table"])).unwrap();
        assert!(p.family.is_none());
        assert_eq!(p.args, args(&["bridget", "codex", "--table"]));
        let p = normalize(args(&["bridget", "agents", "--", "--table"])).unwrap();
        assert!(p.family.is_none());
    }
    #[test]
    fn table225_width_unicode_colors_unknown_and_host() {
        let v = json!({"status":"ok","data":{"items":[{"display_name":"Gentil 🥔","specialist":{"agent_id":"stable-uuid","enabled":true,"scope":{"projects":["bridget"]}},"roles":[{"label":"Patate","role_id":"patate"}],"presence":{"state":"busy","host":"cartae"}}]}});
        let s = render("specialists", &v.to_string(), 80, false);
        for word in ["Gentil", "Patate", "bridget", "cartae", "Occupé"] {
            assert!(s.contains(word), "{s}");
        }
        assert!(!s.contains('\x1b'));
        assert!(s.lines().all(|l| l.width() <= 80));
        assert!(render("specialists", &v.to_string(), 80, true).contains("\x1b["));
        let s = render("specialists", &v.to_string(), 32, false);
        assert!(s.lines().all(|l| l.width() <= 32), "{s}");
        let empty = render("agents", "[]", 80, false);
        assert!(empty.contains("Aucune entrée"));
        let s=render("specialists",&json!({"data":{"items":[{"display_name":"Inconnu","specialist":{"enabled":true},"roles":[]} ]}}).to_string(),80,false);
        assert!(s.contains("Non observé"));
        assert!(!s.contains("Connecté"));
        assert!(
            !render(
                "unknown",
                &json!({"field":"\u{1b}[31msecret"}).to_string(),
                80,
                false
            )
            .contains('\x1b')
        );
    }
    #[test]
    fn table225_flags_are_specific_and_receipts_keep_context() {
        for input in [
            &["bridget", "control", "budget", "0", "--table"][..],
            &["bridget", "spawn", "codex", "--persistent", "--table"][..],
            &["bridget", "inbox", "list", "--table"][..],
            &["bridget", "guichet", "deposer", "mission-status", "--table"][..],
            &["bridget", "clients", "policy", "--table"][..],
            &["bridget", "ledger", "--limit", "8", "--table"][..],
        ] {
            let normalized = normalize(args(input)).unwrap();
            assert!(
                !normalized.args.iter().any(|x| x == "--json"),
                "{normalized:?}"
            );
            assert!(normalized.family.is_some());
        }
        let s=render("task",&json!({"status":"refused","code":"named_cause","detail":{"admission":{"state":"not_recorded","task_id":null}},"next_action":"Conserver la clé"}).to_string(),120,false);
        for word in ["refused", "named_cause", "not_recorded", "Conserver la clé"] {
            assert!(s.contains(word), "{s}");
        }
        let s = render(
            "specialists",
            &json!({"status":"ok","data":{"items":[],"next_cursor":"cursor-native"}}).to_string(),
            120,
            false,
        );
        assert!(s.contains("cursor-native"));
        assert!(s.contains("Aucune entrée"));
        let s=render("updates",&json!({"hosts":{"ksstore":{"stage":"not_reported","installed":null},"sysle2":{"stage":"stale","installed":"0.7.3"}}}).to_string(),160,false);
        assert!(s.contains("ksstore"));
        assert!(s.contains("Non signalé"));
        assert!(s.contains("Périmé"));
    }
    #[test]
    fn table225_review_regressions_preserve_authority_body_and_wide_cells() {
        for command in ["who", "discover"] {
            let a = normalize(args(&["bridget", "--table", command])).unwrap();
            assert_eq!(a.args, args(&["bridget", command]));
            assert_eq!(a.family.as_deref(), Some("agents"));
        }
        let a = normalize(args(&[
            "bridget",
            "delegate-grant",
            "target",
            "--revoke",
            "--table",
        ]))
        .unwrap();
        assert_eq!(
            a.args,
            args(&["bridget", "delegate-grant", "target", "--revoke"])
        );
        for values in [
            args(&["bridget", "send", "--to", "target", "expliquer", "--table"]),
            args(&["bridget", "reply", "expliquer", "--table"]),
            args(&[
                "bridget", "send", "--to", "target", "--", "message", "--table",
            ]),
        ] {
            let a = normalize(values.clone()).unwrap();
            assert_eq!(a.args, values);
            assert!(a.family.is_none());
        }
        let a = normalize(args(&[
            "bridget", "--table", "send", "--to", "target", "--", "message", "--table",
        ]))
        .unwrap();
        assert_eq!(
            a.args,
            args(&[
                "bridget", "send", "--to", "target", "--", "message", "--table"
            ])
        );
        let s = render("journal", "12:00:03 texte", 40, false);
        assert!(s.contains("12:00:03 texte"));
        assert!(!s.contains("│ 12 "));
        let headers = vec!["中".into(); "中".len() + 6];
        let rows = vec![vec!["中中".into(); headers.len()]];
        let s = table(&headers, &rows, 40, false);
        assert!(s.lines().all(|l| l.width() <= 40), "{s}");
    }
    #[test]
    fn table225_default_arguments_are_byte_for_byte_unchanged() {
        for values in [
            args(&["bridget", "agents", "--json"]),
            args(&["bridget", "mcp"]),
            args(&["bridget", "codex", "--", "--table"]),
            args(&["bridget", "task", "respond", "--answer", "--table"]),
        ] {
            assert_eq!(normalize(values.clone()).unwrap().args, values);
        }
    }
}
