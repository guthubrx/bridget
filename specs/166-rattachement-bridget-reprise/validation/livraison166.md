# Reçu de livraison — SPEC-166

Date : 2026-10-15.

## Résultat

La reprise d’un fil Codex T3 recharge maintenant l’identité Bridget courante avant le premier tour si l’identité montée est absente ou périmée. Un échec de remontage bloque le tour. Il ne réutilise pas une identité voisine.

## Sources et application

- Source T3 intégrée et poussée : `76780a20bb22b9390eab2389eaf0db87e1349e7e`
- Branche T3 : `local/main-20261009`
- Application : `/Applications/T3 Code (Local).app`
- Version active : `0.0.45-local.166`
- SHA-256 ASAR : `b397aaa905d5c023392afee8459c6d6150a576fc7ef5d18987c0dafb822fb433`
- Archive signée : `/Users/moi/.cache/t3-final166.K49J0ZDU/artifacts/T3-Code-0.0.45-local.166-arm64.zip`
- SHA-256 archive : `8fbf8c9a7043bb0e263939fb70cb77643805f790ca71c9222abcd19b38b3328c`
- Rollback : `/Users/moi/.cache/t3-install166.VqBHdpl2/T3-before.app`

## Tests

- Adaptateur Codex, gestionnaire de sessions et registre privé : 233 PASS.
- Attestation Codex ciblée : 8 PASS, 13 tests exclus par le filtre.
- Format, lint ciblé et `git diff --check` : conformes.
- Deux avertissements lint préexistants restent hors diff.
- Le typecheck global conserve des erreurs héritées hors fichiers affectés. Aucun succès global n’est déduit.

## Recette réelle

Le fil existant `17f50a6d-ec2f-494a-b9a3-0def5b14475b`, intitulé « Relancer le processus », renvoyait auparavant `t3_session_unavailable`.

Après activation de la version 166, un nouveau tour a utilisé un outil Bridget en lecture seule. Le résultat exact est `RECETTE166_OK`. Aucune réinstallation de Bridget n’a été faite. Le pont était `running` avec 32 fils et le serveur T3 utilisait le PID 65608.

## Incident d’activation

La commande `launchctl submit` utilisée pour automatiser l’unique relance a créé une tâche persistante. Elle a fermé et rouvert T3 71 fois. Ce comportement venait du script d’activation, pas du correctif T3.

Le service `com.bridget.t3-activate166` a été retiré. Le script `/Users/moi/.cache/t3-activate166.sh` a été neutralisé. Le processus principal T3 PID 65544 et le serveur PID 65608 sont restés stables après le retrait. Firefox et Bridget n’ont pas été arrêtés.

## État final

- 14 tâches sur 14 terminées.
- Statut : Implemented.
- Pipeline : Livré.
- Données utilisateur conservées.
- Rollback conservé.
