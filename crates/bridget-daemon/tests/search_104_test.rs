//! Session 104 — recherche dans les échanges : harnais isolé et matrice S01–S31.
//!
//! Deux niveaux : (1) le moteur sur une base fixture (`search_on`/`read_on`),
//! déterministe et rapide ; (2) le daemon isolé (identité attestée, permis,
//! fils 102, MCP et CLI réels). Chaque test a son propre home, socket, base
//! et journaux sous /tmp ; aucun fournisseur réel ; arrêt SIGTERM attendu,
//! jamais SIGKILL.
#![allow(dead_code)]

#[path = "support/idempotent.rs"]
pub mod fixture;

use bridget_daemon::ledger::search::{read_on, search_on};
use bridget_transport::protocol::{
    LedgerReadOutcomeV1, LedgerReadRequest, LedgerSearchHit, LedgerSearchOutcomeV1,
    LedgerSearchPage, LedgerSearchRequest, LedgerSearchSource,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const PRODUCTION_SOCKET: &str = "/Users/user/.cache/bridget-core/bridget.sock";

pub const AGENT_A: &str = "10400000-0000-4000-8000-00000000000a";
pub const AGENT_B: &str = "10400000-0000-4000-8000-00000000000b";
pub const AGENT_C: &str = "10400000-0000-4000-8000-00000000000c";
pub const AGENT_E: &str = "10400000-0000-4000-8000-00000000000e";

// ---------------------------------------------------------------------------
// Harnais commun
// ---------------------------------------------------------------------------

pub fn spec104_root(label: &str) -> PathBuf {
    let root = test_root(label);
    assert!(root.starts_with("/tmp"), "racine hors /tmp : {root:?}");
    let socket = socket(&root);
    assert_ne!(socket, Path::new(PRODUCTION_SOCKET));
    assert!(!socket.starts_with("/Users/user/.cache/bridget-core"));
    root
}

pub fn stop_cooperatively(mut daemon: DaemonProcess) {
    signal_test_group(&mut daemon.child, libc::SIGTERM);
    let deadline = Instant::now() + Duration::from_secs(10);
    while daemon.child.try_wait().expect("état du daemon").is_none() {
        assert!(
            Instant::now() < deadline,
            "daemon isolé non terminé sur SIGTERM (10 s)"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    wait_child(&mut daemon.child, Duration::from_secs(1));
    if let Some(logs) = daemon.logs.take() {
        let _ = logs.join();
    }
}

/// Base fixture au format du daemon (schéma + index 104), sans daemon.
fn fixture_db(label: &str) -> (PathBuf, Connection) {
    let root = spec104_root(label);
    let db = root.join("state/bridget.db");
    drop(fixture_store(&db).expect("store fixture"));
    let conn = Connection::open(&db).unwrap();
    (db, conn)
}

fn insert(conn: &Connection, id: &str, ts: i64, sender: &str, target: &str, body: &str) {
    conn.execute(
        "INSERT OR REPLACE INTO ledger (id, ts, sender, target, body, conversation_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, ts, sender, target, body, format!("{sender}|{target}")],
    )
    .unwrap();
}

fn now() -> i64 {
    issued_at()
}

fn request(query: &str) -> LedgerSearchRequest {
    LedgerSearchRequest {
        source: LedgerSearchSource::Messages,
        query: query.to_string(),
        author: None,
        peer: None,
        since: None,
        until: None,
        limit: None,
        cursor: None,
        thread_id: None,
    }
}

fn page(outcome: LedgerSearchOutcomeV1) -> LedgerSearchPage {
    match outcome {
        LedgerSearchOutcomeV1::Ok(page) => page,
        LedgerSearchOutcomeV1::Error { code, reason } => panic!("refus {code} : {reason}"),
    }
}

fn error_code(outcome: &LedgerSearchOutcomeV1) -> &str {
    match outcome {
        LedgerSearchOutcomeV1::Error { code, .. } => code,
        LedgerSearchOutcomeV1::Ok(page) => panic!("succès inattendu : {page:?}"),
    }
}

fn read_code(outcome: &LedgerReadOutcomeV1) -> &str {
    match outcome {
        LedgerReadOutcomeV1::Error { code, .. } => code,
        LedgerReadOutcomeV1::Ok(fragment) => panic!("succès inattendu : {fragment:?}"),
    }
}

/// Clé physique d'un résultat message.
fn hit_key(hit: &LedgerSearchHit) -> (String, String) {
    match hit {
        LedgerSearchHit::Message { id, target, .. } => (id.clone(), target.clone()),
        LedgerSearchHit::ThreadEntry { thread_id, seq, .. } => (thread_id.clone(), seq.to_string()),
    }
}

/// Parcourt toutes les pages jusqu'à `has_more = false` ; rend les résultats
/// dans l'ordre et le nombre de pages.
fn collect_all(
    conn: &Connection,
    actor: &str,
    base: &LedgerSearchRequest,
) -> (Vec<LedgerSearchHit>, usize) {
    let mut hits = Vec::new();
    let mut cursor: Option<String> = base.cursor.clone();
    let mut pages = 0;
    loop {
        let mut request = base.clone();
        request.cursor = cursor.clone();
        let page = page(search_on(conn, actor, &request));
        pages += 1;
        assert!(pages < 10_000, "pagination sans fin");
        hits.extend(page.hits);
        if !page.has_more {
            assert!(page.next_cursor.is_none());
            assert_eq!(page.stop_reason, "exhausted");
            break;
        }
        cursor = Some(page.next_cursor.expect("curseur avec has_more"));
    }
    (hits, pages)
}

/// Repli de référence de l'oracle : indépendant du moteur (table locale).
fn oracle_fold(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            'é' | 'è' | 'ê' | 'ë' | 'É' => 'e',
            'à' | 'â' | 'ä' | 'á' | 'å' | 'À' => 'a',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ç' => 'c',
            'ÿ' | 'ý' | 'Ÿ' => 'y',
            other => other,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn oracle_matches(body: &str, terms: &[&str]) -> bool {
    let folded = oracle_fold(body);
    terms.iter().all(|term| folded.contains(&oracle_fold(term)))
}

struct Row {
    id: String,
    ts: i64,
    sender: String,
    target: String,
    body: String,
}

/// Corpus S01 : 1000 lignes où A participe, 1000 lignes tierces B↔C, corps
/// variés (accents, doublons d'identifiant, même seconde).
fn oracle_corpus(base_ts: i64) -> Vec<Row> {
    let words = [
        "pagination",
        "Erreur",
        "café",
        "décision",
        "curseur",
        "budget",
        "Ÿvette",
        "socket",
    ];
    let mut rows = Vec::new();
    for i in 0..1000usize {
        let w1 = words[i % words.len()];
        let w2 = words[(i * 7 + 3) % words.len()];
        let w3 = words[(i * 13 + 5) % words.len()];
        let body = format!("msg {i} : {w1} puis {w2} et enfin {w3} — fin {i}");
        let (sender, target) = match i % 4 {
            0 => (AGENT_A, AGENT_B),
            1 => (AGENT_B, AGENT_A),
            2 => (AGENT_A, AGENT_C),
            _ => (AGENT_C, AGENT_A),
        };
        rows.push(Row {
            // Identifiants dupliqués vers deux cibles pour 1 ligne sur 10.
            id: if i % 10 == 0 {
                format!("dup-{}", i / 20)
            } else {
                format!("m-{i}")
            },
            // Même seconde pour des paquets de 3 lignes.
            ts: base_ts - (i / 3) as i64,
            sender: sender.into(),
            target: target.into(),
            body,
        });
    }
    for i in 0..1000usize {
        let w1 = words[i % words.len()];
        rows.push(Row {
            id: format!("tiers-{i}"),
            ts: base_ts - (i / 3) as i64,
            sender: if i % 2 == 0 {
                AGENT_B.into()
            } else {
                AGENT_C.into()
            },
            target: if i % 2 == 0 {
                AGENT_C.into()
            } else {
                AGENT_B.into()
            },
            body: format!("tiers {i} : {w1} pagination erreur café"),
        });
    }
    rows
}

fn load_rows(conn: &Connection, rows: &[Row]) {
    conn.execute_batch("BEGIN").unwrap();
    for row in rows {
        insert(conn, &row.id, row.ts, &row.sender, &row.target, &row.body);
    }
    conn.execute_batch("COMMIT").unwrap();
}

// ---------------------------------------------------------------------------
// T002 — harnais isolé : daemon, inscription, arrêt sans SIGKILL
// ---------------------------------------------------------------------------

#[test]
fn spec104_harness_isolated_daemon_and_fixture() {
    let root = spec104_root("spec104-harness");
    let socket = socket(&root);
    let daemon = spawn_daemon(&root, None);
    let mut a = register_agent_as(&socket, AGENT_A, "spec104-a");
    a.send(WrapperToDaemon::LedgerSearch {
        request: request("rien"),
    });
    let DaemonToWrapper::LedgerSearchResult { outcome } = a.receive() else {
        panic!("résultat de recherche attendu");
    };
    let page = page(outcome);
    assert!(page.hits.is_empty());
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
    assert!(root.join("state/bridget.db").exists());
    stop_cooperatively(daemon);
    assert!(std::os::unix::net::UnixStream::connect(&socket).is_err());
}

// ---------------------------------------------------------------------------
// US1 — S01 oracle, S02 littéral, S03 Unicode, S04 filtres, S05 validation,
// S06 confidentialité
// ---------------------------------------------------------------------------

#[test]
fn spec104_s01_s08_oracle_corpus_exact_sans_perte_ni_doublon() {
    let (_db, conn) = fixture_db("spec104-s01");
    let base = now();
    let rows = oracle_corpus(base);
    load_rows(&conn, &rows);

    for query in [
        "pagination",
        "erreur cafe",
        "CAFÉ pagination",
        "decision budget",
        "yvette",
        "introuvable",
    ] {
        let terms: Vec<&str> = query.split_whitespace().collect();
        let expected: Vec<(String, String)> = {
            let mut own: Vec<&Row> = rows
                .iter()
                .filter(|row| row.sender == AGENT_A || row.target == AGENT_A)
                .filter(|row| oracle_matches(&row.body, &terms))
                .collect();
            // Ordre normatif : (ts, id, target) décroissants, binaire.
            own.sort_by(|x, y| {
                y.ts.cmp(&x.ts)
                    .then_with(|| y.id.as_bytes().cmp(x.id.as_bytes()))
                    .then_with(|| y.target.as_bytes().cmp(x.target.as_bytes()))
            });
            own.iter()
                .map(|row| (row.id.clone(), row.target.clone()))
                .collect()
        };
        let mut request = request(query);
        request.limit = Some(50);
        let (hits, pages) = collect_all(&conn, AGENT_A, &request);
        let got: Vec<(String, String)> = hits.iter().map(hit_key).collect();
        assert_eq!(got, expected, "oracle « {query} » ({pages} pages)");
        let unique: BTreeSet<_> = got.iter().collect();
        assert_eq!(unique.len(), got.len(), "doublon pour « {query} »");
        // Aucun tiers, jamais.
        assert!(hits.iter().all(|hit| match hit {
            LedgerSearchHit::Message { sender, target, .. } =>
                sender == AGENT_A || target == AGENT_A,
            _ => false,
        }));
    }
}

#[test]
fn spec104_s02_s03_texte_litteral_accents_et_unicode_documente() {
    let (_db, conn) = fixture_db("spec104-s02");
    let t = now();
    insert(
        &conn,
        "u1",
        t,
        AGENT_A,
        AGENT_B,
        "Le CAFÉ de Ÿvette coûte 100% _sûr_ \\ 'ok' \"dit-il\" 😀 fin",
    );
    insert(
        &conn,
        "u2",
        t - 1,
        AGENT_B,
        AGENT_A,
        "forme de\u{301}compose\u{301}e ici",
    );
    insert(&conn, "u3", t - 2, AGENT_A, AGENT_B, "áåý ÀÂÄ ñ ç");
    // Casse et accents précomposés repliés, ordre des termes indifférent.
    for query in [
        "cafe yvette",
        "YVETTE CAFÉ",
        "coute",
        "100%",
        "_sur_",
        "\\",
        "'ok'",
        "\"dit-il\"",
        "😀",
    ] {
        let p = page(search_on(&conn, AGENT_A, &request(query)));
        assert_eq!(p.hits.len(), 1, "« {query} » : {p:?}");
        assert_eq!(hit_key(&p.hits[0]).0, "u1", "« {query} »");
    }
    // Jokers LIKE inertes : « % » ne joue pas le rôle de joker.
    let p = page(search_on(&conn, AGENT_A, &request("100%X")));
    assert!(p.hits.is_empty());
    // Extrait et offset : l'occurrence est localisée dans le corps ORIGINAL.
    let p = page(search_on(&conn, AGENT_A, &request("😀")));
    let LedgerSearchHit::Message {
        match_offset,
        excerpt,
        ..
    } = &p.hits[0]
    else {
        panic!()
    };
    let body = "Le CAFÉ de Ÿvette coûte 100% _sûr_ \\ 'ok' \"dit-il\" 😀 fin";
    assert_eq!(*match_offset as usize, body.find('😀').unwrap());
    assert!(excerpt.starts_with('😀'), "{excerpt}");
    // Limite documentée : la forme décomposée n'est pas assimilée à « é ».
    let p = page(search_on(&conn, AGENT_A, &request("décomposée")));
    assert!(p.hits.is_empty(), "NFD non assimilée : {p:?}");
    let p = page(search_on(
        &conn,
        AGENT_A,
        &request("de\u{301}compose\u{301}e"),
    ));
    assert_eq!(p.hits.len(), 1);
    assert!(p.notices.iter().any(|n| n.starts_with("unicode:")));
    // á/å/ý et majuscules accentuées repliées.
    let p = page(search_on(&conn, AGENT_A, &request("aay aaa n c")));
    assert_eq!(p.hits.len(), 1);
}

#[test]
fn spec104_s04_filtres_auteur_correspondant_dates_inclusives() {
    let (_db, conn) = fixture_db("spec104-s04");
    let t = now();
    insert(&conn, "f1", t, AGENT_A, AGENT_B, "mot");
    insert(&conn, "f2", t - 10, AGENT_B, AGENT_A, "mot");
    insert(&conn, "f3", t - 20, AGENT_C, AGENT_A, "mot");
    insert(&conn, "f4", t - 20, AGENT_A, AGENT_A, "mot à soi");
    let keys = |page: LedgerSearchPage| {
        page.hits
            .iter()
            .map(|hit| hit_key(hit).0)
            .collect::<Vec<_>>()
    };
    let mut r = request("mot");
    r.author = Some(AGENT_B.into());
    assert_eq!(keys(page(search_on(&conn, AGENT_A, &r))), ["f2"]);
    let mut r = request("mot");
    r.peer = Some(AGENT_C.into());
    assert_eq!(keys(page(search_on(&conn, AGENT_A, &r))), ["f3"]);
    let mut r = request("mot");
    r.peer = Some(AGENT_A.into());
    assert_eq!(
        keys(page(search_on(&conn, AGENT_A, &r))),
        ["f4"],
        "message à soi-même"
    );
    // Bornes inclusives, même seconde.
    let mut r = request("mot");
    r.since = Some(t - 20);
    r.until = Some(t - 10);
    // Même seconde : ordre binaire décroissant des identifiants (f4 avant f3).
    assert_eq!(
        keys(page(search_on(&conn, AGENT_A, &r))),
        ["f2", "f4", "f3"]
    );
    let mut r = request("mot");
    r.since = Some(t - 20);
    r.until = Some(t - 20);
    assert_eq!(keys(page(search_on(&conn, AGENT_A, &r))), ["f4", "f3"]);
    // Filtre sans occurrence : succès vide, exhausted, aucun curseur.
    let mut r = request("mot");
    r.since = Some(t + 100);
    let p = page(search_on(&conn, AGENT_A, &r));
    assert!(p.hits.is_empty() && !p.has_more && p.next_cursor.is_none());
    // Les rejets par filtre comptent comme candidats traités.
    let mut r = request("mot");
    r.author = Some(AGENT_C.into());
    let p = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p.hits.len(), 1);
    assert_eq!(p.scanned_count, 4);
    assert_eq!(p.scanned_bytes, "mot".len() as u64);
}

#[test]
fn spec104_s05_validation_sans_sql() {
    // Connexion mémoire SANS table ledger : toute requête SQL échouerait en
    // storage_unavailable ; un refus invalid_params prouve l'arrêt avant SQL.
    let conn = Connection::open_in_memory().unwrap();
    let cases: Vec<(LedgerSearchRequest, &str)> = vec![
        (request(""), "vide"),
        (request("   \t"), "blanc"),
        (request(&"a".repeat(257)), "257 octets"),
        (request("a b c d e f g h i"), "9 termes"),
        (
            {
                let mut r = request("x");
                r.limit = Some(0);
                r
            },
            "limit 0",
        ),
        (
            {
                let mut r = request("x");
                r.limit = Some(51);
                r
            },
            "limit 51",
        ),
        (
            {
                let mut r = request("x");
                r.since = Some(10);
                r.until = Some(5);
                r
            },
            "dates inversées",
        ),
        (
            {
                let mut r = request("x");
                r.since = Some(-1);
                r
            },
            "since négatif",
        ),
        (
            {
                let mut r = request("x");
                r.peer = Some("pas-un-uuid".into());
                r
            },
            "faux UUID",
        ),
        (
            {
                let mut r = request("x");
                r.author = Some(AGENT_A.to_uppercase());
                r
            },
            "UUID non canonique",
        ),
        (
            {
                let mut r = request("x");
                r.thread_id = Some(AGENT_A.into());
                r
            },
            "thread_id sans source thread",
        ),
        (
            {
                let mut r = request("x");
                r.source = LedgerSearchSource::Thread;
                r
            },
            "source thread sans thread_id",
        ),
        (
            {
                let mut r = request("x");
                r.source = LedgerSearchSource::Thread;
                r.thread_id = Some(AGENT_A.into());
                r.peer = Some(AGENT_B.into());
                r
            },
            "peer avec thread",
        ),
    ];
    for (request, label) in cases {
        let outcome = search_on(&conn, AGENT_A, &request);
        assert_eq!(error_code(&outcome), "invalid_params", "{label}");
    }
    // 256 octets et 8 termes passent la validation (puis SQL échoue : table absente).
    let outcome = search_on(&conn, AGENT_A, &request(&"a ".repeat(8)));
    assert_eq!(error_code(&outcome), "storage_unavailable");
    let outcome = search_on(&conn, AGENT_A, &request(&"a".repeat(256)));
    assert_eq!(error_code(&outcome), "storage_unavailable");
    // Relecture : mêmes garanties.
    let read = |id: &str, target: &str, offset: u64, digest: Option<&str>| {
        read_on(
            &conn,
            AGENT_A,
            &LedgerReadRequest {
                id: id.into(),
                target: target.into(),
                offset,
                digest: digest.map(str::to_string),
            },
        )
    };
    assert_eq!(read_code(&read("", "t", 0, None)), "invalid_params");
    assert_eq!(
        read_code(&read(&"i".repeat(257), "t", 0, None)),
        "invalid_params"
    );
    assert_eq!(
        read_code(&read("i", "t", 5, None)),
        "invalid_params",
        "digest requis si offset > 0"
    );
    assert_eq!(read_code(&read("i", "t", 0, Some("ZZ"))), "invalid_params");
    assert_eq!(read_code(&read("i", "t", 0, None)), "storage_unavailable");
}

#[test]
fn spec104_s06_confidentialite_participant_et_compteurs() {
    let (_db, conn) = fixture_db("spec104-s06");
    let t = now();
    insert(&conn, "p1", t, AGENT_A, AGENT_B, "secret partagé AB");
    insert(&conn, "p2", t - 1, AGENT_B, AGENT_C, "secret BC");
    insert(&conn, "p3", t - 2, AGENT_C, AGENT_B, "secret CB");
    let p = page(search_on(&conn, AGENT_A, &request("secret")));
    assert_eq!(p.hits.len(), 1);
    assert_eq!(hit_key(&p.hits[0]).0, "p1");
    // Les compteurs ne trahissent pas l'existence des échanges B↔C.
    assert_eq!(p.scanned_count, 1);
    assert_eq!(p.scanned_bytes, "secret partagé AB".len() as u64);
    let p = page(search_on(&conn, AGENT_E, &request("secret")));
    assert!(p.hits.is_empty() && p.scanned_count == 0 && p.next_cursor.is_none());
    // Relecture : une clé tierce et une clé inexistante donnent la même erreur.
    let r = |id: &str, target: &str| {
        read_on(
            &conn,
            AGENT_A,
            &LedgerReadRequest {
                id: id.into(),
                target: target.into(),
                offset: 0,
                digest: None,
            },
        )
    };
    assert_eq!(read_code(&r("p2", AGENT_C)), "not_found_or_forbidden");
    assert_eq!(read_code(&r("inconnu", AGENT_C)), "not_found_or_forbidden");
}

// ---------------------------------------------------------------------------
// US2 — S07 page vide reprenable, S09 bornes de travail, S10 bornes de sortie,
// S11/S12 curseur, S13 corpus vivant, S14 erreur SQLite, S15 métadonnées
// ---------------------------------------------------------------------------

#[test]
fn spec104_s07_page_vide_avec_curseur_puis_occurrence() {
    let (_db, conn) = fixture_db("spec104-s07");
    let t = now();
    for i in 0..128 {
        insert(
            &conn,
            &format!("n-{i:03}"),
            t - i,
            AGENT_A,
            AGENT_B,
            "rien à voir",
        );
    }
    insert(&conn, "cible", t - 500, AGENT_B, AGENT_A, "la perle rare");
    let p1 = page(search_on(&conn, AGENT_A, &request("perle")));
    assert!(p1.hits.is_empty(), "{p1:?}");
    assert!(p1.has_more);
    assert_eq!(p1.stop_reason, "scan_budget");
    assert_eq!(p1.scanned_count, 128);
    let cursor = p1.next_cursor.clone().expect("curseur malgré page vide");
    let mut r = request("perle");
    r.cursor = Some(cursor);
    let p2 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p2.hits.len(), 1);
    assert_eq!(hit_key(&p2.hits[0]).0, "cible");
    assert!(!p2.has_more);
    assert_eq!(p2.stop_reason, "exhausted");
    // Limite atteinte sur la DERNIÈRE ligne existante : exhausted prime.
    let mut r = request("perle");
    r.limit = Some(1);
    r.cursor = p1.next_cursor.clone();
    let p3 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p3.hits.len(), 1);
    assert!(!p3.has_more && p3.next_cursor.is_none());
    assert_eq!(p3.stop_reason, "exhausted");
    // Limite atteinte AVANT la dernière ligne : result_limit, la ligne suivante
    // n'est pas acquittée et se retrouve en tête de la page suivante.
    insert(
        &conn,
        "cible2",
        t - 501,
        AGENT_B,
        AGENT_A,
        "la perle rare bis",
    );
    let mut r = request("perle");
    r.limit = Some(1);
    r.cursor = p1.next_cursor.clone();
    let p4 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(hit_key(&p4.hits[0]).0, "cible");
    assert_eq!(p4.stop_reason, "result_limit");
    assert_eq!(p4.scanned_count, 1);
    r.cursor = p4.next_cursor.clone();
    let p5 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(hit_key(&p5.hits[0]).0, "cible2");
    assert!(!p5.has_more);
}

#[test]
fn spec104_s09_bornes_de_travail_octets_et_corps_geants() {
    let (_db, conn) = fixture_db("spec104-s09");
    let t = now();
    // 300 Kio × 5 : la sélection s'arrête quand 1 Mio est atteint, avec une
    // seule ligne entière au-delà du budget restant.
    let big = "x".repeat(300 * 1024) + " aiguille";
    for i in 0..5 {
        insert(&conn, &format!("b-{i}"), t - i, AGENT_A, AGENT_B, &big);
    }
    let p = page(search_on(&conn, AGENT_A, &request("aiguille")));
    assert_eq!(p.hits.len(), 4, "{}", p.stop_reason);
    assert_eq!(p.stop_reason, "byte_budget");
    assert!(p.scanned_bytes >= 1 << 20 && p.scanned_bytes < (1 << 20) + big.len() as u64 + 1);
    assert!(p.has_more);
    let mut r = request("aiguille");
    r.cursor = p.next_cursor.clone();
    let p2 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p2.hits.len(), 1);
    assert!(!p2.has_more);

    // Corps > 16 Mio : ignoré, signalé, jamais chargé ; un corps de 16 Mio
    // exactement est admissible.
    let (_db, conn) = fixture_db("spec104-s09b");
    conn.execute(
        "INSERT INTO ledger (id, ts, sender, target, body, conversation_key)
         VALUES ('geant', ?1, ?2, ?3, zeroblob(0) || ?4, 'k')",
        params![t, AGENT_A, AGENT_B, "y".repeat((16 << 20) + 1)],
    )
    .unwrap();
    insert(
        &conn,
        "limite",
        t - 1,
        AGENT_A,
        AGENT_B,
        &("z".repeat((16 << 20) - 8) + " trouvé"),
    );
    insert(&conn, "petit", t - 2, AGENT_A, AGENT_B, "petit trouvé");
    let p = page(search_on(&conn, AGENT_A, &request("trouvé")));
    assert_eq!(p.skipped_oversized, 1, "{p:?}");
    assert!(
        p.notices
            .iter()
            .any(|n| n.starts_with("oversized_source: 1 "))
    );
    let keys: Vec<String> = p.hits.iter().map(|h| hit_key(h).0).collect();
    assert_eq!(
        keys,
        ["limite"],
        "un corps de 16 Mio est traité entier puis arrêt"
    );
    assert_eq!(p.stop_reason, "byte_budget");
    let mut r = request("trouvé");
    r.cursor = p.next_cursor.clone();
    let p2 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(
        p2.hits.iter().map(|h| hit_key(h).0).collect::<Vec<_>>(),
        ["petit"]
    );
    assert!(
        p2.scanned_bytes < 100,
        "le corps géant n'a jamais été chargé"
    );
}

