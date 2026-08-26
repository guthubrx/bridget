# Plan d'implémentation — Activation gouvernée des outils de pilotage

## Métadonnées

- **Spec** : 018-activation-outils-pilotage
- **Branche** : `session-18-activation-outils-pilotage`
- **Priorité** : P1
- **Dépendance** : SPEC-011 (implémentée)

## Contexte technique

- **Langages** : Bash portable macOS/Linux ; outils actifs en Python inchangés.
- **Socle** : Git, `cmp`, `mv`, `readlink`, `sha256sum` ou `shasum`.
- **Compatibilité** : éviter les fonctions Bash postérieures à 3.2.
- **Stockage** : releases utilisateur sous `$HOME/.local/share/bridget/`.
- **Tests** : harnais shell dans des dépôts Git temporaires, sans accès production.
- **Complexité** : O(1), une quantité bornée de commandes et deux artefacts.

## Vérification de la constitution

- [x] Session et branche validées avant écriture.
- [x] Worktree dédié ; checkout principal laissé sur `main`.
- [x] Propriété formulée puis validée avant le geste.
- [x] Périmètre minimal : deux installateurs partageant la même frontière.
- [x] Aucune dépendance nouvelle.
- [x] Tests discriminants avant livraison et gates projet annoncées.
- [x] Décision structurante consignée dans une ADR.
- [x] Aucun chemin de production modifié depuis la machine Linux.

## Architecture retenue

```text
checkout principal main propre
        │  gardes Git + HEAD ancêtre de origin/main
        ▼
objet Git HEAD:scripts/<outil>.py
        │  extraction + SHA-256
        ▼
~/.local/share/bridget/pilotage/releases/<SHA>/<outil>
        │  lien absolu remplacé atomiquement
        ▼
~/.local/bin/<outil>
```

Le checkout n'est qu'une autorité de sélection. Les octets de production ne
le référencent jamais après activation.

## Fichiers prévus

- `scripts/lib/pilotage-release.sh` : préconditions, matérialisation,
  provenance et activation atomique.
- `scripts/install-bridget-idle.sh` : parsing CLI puis appel de la politique.
- `scripts/install-bridget-ronde.sh` : politique avant tout effet sur unités.
- `scripts/test-018-pilotage-install.sh` : cinq refus, deux succès, idempotence,
  corruption et survie à la suppression du dépôt.
- `scripts/test-bridget-ronde.sh` : conserver les oracles métier ; déplacer la
  preuve d'installation vers le harnais gouverné.
- `docs/decisions/012-activation-outils-pilotage-par-release.md` : décision.

## Séquence

1. Écrire le harnais sur dépôts Git jetables et constater son rouge sur les installateurs actuels.
2. Implémenter la politique commune avec les refus dans l'ordre du contrat.
3. Brancher `bridget-idle`, puis valider ses scénarios.
4. Brancher `bridget-ronde` avant toute création d'unité, puis valider le faux succès fermé.
5. Rejouer les harnais existants, les mutants et les gates projet.
6. Consigner les comptes, geler le SHA et livrer pour jury.

## Risques et mitigations

| Risque | Mitigation |
|---|---|
| L'installateur tourne depuis sa propre branche non admise | Branche `main` obligatoire + ancestralité `origin/main` |
| Un fichier sale est copié malgré le SHA annoncé | Refus de l'arbre sale + extraction directe du blob Git |
| Une ancienne copie de ronde reste active | Refus non nul avant toute unité sans `--force` |
| Une release portant un SHA est altérée | Comparaison des octets et refus fail-closed |
| Des octets exacts masquent un lien ou un mode mutable | Contrôle de représentation et de mode avant l'idempotence |
| `mv` suit un lien actif vers un répertoire | Remplacement de l'entrée exacte puis attestation de la cible obtenue |
| Une unité existante diffère de la configuration demandée | Comparaison exacte ; refus non nul sans `--force` |
| Coupure pendant l'activation | Préparation temporaire puis renommage atomique |
| Différence macOS/Linux | Pas de `readlink -f`, fallback SHA-256, Bash 3.2 |

## Validation prévue

- `bash -n` sur les installateurs, le helper et les trois harnais.
- `scripts/test-018-pilotage-install.sh` : scénarios de contrat.
- `scripts/test-bridget-idle.sh` et `scripts/test-bridget-ronde.sh` : métier inchangé.
- Mutants ciblés sur les gardes worktree, branche, propreté, ancestralité et copie.
- Témoins ciblés sur release liée, mode `0755`, lien actif vers répertoire et
  rejeu de ronde avec configuration divergente.
- `cargo test --workspace --no-run` avant tout comptage.
- Suite complète sans `test-support` avec passés/rouges/ignorés et imputation.
- `cargo fmt --all --check` et Clippy selon les capacités Linux.
- Gate `test-support` déclaré non mesuré sur Linux si le défaut kqueue/kevent persiste.

## Hors périmètre

- Déployer réellement sur `/Users/moi` depuis la machine Linux.
- Prouver cryptographiquement un verdict de jury distinct de l'admission sur `origin/main`.
- Nettoyer automatiquement les anciennes releases.
- Modifier la logique métier de `bridget-idle` ou `bridget-ronde`.
