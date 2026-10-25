# Recette prévue — SPEC145

Date : 2026-10-07. Procédure à exécuter après implémentation ; aucun résultat présumé.

## Environnement

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/quickstart.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/quickstart.md#extrait-2).

Rust : utiliser `/Users/moi/.cargo/bin/cargo` directement. Fixer CARGO_TARGET_DIR à un dossier privé145. Le rejeu frais attesté emploie `/tmp/bridget145-cache-check.9Uc6FY`. Pour un futur contrôle frais, créer un nouveau dossier privé avec `mktemp -d /tmp/bridget145-target.XXXXXX`, puis employer son chemin absolu comme CARGO_TARGET_DIR. Ne pas modifier PATH pour sélectionner Cargo. Ne pas utiliser des artefacts du target partagé comme preuve145 après un essai de baseline. Un résultat zéro test ne valide aucune sélection.

## Scénarios

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/quickstart.md#extrait-3).

Risques à reproduire : daemon absent sans autostart ni création de namespace ; inspect intercepté avant initialize_process et resolve_current_identity ; lecture et refus avant collect_closed_attach_views ; racine Git différente du workspace traitée par resolve_communication_project ; sortie métier exit2 décodée sans exposer stderr. Les comparaisons portent sur les données métier, pas sur les octets SQLite/WAL.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/quickstart.md#extrait-4).

## Clôture

La réussite exige des preuves SC145 distinctes pour backend et frontend. Une réussite documentaire ne prouve aucune installation. Commit, fusion, push, déploiement et redémarrage restent soumis à une autorisation distincte.
