//! Gardes d'inertie de rendu — caractères qui mentent à l'affichage.
//!
//! `char::is_control` ne couvre que Unicode **Cc**. Les bidi et autres
//! formatages (**Cf**) passent et permettent d'afficher autre chose que ce
//! qui sera exécuté (Trojan Source). La couverture ci-dessous est celle
//! déjà figée dans `attach::is_format_character` — une seule source.

/// Caractères de format Unicode (Cf et proches) déjà couverts à l'affichage
/// attach : bidi U+202A–U+202E, isolats U+2066–U+206F, zero-width, etc.
pub fn is_format_character(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{061c}'
            | '\u{06dd}'
            | '\u{070f}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08e2}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{110bd}'
            | '\u{110cd}'
            | '\u{13430}'..='\u{1345f}'
            | '\u{1bca0}'..='\u{1bca3}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0001}'
            | '\u{e0020}'..='\u{e007f}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{e0100}'..='\u{e01ef}'
    ) || matches!(character, '\u{0600}'..='\u{0605}' | '\u{2065}')
}

/// Contrôle de ligne (**Cc**) **ou** format (**Cf**/bidi) — rien de ce qui
/// ment à un relecteur humain ne doit franchir une garde de valeur.
pub fn is_disallowed_control(character: char) -> bool {
    character.is_control() || is_format_character(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contrôle positif Cc d'abord ; puis bidi réel. La preuve que le test
    /// meurt sans la couverture Cf : U+202E n'est PAS `is_control`.
    #[test]
    fn oracle_bidi_est_refuse_et_is_control_seul_est_aveugle() {
        assert!(
            is_disallowed_control('\u{0007}'),
            "PROMESSE — un Cc (BEL) DOIT être refusé"
        );
        let bidi = '\u{202e}';
        assert!(
            !bidi.is_control(),
            "preuve d'aveuglement Cc : U+202E n'est pas is_control — \
             retirer is_format_character ferait passer ce caractère"
        );
        assert!(
            is_disallowed_control(bidi),
            "oracle — U+202E (RIGHT-TO-LEFT OVERRIDE) DOIT être refusé"
        );
        assert!(
            is_disallowed_control('\u{2066}'),
            "oracle — U+2066 (LRI, isolat Trojan Source) DOIT être refusé"
        );
    }
}
