# Plan 095

## Résumé et contexte technique

Réutiliser `scripts/federate-ssh.sh` et ses gardes, ajouter gestion explicite de service. Shell Bash compatible macOS 3.2/Linux, OpenSSH, launchctl/systemctl utilisateur. Pas de dépendance Rust ni modification protocole. Python standard disponible pour les tests et, seulement si nécessaire, une sonde Unix sûre et bornée. Choisir la plus petite couture expliquant propriété et état de socket ; jamais reprendre le `rm -f` aveugle de l'ancien install.

## Vérification constitution

Session 095 proposée puis validée ; worktree distinct. La synchronisation projet SpecKit est faite. Les scripts standard de préparation et templates sont absents du checkout core : modèles officiels du dépôt historique lus, application directe limitée aux artefacts 095. Aucun commit ni publication du WIP précédent. Tests ciblés shell pendant le lot, recette système après revue. Pas de rebuild Rust inutile.

## Structure et ownership

- Auteur : scripts/federate-ssh.sh et éventuel helper adjacent strictement nécessaire, scripts/tests/federation_095_test.sh, tests/features/095-federation-services.feature.
- Pilote : specs/095-federation-services, docs/federation-services.md, installation et recette Mac.
- Équipier infrastructure : lecture/conseil puis déploiement Cartae seulement sur mandat distinct.
- Réemploi : mécanisme -R, gardes de chemins et clés 089, shell packaging existant.

## Phases

1. Figement contrat, oracles d'installation et collision rouges.
2. Install/status/remove et service runner privé autonome, templates natifs.
3. Revue sécurité/réutilisation, régressions 089 et tests 095.
4. Sauvegarde et désactivation de l'ancien ensemble Cartae, installation client autonome, tunnel permanent nouveau.
5. Annuaire commun et échange aller/retour, coupure/reconnexion contrôlée, documentation chemins/rollback.

## Minimalisme et responsabilité

Pas de framework de services générique ni de nouvelle API MCP. Le script distribué est le point d'entrée d'administration. Gestionnaire OS assure les relances ; pas de superviseur nouveau. Toute copie installée appartient au nouveau paquet. La complexité supplémentaire porte uniquement validation/configuration et refus de collisions, explicitement testés. Le binaire core conserve l'autorité et les identités ; aucune migration improvisée de l'ancienne DB.