#[test]
fn spec104_s10_bornes_de_sortie_extraits_et_reponse() {
    let (_db, conn) = fixture_db("spec104-s10");
    let t = now();
    // 128 corps de 8 Kio dont l'occurrence est suivie d'un long texte : chaque
    // extrait fait 512 octets ; 50 résultats × ~0,8 Kio < 60 Kio.
    let long = "clé ".to_string() + &"é".repeat(4000);
    for i in 0..128 {
        insert(&conn, &format!("o-{i:03}"), t - i, AGENT_A, AGENT_B, &long);
    }
    let mut r = request("clé");
    r.limit = Some(50);
    let p = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p.hits.len(), 50);
    assert_eq!(p.stop_reason, "result_limit");
    for hit in &p.hits {
        let LedgerSearchHit::Message {
            excerpt,
            body_bytes,
            ..
        } = hit
        else {
            panic!()
        };
        assert!(excerpt.len() <= 512 && std::str::from_utf8(excerpt.as_bytes()).is_ok());
        assert!(
            excerpt.len() >= 511,
            "extrait à frontière UTF-8 : {}",
            excerpt.len()
        );
        assert_eq!(*body_bytes, long.len() as u64);
    }
    let bytes = serde_json::to_vec(&LedgerSearchOutcomeV1::Ok(p.clone()))
        .unwrap()
        .len();
    assert!(bytes <= 61_440, "{bytes}");
    // Budget de sortie : identifiants et cibles longs (≤ 256 octets) font
    // grossir chaque résultat ; la page s'arrête avant 60 Kio et le résultat
    // non inclus n'est pas consommé.
    let (_db, conn) = fixture_db("spec104-s10b");
    let long_id = "i".repeat(250);
    let long_target = "t".repeat(250);
    for i in 0..128 {
        insert(
            &conn,
            &format!("{long_id}{i:03}"),
            t - i,
            AGENT_A,
            &format!("{long_target}{i:03}"),
            &long,
        );
    }
    let mut r = request("clé");
    r.limit = Some(50);
    let p = page(search_on(&conn, AGENT_A, &r));
    let bytes = serde_json::to_vec(&LedgerSearchOutcomeV1::Ok(p.clone()))
        .unwrap()
        .len();
    assert!(bytes <= 61_440, "{bytes}");
    assert!(
        p.hits.len() < 50 && p.has_more,
        "{} résultats, {}",
        p.hits.len(),
        p.stop_reason
    );
    assert_eq!(p.stop_reason, "response_budget");
    assert_eq!(
        p.scanned_count as usize,
        p.hits.len(),
        "le résultat refusé n'est pas acquitté"
    );
    let (all, _) = collect_all(&conn, AGENT_A, &r);
    assert_eq!(all.len(), 128, "aucune perte sur les pages suivantes");
    let unique: BTreeSet<_> = all.iter().map(hit_key).collect();
    assert_eq!(unique.len(), 128);
}

