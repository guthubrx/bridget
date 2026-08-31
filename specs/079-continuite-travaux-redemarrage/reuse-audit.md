# Audit de réutilisation - SPEC-079

## Décision

Statut: PASS
Date: 2026-08-31
Feature dir: /home/moi/bridget-referent/.worktrees/session-079-continuite-travaux-redemarrage/specs/079-continuite-travaux-redemarrage

Le système possède déjà les primitives nécessaires à la continuité. La SPEC-079
les raccorde au chemin UI idempotent et ajoute uniquement la politique de ronde
absente. Aucun second moteur d'exécution, registre projet ou transport n'est créé.

## Réutilisations

| Besoin | Existant | Décision |
|---|---|---|
| travail logique | WorkSubmission | conserver le même submission_id |
| tentative runtime | Execution | créer un descendant |
| corps exact | work_submissions.message_json | relire sans synthèse |
| lignée | execution_continuations | mode reconstructed |
| remise rejouable | send_deliveries | réutiliser |
| lien causal | send_delivery_execution_links | rendre atomique et consommer |
| suppression doublon wrapper | IdempotentDeliveryTracker | conserver |
| identité projet | ProjectReference | propager sans inférence |
| autorité projet | ProjectBinding | valider génération et état |
| observation ronde | scripts/bridget-ronde.py | laisser passive |

## Duplications évitées

- aucune file de recovery parallèle;
- aucune table de message bis;
- aucun coordinateur par projet;
- aucun timer systemd par projet;
- aucune branche fournisseur;
- aucune déduction de projet depuis cwd, nom ou domaine.

## Constat productif

Le wrapper installé `/home/moi/.local/bin/bridget-ronde-complete` réalise une
émission active qui n'existe pas dans le dépôt. La SPEC-079 ajoute un dispatcher
source-controlled et ne modifie pas ce fichier productif avant une livraison
explicite.

## Gate

- [x] SPEC-063 à 067 relues.
- [x] Lien remise-exécution existant identifié.
- [x] Méthode `recoverable_execution_ids` existante mais sans appel identifiée.
- [x] Carte Maicie générique distinguée d'une exécution Bridget.
- [x] Ronde passive du dépôt distinguée du wrapper actif installé.
- [x] Aucun doublon non arbitré.
