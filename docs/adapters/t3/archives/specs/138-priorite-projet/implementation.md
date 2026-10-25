# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/implementation.md](../../../../../../specs/138-priorite-projet/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Rapport | Preuve disponible | Limite actuelle |
|---|---|---|
| Noyau SPEC-138 — preuves réelles, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core.md | RED noyau initiale et RED T019 capturées ; recherche du scénario processus réel avant création | GREEN final noyau/T019 encore à consigner |
| SPEC138 — Façades de communication, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades.md | MCP51, CLI80 et client14 PASS rapportés ; matrice réelle et recette Agent Loop opt-in écrites | Recettes réelles encore à exécuter ; pas de PASS global |
| SPEC138 — Annonces propriétaires wrapper et T3, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/transport-facades.md | Cinq tests transport PASS dans une vague20 PASS/1 FAIL | RED séparée non capturée ; cette vague n'est pas une validation globale |
| T019 — Stockage durable des avertissements de contrôle, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/control-store.md | Deux RED migration puis suite SQLite17 PASS, exit0 ; warnings durables et canon exact | Seulement stockage ; garde daemon et restart processus restent au noyau |
| Preuves Agent Loop — SPEC-138, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop.md | Course et pannes E/S observées RED ; suite finale152 PASS, exit0 confirmée par le principal et revue corrective APPROVE | Transports surtout simulés ; recette daemon réel reste T017 ; limites machine/rollback/legacy ci-dessous |

<a id="extrait-2"></a>

## Extrait 2

Bridget est publié sur github/main au commit
`7f6aba8527d2ee764e61613e3e579f54a290bf65`. Le daemon actif annonce ce build.
L'application T3 reste ouverte : 28 fils, 28 présences actives et 28 projets
attestés. Agent Loop est publié sur main au commit `8008b08e`. Ses quatre
fichiers actifs sont identiques à la publication. La branche active 104 reste
distincte et non poussée pour préserver les autres travaux.