#[test]
fn spec104_s11_s12_curseur_sans_pouvoir_et_malforme() {
    let (_db, conn) = fixture_db("spec104-s11");
    let t = now();
    for i in 0..130 {
        insert(
            &conn,
            &format!("a-{i:03}"),
            t - i,
            AGENT_A,
            AGENT_B,
            "commun",
        );
    }
    for i in 0..130 {
        insert(
            &conn,
            &format!("c-{i:03}"),
            t - i,
            AGENT_C,
            AGENT_B,
            "commun",
        );
    }
    let p = page(search_on(&conn, AGENT_A, &request("commun")));
    let cursor = p.next_cursor.clone().expect("suite");
    // Le curseur de A chez C est refusé ; celui de A avec d'autres filtres aussi.
    let mut r = request("commun");
    r.cursor = Some(cursor.clone());
    assert_eq!(error_code(&search_on(&conn, AGENT_C, &r)), "invalid_cursor");
    let mut r = request("commun ");
    r.cursor = Some(cursor.clone());
    assert_eq!(
        error_code(&search_on(&conn, AGENT_A, &r)),
        "invalid_cursor",
        "query textuellement différente"
    );
    let mut r = request("commun");
    r.cursor = Some(cursor.clone());
    r.author = Some(AGENT_A.into());
    assert_eq!(error_code(&search_on(&conn, AGENT_A, &r)), "invalid_cursor");
    // limit peut changer entre deux pages.
    let mut r = request("commun");
    r.cursor = Some(cursor.clone());
    r.limit = Some(3);
    assert_eq!(page(search_on(&conn, AGENT_A, &r)).hits.len(), 3);
    // Curseur forgé : borne « before » déplacée sur une clé tierce ; aucune
    // ligne de C n'apparaît, quelle que soit la borne.
    let bytes = (0..cursor.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cursor[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let mut decoded: Value = serde_json::from_slice(&bytes).unwrap();
    decoded["before"] = json!({"ts": t + 10, "id": "c-000", "target": AGENT_B});
    decoded["upper"] = json!({"ts": t + 10, "id": "zzz", "target": "zzz"});
    let forged: String = serde_json::to_vec(&decoded)
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut r = request("commun");
    r.cursor = Some(forged);
    let p = page(search_on(&conn, AGENT_A, &r));
    assert!(
        p.hits.iter().all(|h| hit_key(h).0.starts_with("a-")),
        "{p:?}"
    );
    // Malformés : hex invalide, majuscules, version future, trop long, tuple
    // incomplet, before après upper, actor différent.
    let mut cases: Vec<(String, &str)> = vec![
        ("zz".into(), "hex invalide"),
        (cursor.to_uppercase(), "hex majuscule"),
        ("0".repeat(16_386), "trop long"),
        ("".into(), "vide"),
    ];
    let forge = |mutate: &dyn Fn(&mut Value)| -> String {
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        mutate(&mut value);
        serde_json::to_vec(&value)
            .unwrap()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    };
    cases.push((forge(&|v| v["v"] = json!(2)), "version future"));
    cases.push((forge(&|v| v["actor"] = json!(AGENT_C)), "acteur différent"));
    cases.push((
        forge(&|v| v["before"] = json!({"ts": 1, "id": "x"})),
        "tuple incomplet",
    ));
    cases.push((
        forge(&|v| {
            v["before"] = v["upper"].clone();
            v["upper"] = json!({"ts": 0, "id": "a", "target": "b"});
        }),
        "before après upper",
    ));
    cases.push((
        forge(&|v| v["before"] = json!({"ts": 1, "id": "x".repeat(257), "target": "b"})),
        "id > 256",
    ));
    cases.push((
        forge(&|v| v["source"] = json!("thread")),
        "source différente",
    ));
    cases.push((forge(&|v| v["extra"] = json!(1)), "champ inconnu"));
    cases.push((
        forge(&|v| v["fingerprint"] = json!("00")),
        "empreinte différente",
    ));
    for (cursor, label) in cases {
        let mut r = request("commun");
        r.cursor = Some(cursor);
        assert_eq!(
            error_code(&search_on(&conn, AGENT_A, &r)),
            "invalid_cursor",
            "{label}"
        );
    }
}

#[test]
fn spec104_s13_corpus_vivant_entre_deux_pages() {
    let (_db, conn) = fixture_db("spec104-s13");
    let t = now();
    for i in 0..200 {
        insert(
            &conn,
            &format!("v-{i:03}"),
            t - i,
            AGENT_A,
            AGENT_B,
            "vivant",
        );
    }
    let mut r = request("vivant");
    r.limit = Some(50);
    let p1 = page(search_on(&conn, AGENT_A, &r));
    assert_eq!(p1.hits.len(), 50);
    assert_eq!(p1.consistency, "live_bounded");
    // Insertion plus récente : hors de la borne haute de la première page.
    insert(&conn, "nouveau", t + 5, AGENT_A, AGENT_B, "vivant nouveau");
    // Purge d'une ligne à venir et remplacement du corps d'une autre.
    conn.execute("DELETE FROM ledger WHERE id = 'v-060'", [])
        .unwrap();
    insert(
        &conn,
        "v-070",
        t - 70,
        AGENT_A,
        AGENT_B,
        "remplacé sans le mot",
    );
    r.cursor = p1.next_cursor.clone();
    let (rest, _) = collect_all(&conn, AGENT_A, &r);
    let keys: BTreeSet<String> = rest.iter().map(|h| hit_key(h).0).collect();
    assert!(!keys.contains("nouveau"), "borne haute stable");
    assert!(!keys.contains("v-060") && !keys.contains("v-070"));
    assert_eq!(rest.len(), 150 - 2);
    // Une nouvelle recherche voit la ligne récente.
    let p = page(search_on(&conn, AGENT_A, &request("vivant nouveau")));
    assert_eq!(hit_key(&p.hits[0]).0, "nouveau");
    // Source purgée : relecture honnête.
    let read = read_on(
        &conn,
        AGENT_A,
        &LedgerReadRequest {
            id: "v-060".into(),
            target: AGENT_B.into(),
            offset: 0,
            digest: None,
        },
    );
    assert_eq!(read_code(&read), "not_found_or_forbidden");
}

#[test]
fn spec104_s14_erreur_sqlite_jamais_resultat_vide() {
    let (db, conn) = fixture_db("spec104-s14");
    insert(&conn, "e1", now(), AGENT_A, AGENT_B, "présent");
    let ro = bridget_daemon::store::Store::open_read_only(&db).unwrap();
    // Session 107 : la base est en WAL. Un écrivain en transaction, même avec
    // une écriture en attente, ne bloque plus le lecteur : la recherche répond.
    let writer = Connection::open(&db).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    insert(&writer, "e2", now() - 1, AGENT_A, AGENT_B, "présent aussi");
    assert_eq!(
        page(search_on(&ro, AGENT_A, &request("présent")))
            .hits
            .len(),
        1,
        "lecture pendant une écriture ouverte : état validé précédent"
    );
    writer.execute_batch("COMMIT").unwrap();
    assert_eq!(
        page(search_on(&ro, AGENT_A, &request("présent")))
            .hits
            .len(),
        2
    );
    // Un vrai SQLITE_BUSY reste possible : une connexion en verrouillage
    // exclusif persistant (`locking_mode=EXCLUSIVE`) exclut tout autre accès.
    // Elle exige qu'aucune autre connexion WAL ne soit ouverte : on referme
    // les nôtres, puis un lecteur neuf subit le refus. La lecture bornée
    // (100 ms) échoue alors en storage_unavailable, jamais en hits=[].
    drop(ro);
    drop(writer);
    drop(conn);
    let exclusive = Connection::open(&db).unwrap();
    exclusive
        .execute_batch("PRAGMA locking_mode=EXCLUSIVE; BEGIN IMMEDIATE;")
        .unwrap();
    insert(
        &exclusive,
        "e3",
        now() - 2,
        AGENT_A,
        AGENT_B,
        "présent encore",
    );
    let ro = bridget_daemon::store::Store::open_read_only(&db).unwrap();
    let started = Instant::now();
    assert_eq!(
        error_code(&search_on(&ro, AGENT_A, &request("présent"))),
        "storage_unavailable"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    exclusive.execute_batch("ROLLBACK").unwrap();
    drop(exclusive);
    let conn = Connection::open(&db).unwrap();
    assert_eq!(
        page(search_on(&ro, AGENT_A, &request("présent")))
            .hits
            .len(),
        2
    );
    // Ligne indécodable (corps BLOB non UTF-8) : erreur propagée, pas ignorée.
    conn.execute(
        "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES ('bad', ?1, ?2, ?3, X'FFFE', 'k')",
        params![now() - 1, AGENT_A, AGENT_B],
    )
    .unwrap();
    assert_eq!(
        error_code(&search_on(&ro, AGENT_A, &request("présent"))),
        "storage_unavailable"
    );
    // Table absente : idem.
    conn.execute_batch("DROP TABLE ledger").unwrap();
    assert_eq!(
        error_code(&search_on(&ro, AGENT_A, &request("présent"))),
        "storage_unavailable"
    );
}

#[test]
fn spec104_s15_metadonnees_heritees_trop_longues() {
    let (_db, conn) = fixture_db("spec104-s15");
    let t = now();
    insert(&conn, "ok-1", t, AGENT_A, AGENT_B, "mot");
    insert(&conn, &"h".repeat(257), t - 1, AGENT_A, AGENT_B, "mot");
    let outcome = search_on(&conn, AGENT_A, &request("mot"));
    assert_eq!(error_code(&outcome), "source_metadata_too_large");
    if let LedgerSearchOutcomeV1::Error { reason, .. } = &outcome {
        assert!(!reason.contains("hhhh"), "aucune valeur dans l'erreur");
    }
    // L'agent restreint ses dates : la ligne héritée sort de la fenêtre.
    let mut r = request("mot");
    r.since = Some(t);
    assert_eq!(page(search_on(&conn, AGENT_A, &r)).hits.len(), 1);
    // Lecture d'une clé longue : invalid_params avant SQL.
    let read = read_on(
        &conn,
        AGENT_A,
        &LedgerReadRequest {
            id: "h".repeat(257),
            target: AGENT_B.into(),
            offset: 0,
            digest: None,
        },
    );
    assert_eq!(read_code(&read), "invalid_params");
}

// ---------------------------------------------------------------------------
// US3 — S16 lecture exacte, S17 fragments, S18 corps changé, S30 passage utile
// ---------------------------------------------------------------------------

fn read(
    conn: &Connection,
    actor: &str,
    id: &str,
    target: &str,
    offset: u64,
    digest: Option<&str>,
) -> LedgerReadOutcomeV1 {
    read_on(
        conn,
        actor,
        &LedgerReadRequest {
            id: id.into(),
            target: target.into(),
            offset,
            digest: digest.map(str::to_string),
        },
    )
}

#[test]
fn spec104_s16_s17_s18_lecture_exacte_fragments_et_changement() {
    let (_db, conn) = fixture_db("spec104-s16");
    let t = now();
    insert(&conn, "same", t, AGENT_A, AGENT_B, "corps pour B");
    insert(&conn, "same", t, AGENT_A, AGENT_C, "corps pour C");
    let LedgerReadOutcomeV1::Ok(f) = read(&conn, AGENT_A, "same", AGENT_B, 0, None) else {
        panic!()
    };
    assert_eq!(f.fragment, "corps pour B");
    assert_eq!(f.body_bytes, 12);
    assert!(f.next_offset.is_none());
    assert_eq!(f.digest.len(), 64);
    // B lit sa copie, pas celle de C ; C idem ; E rien.
    let LedgerReadOutcomeV1::Ok(f) = read(&conn, AGENT_B, "same", AGENT_B, 0, None) else {
        panic!()
    };
    assert_eq!(f.fragment, "corps pour B");
    assert_eq!(
        read_code(&read(&conn, AGENT_B, "same", AGENT_C, 0, None)),
        "not_found_or_forbidden"
    );
    assert_eq!(
        read_code(&read(&conn, AGENT_E, "same", AGENT_B, 0, None)),
        "not_found_or_forbidden"
    );

    // Fragments : 64 Kio avec un emoji à cheval sur la limite de 16 Kio.
    let mut body = "a".repeat(16_382);
    body.push('😀');
    body.push_str(&"b".repeat(65_536 - body.len()));
    insert(&conn, "frag", t - 1, AGENT_B, AGENT_A, &body);
    let mut rebuilt = Vec::new();
    let mut offset = 0u64;
    let mut digest: Option<String> = None;
    let mut pieces = 0;
    loop {
        let LedgerReadOutcomeV1::Ok(f) =
            read(&conn, AGENT_A, "frag", AGENT_A, offset, digest.as_deref())
        else {
            panic!()
        };
        assert!(f.fragment.len() <= 16_384);
        assert_eq!(f.body_bytes, body.len() as u64);
        if let Some(previous) = &digest {
            assert_eq!(previous, &f.digest);
        }
        digest = Some(f.digest.clone());
        rebuilt.extend_from_slice(f.fragment.as_bytes());
        pieces += 1;
        match f.next_offset {
            Some(next) => {
                assert!(body.is_char_boundary(next as usize));
                offset = next;
            }
            None => break,
        }
    }
    assert_eq!(rebuilt, body.as_bytes(), "reconstruction exacte");
    assert_eq!(pieces, 5, "16382 + 4 (emoji) puis 16384 …");
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(body.as_bytes()))
    };
    assert_eq!(digest.as_deref(), Some(sha.as_str()));
    // offset = body_bytes : fragment vide et fin ; au-delà ou hors frontière : refus.
    let LedgerReadOutcomeV1::Ok(f) = read(
        &conn,
        AGENT_A,
        "frag",
        AGENT_A,
        body.len() as u64,
        Some(&sha),
    ) else {
        panic!()
    };
    assert!(f.fragment.is_empty() && f.next_offset.is_none());
    assert_eq!(
        read_code(&read(
            &conn,
            AGENT_A,
            "frag",
            AGENT_A,
            body.len() as u64 + 1,
            Some(&sha)
        )),
        "invalid_params"
    );
    assert_eq!(
        read_code(&read(&conn, AGENT_A, "frag", AGENT_A, 16_383, Some(&sha))),
        "invalid_params",
        "milieu d'un emoji"
    );

    // Corps remplacé avant le second morceau : content_changed sans morceau.
    insert(&conn, "frag", t - 1, AGENT_B, AGENT_A, "nouvelle version");
    let outcome = read(&conn, AGENT_A, "frag", AGENT_A, 16_384, Some(&sha));
    assert_eq!(read_code(&outcome), "content_changed");
    if let LedgerReadOutcomeV1::Error { reason, .. } = &outcome {
        assert!(
            !reason.contains("nouvelle version"),
            "aucun contenu dans le refus"
        );
    }
    // Digest fourni à offset 0 : vérifié aussi.
    assert_eq!(
        read_code(&read(&conn, AGENT_A, "frag", AGENT_A, 0, Some(&sha))),
        "content_changed"
    );
    // Corps > 16 Mio : source_too_large.
    conn.execute(
        "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES ('huge', ?1, ?2, ?3, ?4, 'k')",
        params![t - 2, AGENT_A, AGENT_B, "h".repeat((16 << 20) + 1)],
    )
    .unwrap();
    assert_eq!(
        read_code(&read(&conn, AGENT_A, "huge", AGENT_B, 0, None)),
        "source_too_large"
    );
}

#[test]
fn spec104_s30_passage_utile_unicode_offset_original_et_lecture_ciblee() {
    let (_db, conn) = fixture_db("spec104-s30");
    // Préfixe dont le repli change la longueur (Ÿ → y, 2 → 1 octet ; É → e),
    // puis le terme tout à la fin.
    let prefix = "ŸÉŸÉ ".repeat(2000);
    let body = format!("{prefix}la décision finale");
    insert(&conn, "u", now(), AGENT_A, AGENT_B, &body);
    let p = page(search_on(&conn, AGENT_A, &request("DECISION finale")));
    let LedgerSearchHit::Message {
        match_offset,
        excerpt,
        body_digest,
        ..
    } = &p.hits[0]
    else {
        panic!()
    };
    assert_eq!(*match_offset as usize, body.find("décision").unwrap());
    assert_eq!(excerpt, "décision finale");
    // Lecture ciblée depuis l'offset avec l'empreinte : pas de relecture du préfixe.
    let LedgerReadOutcomeV1::Ok(f) = read(
        &conn,
        AGENT_A,
        "u",
        AGENT_B,
        *match_offset,
        Some(body_digest),
    ) else {
        panic!()
    };
    assert_eq!(f.fragment, "décision finale");
    assert!(f.next_offset.is_none());
}

// ===========================================================================
// Niveau daemon : identité attestée, fils 102, non-mutation, permis,
// performance, parité CLI/MCP, injection.
// ===========================================================================

pub fn op() -> String {
    uuid::Uuid::new_v4().hyphenated().to_string()
}

pub fn thread(client: &mut Client, action: Value) -> Value {
    let request: bridget_transport::protocol::ThreadAction =
        serde_json::from_value(action).expect("action de fil valide");
    client.send(WrapperToDaemon::ThreadRequest {
        request: bridget_transport::protocol::ThreadRequest {
            version: bridget_transport::protocol::THREAD_CONTRACT_VERSION,
            request,
        },
    });
    match client.receive() {
        DaemonToWrapper::ThreadResult { result } => result.result,
        other => panic!("résultat de fil attendu, reçu {other:?}"),
    }
}

fn create_thread(client: &mut Client, title: &str, members: &[&str]) -> String {
    let created = thread(
        client,
        json!({"action":"create","title":title,"members":members,"operation_id":op()}),
    );
    assert_eq!(created["status"], "created", "{created}");
    created["thread_id"].as_str().unwrap().to_string()
}

fn post(client: &mut Client, thread_id: &str, body: &str) -> u64 {
    let posted = thread(
        client,
        json!({"action":"post","thread_id":thread_id,"body":body,"notify":[],"operation_id":op()}),
    );
    assert_eq!(posted["status"], "posted", "{posted}");
    posted["seq"].as_u64().unwrap()
}

fn search_via(client: &mut Client, request: LedgerSearchRequest) -> LedgerSearchOutcomeV1 {
    client.send(WrapperToDaemon::LedgerSearch { request });
    match client.receive() {
        DaemonToWrapper::LedgerSearchResult { outcome } => outcome,
        other => panic!("résultat de recherche attendu, reçu {other:?}"),
    }
}

fn read_via(client: &mut Client, request: LedgerReadRequest) -> LedgerReadOutcomeV1 {
    client.send(WrapperToDaemon::LedgerRead { request });
    match client.receive() {
        DaemonToWrapper::LedgerReadResult { outcome } => outcome,
        other => panic!("résultat de relecture attendu, reçu {other:?}"),
    }
}

fn thread_request(thread_id: &str, query: &str) -> LedgerSearchRequest {
    let mut r = request(query);
    r.source = LedgerSearchSource::Thread;
    r.thread_id = Some(thread_id.to_string());
    r
}

/// CLI réelle avec l'identité attestée d'un agent déjà inscrit (preuve privée).
fn cli_as(root: &Path, agent_id: &str, instance_id: &str, args: &[&str]) -> std::process::Output {
    let name_file = root.join(format!("state/name-{instance_id}"));
    private_write(&name_file, agent_id).unwrap();
    let mut command = isolated_command(root);
    command
        .env("BRIDGET_AGENT_ID_FILE", &name_file)
        .env("BRIDGET_AGENT_INSTANCE_ID", instance_id)
        .args(args);
    run_command(command)
}

fn output_text(output: &std::process::Output) -> String {
    format!(
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn table_dump(conn: &Connection, table: &str) -> Vec<String> {
    let mut statement = conn
        .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
        .unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            Ok((0..columns)
                .map(|index| format!("{:?}", row.get_ref(index).unwrap()))
                .collect::<Vec<_>>()
                .join("|"))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Insère directement dans la base du daemon (avant son démarrage).
fn seed_db(root: &Path) -> Connection {
    let db = root.join("state/bridget.db");
    drop(fixture_store(&db).expect("store fixture"));
    std::fs::set_permissions(&db, std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();
    Connection::open(&db).unwrap()
}

struct Trio {
    root: PathBuf,
    socket: PathBuf,
    daemon: Option<DaemonProcess>,
    a: Client,
    b: Client,
    e: Client,
}

impl Trio {
    fn start(label: &str, seed: impl FnOnce(&Connection)) -> Self {
        Self::start_with(label, seed, |root| spawn_daemon(root, None))
    }

    fn start_with(
        label: &str,
        seed: impl FnOnce(&Connection),
        spawn: impl FnOnce(&Path) -> DaemonProcess,
    ) -> Self {
        let root = spec104_root(label);
        let socket = socket(&root);
        seed(&seed_db(&root));
        let daemon = spawn(&root);
        let a = register_agent_as(&socket, AGENT_A, "spec104-a");
        let b = register_agent_as(&socket, AGENT_B, "spec104-b");
        let e = register_agent_as(&socket, AGENT_E, "spec104-e");
        Self {
            root,
            socket,
            daemon: Some(daemon),
            a,
            b,
            e,
        }
    }
    fn stop(&mut self) {
        if let Some(daemon) = self.daemon.take() {
            stop_cooperatively(daemon);
        }
    }
}

// ---------------------------------------------------------------------------
// S06 (identité) — connexion non attestée refusée avant SQL
// ---------------------------------------------------------------------------

#[test]
fn spec104_s06_identite_absente_ou_fichier_seul_refusee() {
    let mut trio = Trio::start("spec104-identite", |conn| {
        insert(conn, "x", now(), AGENT_A, AGENT_B, "présent");
    });
    // Connexion brute sans Register ni preuve auxiliaire : aucune identité vivante.
    let mut anonymous = Client::connect(&trio.socket);
    let outcome = search_via(&mut anonymous, request("présent"));
    assert_eq!(error_code(&outcome), "identity_unavailable");
    let outcome = read_via(
        &mut anonymous,
        LedgerReadRequest {
            id: "x".into(),
            target: AGENT_B.into(),
            offset: 0,
            digest: None,
        },
    );
    assert_eq!(read_code(&outcome), "identity_unavailable");
    // Un fichier d'identité sans preuve ne suffit pas à la CLI : refus, pas de
    // résultat emprunté à A.
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "instance-sans-preuve",
        &["ledger", "search", "--query", "présent", "--json"],
    );
    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("\"hits\""),
        "{}",
        output_text(&output)
    );
    // L'identité attestée, elle, voit son message.
    let p = page(search_via(&mut trio.a, request("présent")));
    assert_eq!(p.hits.len(), 1);
    trio.stop();
}

// ---------------------------------------------------------------------------
// S19/S20/S21/S31 — fils 102 : membre, non-membre, instantané, non-mutation
// ---------------------------------------------------------------------------

#[test]
fn spec104_s19_s20_fil_membre_instantane_et_refus_identiques() {
    let mut trio = Trio::start("spec104-s19", |_| {});
    let thread_id = create_thread(&mut trio.a, "Décisions", &[AGENT_B]);
    for i in 1..=130u64 {
        let body = if i % 10 == 0 {
            format!("entrée {i} : DÉCISION prise")
        } else {
            format!("entrée {i} : bavardage")
        };
        assert_eq!(post(&mut trio.a, &thread_id, &body), i);
    }
    // Membre : résultats typés fil, référence history dans les notices.
    let mut r = thread_request(&thread_id, "decision");
    r.limit = Some(5);
    let p = page(search_via(&mut trio.b, r.clone()));
    assert_eq!(p.hits.len(), 5, "{p:?}");
    assert_eq!(p.consistency, "immutable_upper_bound");
    assert!(
        p.notices
            .iter()
            .any(|n| n.contains("bridget_thread action=history"))
    );
    let seqs: Vec<u64> = p
        .hits
        .iter()
        .map(|h| match h {
            LedgerSearchHit::ThreadEntry {
                seq,
                thread_id: t,
                author_id,
                ..
            } => {
                assert_eq!(t, &thread_id);
                assert_eq!(author_id, AGENT_A);
                *seq
            }
            _ => panic!(),
        })
        .collect();
    assert_eq!(seqs, [130, 120, 110, 100, 90]);
    assert_eq!(p.stop_reason, "result_limit");
    // Entrée 131 concurrente : la suite reste sous la borne 130 ; une nouvelle
    // recherche voit 131.
    assert_eq!(
        post(&mut trio.a, &thread_id, "DÉCISION 131 concurrente"),
        131
    );
    let mut hits = p.hits.clone();
    let mut cursor = p.next_cursor.clone();
    while let Some(c) = cursor.clone() {
        r.cursor = Some(c);
        let next = page(search_via(&mut trio.b, r.clone()));
        hits.extend(next.hits);
        cursor = next.next_cursor;
        if !next.has_more {
            break;
        }
    }
    let all: Vec<u64> = hits
        .iter()
        .map(|h| match h {
            LedgerSearchHit::ThreadEntry { seq, .. } => *seq,
            _ => panic!(),
        })
        .collect();
    assert_eq!(
        all,
        (1..=13).rev().map(|k| k * 10).collect::<Vec<_>>(),
        "instantané 1..130"
    );
    let fresh = page(search_via(
        &mut trio.b,
        thread_request(&thread_id, "concurrente"),
    ));
    assert_eq!(fresh.hits.len(), 1);
    // Filtres auteur/dates sur un fil ; peer interdit (validation).
    let mut r = thread_request(&thread_id, "decision");
    r.author = Some(AGENT_B.into());
    assert!(page(search_via(&mut trio.b, r)).hits.is_empty());
    let mut r = thread_request(&thread_id, "decision");
    r.peer = Some(AGENT_A.into());
    assert_eq!(error_code(&search_via(&mut trio.b, r)), "invalid_params");
    // Non-membre et fil inexistant : même refus, sans compteur distinct.
    let refused = search_via(&mut trio.e, thread_request(&thread_id, "decision"));
    assert_eq!(error_code(&refused), "not_found_or_forbidden");
    let absent = search_via(&mut trio.e, thread_request(AGENT_C, "decision"));
    assert_eq!(error_code(&absent), "not_found_or_forbidden");
    if let (
        LedgerSearchOutcomeV1::Error { reason: r1, .. },
        LedgerSearchOutcomeV1::Error { reason: r2, .. },
    ) = (&refused, &absent)
    {
        assert_eq!(r1, r2);
        assert!(!r1.contains("Décisions"), "aucun titre dans le refus");
    }
    // Un curseur de fil chez un non-membre : refusé avant toute lecture.
    let mut r = thread_request(&thread_id, "decision");
    r.limit = Some(5);
    let first = page(search_via(&mut trio.b, r.clone()));
    r.cursor = first.next_cursor;
    assert_eq!(error_code(&search_via(&mut trio.e, r)), "invalid_cursor");
    trio.stop();
}

#[test]
fn spec104_s21_recherche_sans_effet_ni_message() {
    let mut trio = Trio::start("spec104-s21", |conn| {
        insert(
            conn,
            "dm",
            now(),
            AGENT_B,
            AGENT_A,
            "message direct à retrouver",
        );
    });
    let thread_id = create_thread(&mut trio.a, "Sans effet", &[AGENT_B]);
    post(&mut trio.a, &thread_id, "à retrouver dans le fil");
    let db = trio.root.join("state/bridget.db");
    let conn = Connection::open(&db).unwrap();
    let tables = [
        "thread_reads",
        "thread_wakes",
        "thread_operations",
        "discussion_members",
        "discussion_entries",
        "tracked_requests",
        "ledger",
    ];
    let before: Vec<Vec<String>> = tables.iter().map(|t| table_dump(&conn, t)).collect();
    let p = page(search_via(
        &mut trio.b,
        thread_request(&thread_id, "retrouver"),
    ));
    assert_eq!(p.hits.len(), 1);
    let p = page(search_via(&mut trio.a, request("retrouver")));
    assert_eq!(p.hits.len(), 1);
    let LedgerReadOutcomeV1::Ok(_) = read_via(
        &mut trio.a,
        LedgerReadRequest {
            id: "dm".into(),
            target: AGENT_A.into(),
            offset: 0,
            digest: None,
        },
    ) else {
        panic!()
    };
    let after: Vec<Vec<String>> = tables.iter().map(|t| table_dump(&conn, t)).collect();
    assert_eq!(
        before, after,
        "aucune table modifiée par la recherche ou la relecture"
    );
    // Aucune remise ni sollicitation n'a été émise vers B ou A.
    assert_no_delivery(&mut trio.b);
    assert_no_delivery(&mut trio.a);
    // La lecture confirmée du fil n'a pas avancé : B lit encore l'entrée.
    let read = thread(
        &mut trio.b,
        json!({"action":"read","thread_id":thread_id,"limit":10}),
    );
    assert_eq!(read["entries"].as_array().map(Vec::len), Some(1), "{read}");
    trio.stop();
}

#[test]
fn spec104_s31_capacite_fils_absente() {
    let (_db, conn) = fixture_db("spec104-s31");
    conn.execute_batch("DROP TABLE thread_wakes; DROP TABLE thread_reads; DROP TABLE thread_operations; DROP TABLE discussion_entries; DROP TABLE discussion_members; DROP TABLE discussion_threads;").unwrap();
    let outcome = search_on(&conn, AGENT_A, &thread_request(AGENT_C, "x"));
    assert_eq!(error_code(&outcome), "capability_unavailable");
    // Les messages restent cherchables.
    insert(&conn, "m", now(), AGENT_A, AGENT_B, "x");
    assert_eq!(page(search_on(&conn, AGENT_A, &request("x"))).hits.len(), 1);
}

// ---------------------------------------------------------------------------
// S22 — identité révoquée pendant le travail ; S24 — permis et concurrence
// ---------------------------------------------------------------------------

fn slow_corpus(conn: &Connection, agent: &str, rows: usize) {
    slow_corpus_at(conn, agent, rows, now());
}

fn slow_corpus_at(conn: &Connection, agent: &str, rows: usize, base_ts: i64) {
    // Corps de 16 Mio (multi-octets pour un repli plus coûteux) : une page
    // traite un seul de ces corps, ce qui laisse une fenêtre observable.
    let body = "é".repeat((8 << 20) - 8) + " cible"; // 16 Mio − 10 octets
    for i in 0..rows {
        insert(
            conn,
            &format!("slow-{agent}-{i}"),
            base_ts - i as i64,
            agent,
            AGENT_C,
            &body,
        );
    }
}

#[test]
fn spec104_s22_identite_revoquee_pendant_le_travail() {
    let mut trio = Trio::start("spec104-s22", |conn| slow_corpus(conn, AGENT_A, 1));
    // Connexion auxiliaire attestée de A (même chemin que le MCP :
    // RegisterAuxiliary avec la preuve privée) : sa légitimité dépend de la
    // route vivante du owner.
    let mut aux = Client::connect(&trio.socket);
    attest_agent(&trio.socket, &mut aux, AGENT_A, "spec104-a");
    aux.send(WrapperToDaemon::LedgerSearch {
        request: request("cible"),
    });
    std::thread::sleep(Duration::from_millis(40));
    // Le owner disparaît pendant le repli : aucun résultat n'est publié.
    drop(std::mem::replace(
        &mut trio.a,
        Client::connect(&trio.socket),
    ));
    let outcome = match aux.receive() {
        DaemonToWrapper::LedgerSearchResult { outcome } => outcome,
        other => panic!("résultat attendu, reçu {other:?}"),
    };
    assert_eq!(error_code(&outcome), "identity_unavailable");
    // Permis libéré : une autre identité cherche normalement.
    let p = page(search_via(&mut trio.b, request("cible")));
    assert!(p.hits.is_empty());
    trio.stop();
}

#[test]
fn spec104_s24_deux_permis_troisieme_busy_et_daemon_reactif() {
    let mut trio = Trio::start("spec104-s24", |conn| {
        slow_corpus(conn, AGENT_A, 1);
        slow_corpus(conn, AGENT_B, 1);
        insert(conn, "e", now(), AGENT_E, AGENT_C, "cible rapide");
    });
    let socket = trio.socket.clone();
    let a = std::mem::replace(&mut trio.a, Client::connect(&socket));
    let b = std::mem::replace(&mut trio.b, Client::connect(&socket));
    let started = Instant::now();
    let workers: Vec<_> = [a, b]
        .into_iter()
        .map(|mut client| {
            std::thread::spawn(move || {
                let outcome = search_via(&mut client, request("cible"));
                (page(outcome).hits.len(), client)
            })
        })
        .collect();
    std::thread::sleep(Duration::from_millis(60));
    // Troisième lecture pendant les deux premières : busy immédiat, c'est-à-dire
    // sans attendre qu'un permis se libère. Session 123 : constat relatif, un
    // seuil absolu de 500 ms échouait sous charge sans défaut du daemon.
    let third = search_via(&mut trio.e, request("cible"));
    assert_eq!(error_code(&third), "busy");
    assert!(
        workers.iter().all(|worker| !worker.is_finished()),
        "busy doit revenir avant la fin des recherches qui tiennent les permis"
    );
    // Le daemon reste réactif pendant les recherches : l'annuaire répond.
    let mut probe = Client::connect(&socket);
    probe.send(WrapperToDaemon::ListAgents);
    assert!(matches!(probe.receive(), DaemonToWrapper::AgentList { .. }));
    for worker in workers {
        let (hits, _client) = worker.join().unwrap();
        assert_eq!(hits, 1);
    }
    eprintln!(
        "MESURE104 s24 deux recherches 16 Mio concurrentes : {:?}",
        started.elapsed()
    );
    // Permis rendus : E cherche maintenant.
    assert_eq!(
        page(search_via(&mut trio.e, request("cible"))).hits.len(),
        1
    );
    trio.stop();
}

// ---------------------------------------------------------------------------
// S25/S26 — performance : archive 100 000 × 1 Kio, gros corps, mémoire
// ---------------------------------------------------------------------------

fn rss_kib(pid: u32) -> u64 {
    let output = std::process::Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0)
}

fn percentile(samples: &mut [Duration], p: f64) -> Duration {
    samples.sort();
    let index = ((samples.len() as f64 - 1.0) * p).round() as usize;
    samples[index]
}

#[test]
fn spec104_s25_s26_performance_archive_gros_corps_et_memoire() {
    let release = !cfg!(debug_assertions);
    // Daemon de performance : le disjoncteur par paire (8 échanges / 180 s)
    // est dimensionné pour 200 messages témoins ; rien d'autre ne change.
    let mut trio = Trio::start_with(
        "spec104-s25",
        |conn| {
            let base = now();
            let filler = "lorem ipsum dolor sit amet ".repeat(37); // ≈ 1 Kio
            conn.execute_batch("BEGIN").unwrap();
            let mut statement = conn
            .prepare("INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES (?1, ?2, ?3, ?4, ?5, 'k')")
            .unwrap();
            for i in 0..100_000i64 {
                let (sender, target) = if i % 2 == 0 {
                    (AGENT_A, AGENT_B)
                } else {
                    (AGENT_B, AGENT_A)
                };
                let body = if i % 97 == 0 {
                    format!("{filler} aiguille {i}")
                } else {
                    format!("{filler} {i}")
                };
                statement
                    .execute(params![format!("p-{i:06}"), base - i, sender, target, body])
                    .unwrap();
            }
            drop(statement);
            conn.execute_batch("COMMIT").unwrap();
            // Corps géants de E datés avant les messages témoins : la fenêtre
            // `until` les isole des 200 remises reçues pendant le banc.
            slow_corpus_at(conn, AGENT_E, 3, now() - 1_000);
        },
        spawn_performance_daemon,
    );
    let pid = trio.daemon.as_ref().unwrap().child.id();
    let rss_before = rss_kib(pid);

    // 200 pages de recherche (A) en parallèle de 200 messages directs (B → A).
    let socket = trio.socket.clone();
    let mut a = std::mem::replace(&mut trio.a, Client::connect(&socket));
    let searcher = std::thread::spawn(move || {
        let mut samples = Vec::new();
        let mut cursor = None;
        let mut total_hits = 0;
        let mut retries = 0u32;
        for _ in 0..200 {
            let mut r = request("aiguille");
            r.limit = Some(50);
            r.cursor = cursor.clone();
            let started = Instant::now();
            // Un écrivain qui valide (journal rollback, fsync sous charge) peut
            // dépasser l'attente SQLite de 100 ms : le contrat répond alors
            // `storage_unavailable` et le client rejoue la même page. La latence
            // mesurée inclut ces rejeux ; leur nombre est rapporté et borné.
            let p = loop {
                match search_via(&mut a, r.clone()) {
                    LedgerSearchOutcomeV1::Ok(page) => break page,
                    LedgerSearchOutcomeV1::Error { code, .. }
                        if code == "storage_unavailable" && retries < 50 =>
                    {
                        retries += 1;
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    LedgerSearchOutcomeV1::Error { code, reason } => {
                        panic!("refus {code} : {reason}")
                    }
                }
            };
            samples.push(started.elapsed());
            total_hits += p.hits.len();
            cursor = p.next_cursor.clone();
            if !p.has_more {
                break;
            }
        }
        (samples, total_hits, retries, a)
    });
    // Messages directs témoins B → E pendant les recherches de A : latence
    // mesurée de l'envoi (client négocié 099 de B) à la remise chez E.
    let mut sender = Client::connect(&socket);
    sender.send(WrapperToDaemon::RoleHandshake {
        role: bridget_transport::protocol::ConnectionRole::Client,
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::RoleAccepted { .. }
    ));
    sender.send(WrapperToDaemon::ClientHello {
        contract_version: bridget_transport::protocol::CLIENT_CONTRACT_VERSION,
        issuer_scope: "spec104-performance-scope".to_string(),
        capabilities: vec![
            bridget_transport::protocol::ClientCapability::SendIdempotent,
            bridget_transport::protocol::ClientCapability::Lookup,
        ],
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    attest_agent(&socket, &mut sender, AGENT_B, "spec104-b");
    let mut dm_samples = Vec::new();
    for i in 0..200 {
        let mut message = bridget_core::BridgetMessage::new(
            AGENT_B,
            AGENT_E,
            format!("témoin direct {i} {}", op()),
        );
        let id = op();
        message.id = id.clone();
        let started = Instant::now();
        sender.send(WrapperToDaemon::SendIdempotent {
            message,
            message_id: id,
            issued_at: now(),
        });
        match sender.receive() {
            DaemonToWrapper::IdempotencyResult { .. } => {}
            other => panic!("accusé d'envoi attendu, reçu {other:?}"),
        }
        let delivered = receive_delivery(&mut trio.e);
        assert!(
            matches!(delivered, DaemonToWrapper::DeliverIdempotent { .. }),
            "{delivered:?}"
        );
        dm_samples.push(started.elapsed());
    }
    let (mut samples, total_hits, retries, _a) = searcher.join().unwrap();
    let p95_search = percentile(&mut samples, 0.95);
    let p95_dm = percentile(&mut dm_samples, 0.95);
    eprintln!(
        "MESURE104 s25 archive 100000×1Kio : {} pages, {} résultats, p95 page={:?}, max={:?}, rejeux storage_unavailable={retries} ; 200 DM témoins p95={:?} ; build={}",
        samples.len(),
        total_hits,
        p95_search,
        samples.last().unwrap(),
        p95_dm,
        if release { "release" } else { "debug" }
    );
    assert!(retries < 50, "trop de rejeux : {retries}");
    if release {
        assert!(
            p95_search < Duration::from_secs(1),
            "p95 recherche {p95_search:?}"
        );
        assert!(
            p95_dm < Duration::from_secs(1),
            "p95 messages directs {p95_dm:?}"
        );
    }

    // S26 : trois candidats de 16 Mio pour E ; une page traite un corps hors
    // budget puis s'arrête ; mémoire additionnelle du daemon < 128 Mio.
    let mut big_samples = Vec::new();
    let mut cursor = None;
    let mut pages_count = 0;
    let until = now() - 500; // même filtre à chaque page : le curseur l'exige
    loop {
        let mut r = request("cible");
        r.until = Some(until);
        r.cursor = cursor.clone();
        let started = Instant::now();
        let p = page(search_via(&mut trio.e, r));
        big_samples.push(started.elapsed());
        pages_count += 1;
        assert_eq!(p.hits.len(), 1, "un corps de 16 Mio par page");
        assert!(p.scanned_bytes <= (16 << 20) + 16, "{}", p.scanned_bytes);
        cursor = p.next_cursor.clone();
        if !p.has_more {
            break;
        }
    }
    let rss_after = rss_kib(pid);
    let added_mib = rss_after.saturating_sub(rss_before) / 1024;
    eprintln!(
        "MESURE104 s26 trois corps 16 Mio : {pages_count} pages, max page={:?}, RSS daemon avant={} Mio après={} Mio (+{added_mib} Mio) ; build={}",
        big_samples.iter().max().unwrap(),
        rss_before / 1024,
        rss_after / 1024,
        if release { "release" } else { "debug" }
    );
    assert_eq!(pages_count, 3);
    if release {
        assert!(
            big_samples.iter().all(|d| *d < Duration::from_secs(2)),
            "{big_samples:?}"
        );
        assert!(added_mib < 128, "mémoire additionnelle {added_mib} Mio");
    }
    trio.stop();
}

// ---------------------------------------------------------------------------
// S27 — parité CLI/MCP, ancien contrat ; S29 — injection et non-réseau
// ---------------------------------------------------------------------------

#[test]
fn spec104_s27_s29_parite_cli_mcp_ancien_contrat_et_injection() {
    let hostile = "cible \u{1b}[31mROUGE\u{1b}[0m '; DROP TABLE ledger; -- voir /etc/passwd et https://exemple.invalid/x";
    let mut trio = Trio::start("spec104-s27", |conn| {
        insert(conn, "h1", now(), AGENT_B, AGENT_A, hostile);
        insert(conn, "h2", now() - 1, AGENT_A, AGENT_B, "seconde cible");
    });
    // MCP : catalogue inchangé (pas de nouvel outil), schéma fermé, action=search.
    let mut mcp = McpProcess::start(&trio.root, AGENT_A, "spec104-a");
    let init = mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert!(init["result"]["protocolVersion"].is_string(), "{init}");
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    let tools = mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("search")),
        "aucun outil MCP supplémentaire : {names:?}"
    );
    let ledger_tool = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "bridget_ledger")
        .unwrap();
    assert_eq!(
        ledger_tool["inputSchema"]["properties"]["action"]["enum"],
        json!(["recent", "search", "read"])
    );
    assert_eq!(ledger_tool["inputSchema"]["additionalProperties"], false);

    let call = |mcp: &mut McpProcess, id: u64, args: Value| {
        mcp.request(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_ledger","arguments":args}}))
    };
    let searched = call(
        &mut mcp,
        3,
        json!({"action":"search","query":"cible","limit":10}),
    );
    let structured = &searched["result"]["structuredContent"];
    assert_eq!(structured["status"], "ok", "{searched}");
    assert!(searched["result"]["isError"].is_null());
    assert_eq!(structured["hits"].as_array().unwrap().len(), 2);
    assert_eq!(structured["hits"][0]["kind"], "message");
    // Le JSON garde la donnée brute (ANSI compris) : aucune réécriture.
    assert_eq!(structured["hits"][0]["excerpt"], hostile);
    let text: Value =
        serde_json::from_str(searched["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(&text, structured);
    // Lecture ciblée depuis un résultat.
    let hit = &structured["hits"][0];
    let read = call(
        &mut mcp,
        4,
        json!({"action":"read","id":hit["id"],"target":hit["target"],"offset":hit["match_offset"],"digest":hit["body_digest"]}),
    );
    assert_eq!(
        read["result"]["structuredContent"]["fragment"], hostile,
        "{read}"
    );
    // Refus typés = erreurs d'outil.
    let bad = call(
        &mut mcp,
        5,
        json!({"action":"search","query":"","limit":10}),
    );
    assert_eq!(bad["result"]["isError"], true, "{bad}");
    assert_eq!(bad["result"]["structuredContent"]["code"], "invalid_params");
    let bad = call(
        &mut mcp,
        6,
        json!({"action":"search","query":"cible","view":"messages"}),
    );
    assert!(
        bad["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("view")
            || bad["result"]["isError"] == true,
        "{bad}"
    );
    let bad = call(
        &mut mcp,
        7,
        json!({"action":"read","id":"h2","target":AGENT_B,"offset":1}),
    );
    assert_eq!(
        bad["result"]["structuredContent"]["code"], "invalid_params",
        "{bad}"
    );
    let bad = call(&mut mcp, 8, json!({"action":"recent","query":"cible"}));
    assert!(
        bad["error"].is_object() || bad["result"]["isError"] == true,
        "recent n'accepte pas query : {bad}"
    );
    // Ancien contrat : sans action, la projection historique répond comme avant.
    let recent = call(&mut mcp, 9, json!({"view":"messages","limit":5}));
    let old = &recent["result"]["structuredContent"];
    assert!(
        old["messages"].is_array() && old["requests"].is_array(),
        "{recent}"
    );
    assert_eq!(old["messages"].as_array().unwrap().len(), 2);
    assert!(old.get("hits").is_none());
    let table_ok: bool = Connection::open(trio.root.join("state/bridget.db"))
        .unwrap()
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='ledger')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(table_ok, "injection SQL inerte");
    drop(mcp);

    // CLI : mêmes résultats en --json, rendu humain inerte, codes de sortie.
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &[
            "ledger", "search", "--query", "cible", "--limit", "10", "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    let cli_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cli_json["hits"], structured["hits"], "parité CLI/MCP");
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &["ledger", "search", "--query", "cible"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    let human = String::from_utf8_lossy(&output.stdout);
    assert!(!human.contains('\u{1b}'), "ANSI neutralisé : {human:?}");
    assert!(
        human.contains("ROUGE") && human.contains("fin de la partie conservée"),
        "{human}"
    );
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &[
            "ledger", "read", "--id", "h1", "--target", AGENT_A, "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    let read_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(read_json["fragment"], hostile);
    // Codes : 2 paramètres invalides (locaux et daemon), 1 refus.
    for args in [
        vec!["ledger", "search"],
        vec!["ledger", "search", "--query", "x", "--bogus"],
        vec!["ledger", "search", "--query", "x", "--source", "global"],
        vec!["ledger", "search", "--query", "x", "--limit", "0"],
        vec!["ledger", "search", "--query", "x", "--cursor", "zz"],
        vec!["ledger", "search", "--query", "x", "--since", "-3"],
        vec!["ledger", "read", "--id", "h1"],
    ] {
        let output = cli_as(&trio.root, AGENT_A, "spec104-a", &args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}\n{}",
            output_text(&output)
        );
    }
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &[
            "ledger", "read", "--id", "inconnu", "--target", AGENT_A, "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    let refus: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(refus["code"], "not_found_or_forbidden");
    // Zéro résultat = succès.
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &["ledger", "search", "--query", "introuvable"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    // Ancien `ledger --limit` inchangé.
    let output = cli_as(
        &trio.root,
        AGENT_A,
        "spec104-a",
        &["ledger", "--limit", "5"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Derniers 2 messages"),
        "{}",
        output_text(&output)
    );
    trio.stop();
}
