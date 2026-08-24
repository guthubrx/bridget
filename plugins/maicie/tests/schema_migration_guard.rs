//! Garde des numéros de migration Maicie.
//!
//! Deux auteurs qui posent le même entier (016×main, 2026-08-24) produisent
//! un arbre qui compile : Git fusionne le texte, le schéma diverge. Ce test
//! lit `store.rs` et échoue dès que deux blocs revendiquent la même
//! destination, ou que `SCHEMA_VERSION` n'est plus le max des destinations.
//! Il casse donc au rebase de l'auteur, pas à l'intégration.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn store_source() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/store.rs"))
}

fn schema_version_constante(source: &str) -> i64 {
    let ligne = source
        .lines()
        .find(|ligne| {
            let taille = ligne.trim_start();
            taille.starts_with("const SCHEMA_VERSION:")
        })
        .expect("const SCHEMA_VERSION introuvable");
    extraire_entier_final(ligne).expect("SCHEMA_VERSION sans entier")
}

fn corps_migrate(source: &str) -> &str {
    let debut = source.find("fn migrate(").expect("fn migrate introuvable");
    let apres_signe = &source[debut..];
    let ouverture = apres_signe.find('{').expect("corps de migrate introuvable");
    let bytes = apres_signe.as_bytes();
    let mut profondeur = 0i32;
    for (indice, octet) in bytes.iter().enumerate().skip(ouverture) {
        match octet {
            b'{' => profondeur += 1,
            b'}' => {
                profondeur -= 1;
                if profondeur == 0 {
                    return &apres_signe[ouverture..=indice];
                }
            }
            _ => {}
        }
    }
    panic!("accolade fermante de migrate introuvable");
}

fn extraire_entier_final(ligne: &str) -> Option<i64> {
    let sans_commentaire = ligne.split("//").next().unwrap_or(ligne);
    let chiffres: String = sans_commentaire
        .chars()
        .rev()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if chiffres.is_empty() {
        None
    } else {
        chiffres.parse().ok()
    }
}

/// Destination revendiquée par un seuil dans `migrate`.
///
/// - `current_version < N` pose le palier N ;
/// - `current_version == N` est le palier N → N+1.
///
/// Les deux formes du 24/08 (`< 9` et `== 9`) ne se marchent donc pas dessus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seuil {
    StrictementInferieur(i64),
    Egal(i64),
}

impl Seuil {
    fn destination(self) -> i64 {
        match self {
            Self::StrictementInferieur(n) => n,
            Self::Egal(n) => n + 1,
        }
    }
}

fn seuils_de_migration(corps: &str) -> Vec<Seuil> {
    let mut seuils = Vec::new();
    for ligne in corps.lines() {
        let code = ligne.split("//").next().unwrap_or(ligne).trim();
        if let Some(reste) = code.strip_prefix("if current_version < ") {
            let n: i64 = reste
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .expect("seuil < sans entier");
            seuils.push(Seuil::StrictementInferieur(n));
        } else if let Some(reste) = code.strip_prefix("if current_version == ") {
            let n: i64 = reste
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .expect("seuil == sans entier");
            seuils.push(Seuil::Egal(n));
        }
    }
    seuils
}

fn destinations_par_numero(seuils: &[Seuil]) -> BTreeMap<i64, usize> {
    let mut comptes = BTreeMap::new();
    for seuil in seuils {
        *comptes.entry(seuil.destination()).or_insert(0) += 1;
    }
    comptes
}

fn destinations_en_doublon(seuils: &[Seuil]) -> Vec<i64> {
    destinations_par_numero(seuils)
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(version, _)| version)
        .collect()
}

#[test]
fn deux_blocs_ne_revendiquent_pas_la_meme_destination() {
    let seuils = seuils_de_migration(corps_migrate(store_source()));
    assert!(
        !seuils.is_empty(),
        "aucun seuil de migration dans fn migrate"
    );
    let doublons = destinations_en_doublon(&seuils);
    assert!(
        doublons.is_empty(),
        "deux blocs revendiquent la même destination {doublons:?} — collision 016×main"
    );
}

#[test]
fn schema_version_egale_le_max_des_destinations() {
    let source = store_source();
    let declaree = schema_version_constante(source);
    let seuils = seuils_de_migration(corps_migrate(source));
    let max = seuils
        .iter()
        .map(|seuil| seuil.destination())
        .max()
        .expect("aucun seuil de migration");
    assert_eq!(
        declaree, max,
        "SCHEMA_VERSION={declaree} mais max des destinations={max}"
    );
}

#[test]
fn le_fichier_lu_est_bien_store_rs_du_crate() {
    let chemin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/store.rs");
    assert!(chemin.is_file(), "{}", chemin.display());
}

/// Oracle positif : le détecteur doit rougir sur le motif interdit.
/// L'absence de doublon dans `store.rs` ne prouve rien à elle seule.
#[test]
fn un_mutant_qui_duplique_un_numero_est_detecte() {
    let mutant = "\
fn migrate(connection: &mut Connection) {\n\
    if current_version < 8 {\n\
        tx.execute_batch(\"CREATE TABLE a\");\n\
    }\n\
    if current_version < 8 {\n\
        tx.execute_batch(\"CREATE TABLE b\");\n\
    }\n\
}\n";
    let doublons = destinations_en_doublon(&seuils_de_migration(corps_migrate(mutant)));
    assert_eq!(
        doublons,
        vec![8],
        "le mutant duplique la destination 8 : le garde doit la signaler"
    );
}

#[test]
fn un_mutant_dont_schema_version_diverge_du_max_est_detecte() {
    let mutant = "\
const SCHEMA_VERSION: i64 = 12;\n\
fn migrate(connection: &mut Connection) {\n\
    if current_version < 11 {\n\
        tx.execute_batch(\"CREATE TABLE a\");\n\
    }\n\
}\n";
    let declaree = schema_version_constante(mutant);
    let max = seuils_de_migration(corps_migrate(mutant))
        .iter()
        .map(|seuil| seuil.destination())
        .max()
        .expect("aucun seuil");
    assert_ne!(
        declaree, max,
        "le mutant pose SCHEMA_VERSION=12 pour un max=11"
    );
    assert_eq!(declaree, 12);
    assert_eq!(max, 11);
}

#[test]
fn les_seuils_complementaires_inf_et_egal_ne_collident_pas() {
    let source = "\
fn migrate(connection: &mut Connection) {\n\
    if current_version < 9 { }\n\
    if current_version == 9 { }\n\
    if current_version < 11 { }\n\
}\n";
    let seuils = seuils_de_migration(corps_migrate(source));
    assert!(
        destinations_en_doublon(&seuils).is_empty(),
        "< 9 (dest 9) et == 9 (dest 10) doivent coexister, comme le 24/08"
    );
    assert_eq!(
        seuils.iter().map(|seuil| seuil.destination()).max(),
        Some(11)
    );
}
