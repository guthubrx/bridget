# Journal d’implémentation — Session 061

## Métadonnées

- **Spec** : `061-usurpation-emetteur`
- **Branche** : `session-061-usurpation-emetteur`
- **Base gelée** : `9c6f47d` (tête de `session-060`, **pas** `main`)
- **Tête** : `b04cd3a`
- **Date** : 2026-08-28

## Fichier modifié

`crates/bridget-daemon/src/daemon.rs` — variante `RefuseImpersonation`, ordre de
décision dans `resolve_sender_attribution`, bras de refus dans le handler,
témoins.

## Décision de conception

La garde ne porte que sur le `--from` **explicite** (`from_declared == true`).

Fermer aussi le chemin implicite — le nom hérité de `BRIDGET_AGENT_NAME` par un
CLI lancé depuis un wrapper — aurait cassé un usage existant sans fermer aucune
des trois usurpations mesurées : toutes les trois ont été produites par un
`--from` explicite. Le témoin `nom_herite_de_l_environnement_reste_conserve`
atteste cette non-régression.

L’ordre des tests dans la fonction a été inversé : `from_declared` est examiné
**avant** `from_is_addressable`. C’est ce renversement qui constitue le
correctif ; la branche `Keep` subsiste, mais elle n’est plus atteignable par une
déclaration explicite.

## Un témoin de 060 attestait la faille

`emetteur_cli_nomme_adressable_est_conserve`, écrit lors de la session 060,
affirmait qu’un nom déclaré correspondant à un agent connecté est *conservé*.
C’était exactement le comportement à fermer. Il est remplacé par
`emetteur_cli_ne_peut_pas_usurper_un_agent_connecte`.

Un témoin vert n’atteste pas qu’un comportement est bon : il atteste qu’il est
celui qu’on a écrit. Celui-ci protégeait une faille.

## Preuves

- **Témoins nominaux** : `6 passed / 0 failed / 599 filtered`.
- **Mutant causal** : `RefuseImpersonation` → `Keep`, c’est-à-dire le
  rétablissement exact de la faille. SHA-256 sous mutant :
  `10980343da7ce141124e1228c7faf3a450aaae02ab0def9c4429363ebe4f31dc`.
  Effet **sélectif**, mesuré témoin par témoin :

  | témoin | sous mutant |
  |--------|-------------|
  | `un_cli_temporaire_ne_peut_pas_emettre_sous_le_nom_humain` | `0 passed / 1 failed` — meurt |
  | `emetteur_cli_ne_peut_pas_usurper_un_agent_connecte` | `0 passed / 1 failed` — meurt |
  | `emetteur_cli_nomme_non_adressable_est_refuse` (060) | `1 passed / 0 failed` — survit |
  | `nom_herite_de_l_environnement_reste_conserve` | `1 passed / 0 failed` — survit |

  Le mutant tue les deux assertions visées et épargne les deux contrôles : la
  propriété de 060 n’est pas emportée par le correctif de 061.
- **Restauration** : `git checkout --` sur un arbre **commité**, SHA-256 revenu
  à `0293191b0d65a6e4e442cfc31a2c4c96d71d80822fb54d5ced1c3ba1f7691122`,
  identique au blob du commit `b04cd3a`.
- `cargo check --workspace --all-targets` vert ; `rustfmt --edition 2024`
  conforme.

## Incident de méthode, consigné

Une première rédaction du correctif a été **détruite par ma propre séquence de
preuve** : `git checkout -- <fichier>` a restauré depuis `HEAD`, alors que le
travail n’était pas encore commité. Le mutant a donc été « restauré » vers la
version 060, emportant tout le delta 061.

Le SHA-256 de contrôle a détecté l’écart immédiatement — c’est précisément ce à
quoi il sert, et il a fonctionné. Le correctif a été refait, puis **commité
avant** la séquence de preuve. En session 060, l’arbre était déjà commité quand
la mutation a eu lieu, ce qui masquait le danger.

**Règle qui en découle** : muter et restaurer par `git checkout` exige un arbre
propre. Sur un travail non commité, il faut copier le fichier avant mutation et
restaurer depuis la copie.

## Non attesté

- L’intégration de cette branche. Elle dépend de `060`, elle-même non intégrée :
  **`061` ne peut pas être intégrée seule.**
- Le déploiement. La faille reste ouverte sur le daemon en service tant que le
  binaire n’est pas remplacé.
- Toute résistance à un adversaire : voir la limite déclarée dans `spec.md`.
