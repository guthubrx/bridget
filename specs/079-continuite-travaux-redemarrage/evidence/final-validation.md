# Validation finale - SPEC-079

Date: 2026-08-31
Commit de base: 4ad487e
Cache isolé: /tmp/bridget-spec079-current-target-20260831

## Contrôles verts

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- tests ciblés protocole, idempotence, store, daemon, UI et wrapper SPEC-079
- test SQLite réel: acquittement, redémarrage, continuation unique, second
  redémarrage et rejeu de la même remise
- `scripts/test-bridget-ronde-dispatch.sh`

Résultat du harness:

```text
dispatcher de ronde: occurrence stable, zéro projet et grammaire fermée vérifiés
```

## Preuves SPEC-079 principales

- contexte d'exécution optionnel et compatibilité historique;
- transaction remise et lien d'exécution;
- reconstruction avec payload, projet et lignée exacts;
- refus payload absent et actifs concurrents;
- reprise unique au Register;
- refus si tour vivant;
- rejeu identique après deux réouvertures SQLite;
- politique absente désactivée et épinglée au rebind;
- tick global limité aux projets activés;
- message UI humain déclenchant un tour;
- binding wrapper avant filtre de doublon;
- CLI séparant politique projet et tick global.

## Dette globale du dépôt

La suite complète n'est déjà pas verte sur le commit de base.

Reproduits directement sur `origin/main` 4ad487e:

- `matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`: rappel ou
  timeout ACP reçu à la place de la réponse corrélée;
- `spec_067_attestation_secrete_ne_porte_ni_valeur_ni_contenu`:
  `PolicyInvalid("permission secret invalide")`;
- `spec_065_reprise_apres_crash_et_collision_alias_ne_creent_qu_un_actif`:
  `BindingFailed` au lieu de `Active`;
- `loopback_rend_snapshot_et_relaie_un_fragment_attach_d_un_agent_vivant`:
  rappel reçu avant `Subscribe`.

Autres incohérences observées:

- deux tests Codex attendent encore un outil MCP que le prompt source interdit
  explicitement en session app-server;
- le test historique SC-005 laisse six groupes de fixtures après son échec;
- la campagne `--no-fail-fast` s'est bloquée dans `ui_relay_test` avec
  un daemon de fixture. Le Cargo a été arrêté avec accord opérateur; aucun
  service productif n'a été touché.

Ces éléments ne sont pas masqués. Ils sont hors des chemins SPEC-079 ou
reproduits sur le commit de base.

## Limites

Aucun provider authentifié réel, timer productif, daemon productif, commit,
merge, push ou déploiement n'a été utilisé.
