//! Détection déterministe des citations d'objectifs (F37).
//!
//! Zéro LLM : forme UUID + lookup en base. Les UUID inconnus de la base sont
//! ignorés (pas de faux refus sur un hash git).

use uuid::Uuid;

/// Extrait les UUID au format canonique `8-4-4-4-12` présents dans `text`.
/// Les bornes refusent un UUID collé à d'autres hexadécimaux (hash git, etc.).
pub fn extract_uuids(text: &str) -> Vec<Uuid> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index + 36 <= bytes.len() {
        let slice = &bytes[index..index + 36];
        let before_ok = index == 0 || !bytes[index - 1].is_ascii_hexdigit();
        let after_ok = index + 36 == bytes.len() || !bytes[index + 36].is_ascii_hexdigit();
        if before_ok
            && after_ok
            && looks_like_uuid(slice)
            && let Ok(raw) = std::str::from_utf8(slice)
            && let Ok(parsed) = Uuid::parse_str(raw)
        {
            found.push(parsed);
            index += 36;
            continue;
        }
        index += 1;
    }
    found.sort_unstable();
    found.dedup();
    found
}

fn looks_like_uuid(slice: &[u8]) -> bool {
    slice.len() == 36
        && slice[8] == b'-'
        && slice[13] == b'-'
        && slice[18] == b'-'
        && slice[23] == b'-'
        && slice.iter().enumerate().all(|(i, byte)| match i {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

/// Vérifie que chaque UUID connu en base cité dans le but est classé
/// `--depends-on` ou `--reference`. Retourne les UUID non classés.
pub fn unclassified_known_citations(
    goal: &str,
    known_in_store: &[Uuid],
    depends_on: &[Uuid],
    references: &[Uuid],
) -> Vec<Uuid> {
    let cited = extract_uuids(goal);
    let known: std::collections::BTreeSet<_> = known_in_store.iter().copied().collect();
    let classified: std::collections::BTreeSet<_> = depends_on
        .iter()
        .chain(references.iter())
        .copied()
        .collect();
    cited
        .into_iter()
        .filter(|id| known.contains(id) && !classified.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrait_un_uuid_et_ignore_un_hash_git() {
        let goal = "suite de 4ba5602a-4a45-4b79-b928-042b7b0ba361 après abcdef0123456789abcdef0123456789abcdef01";
        let ids = extract_uuids(goal);
        assert_eq!(
            ids,
            vec![Uuid::parse_str("4ba5602a-4a45-4b79-b928-042b7b0ba361").unwrap()]
        );
    }

    #[test]
    fn refuse_le_classement_manquant_pour_un_connu() {
        let known = Uuid::parse_str("4ba5602a-4a45-4b79-b928-042b7b0ba361").unwrap();
        let missing = unclassified_known_citations(
            "voir 4ba5602a-4a45-4b79-b928-042b7b0ba361",
            &[known],
            &[],
            &[],
        );
        assert_eq!(missing, vec![known]);
    }

    #[test]
    fn ignore_un_uuid_absent_de_la_base() {
        let ghost = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let missing = unclassified_known_citations(&format!("voir {ghost}"), &[], &[], &[]);
        assert!(missing.is_empty());
    }
}
