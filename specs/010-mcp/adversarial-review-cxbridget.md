# Contre-revue adverse — cxbridget (codex) — session 010

## Round 1 (spec.md)

**Date** : 2026-08-22 · **Verdict** : `BLOCKED` — 9 objections, **toutes
retenues (9/9, 0 rejetée)**. Le relecteur valide le plafond de trois outils
(Article XIX) et la frontière descendante 007.

| # | Objection | Vérifiée comment | Correction appliquée |
|---|---|---|---|
| 1 | priorité « env var puis filiation » incompatible avec `rename` (nom figé au lancement) | `wrapper.rs:552` : `BRIDGET_AGENT_NAME_FILE` existe précisément pour ça — confirmé | FR-004 réécrite : localisateur **dynamique** (fichier de nom / instance_id) résolu à chaque appel → filiation validée → erreur ; nom d'env figé banni comme source |
| 2 | `agent-pids/<pid>` = nom seul → usurpation par pid recyclé ; profondeur/ancêtre non fixés | raisonnement + constat du contenu actuel | entrée **typée** (pid + naissance du processus + instance_id + chemin du nom), validation naissance/instance, marche bornée jusqu'à 1, premier ancêtre **validé** ; fixtures imposées (2 agents même binaire, npx ×3, imbriqués, orphelin/pid réutilisé) |
| 3 | `bridget_ledger` contradictoire : le binaire sépare `ledger` (messages en base) et `requests` (ListRequests) | `cli.rs:87,99` — confirmé | `bridget_ledger(view: messages\|requests\|both, limit?)`, schémas séparés, requêtes daemon typées, le binaire réutilise la même couche de lecture sans changer sa sortie, équivalence champ à champ testée |
| 4 | `reply`/`reply_timeout` ambigus vs timeout harness | analyse du flux Ack/Nack | FR-002 : retour dès Ack/Nack, `reply_timeout` = cycle daemon (ne borne pas l'appel), timeout court dédié connexion+accusé < timeout harness, `reply_timeout` ignoré avec avertissement si `reply=false` |
| 5 | « sans état » ne décrit pas un serveur stdio concurrent | analyse | FR-008 réécrite : lecteur stdin unique, writer stdout sérialisé, ids chaîne/nombre, `notifications/cancelled` locale, EOF propre, connexion daemon **par appel** à délais bornés, zéro retry ; tests concurrents/redémarrage/abandon |
| 6 | client maison sans matrice de conformité = risque D-201 répété | leçon 007 | FR-009 : matrice obligatoire (11 cas) + fixtures officielles + version de protocole pinnée + **pureté stdout** (logs sur stderr), chaque harness testé contre les mêmes fixtures |
| 7 | « 009 indépendante » trop fort ; danger d'écriture des configs utilisateur | analyse périmètre | FR-012 : branchement strictement éphémère par session, aucune écriture persistante des configs utilisateur, type sans injection session = `unsupported` avec repli binaire ; frontière 009 = transmission de config au spawn, zéro logique MCP |
| 8 | frontière des erreurs indéfinie (JSON-RPC vs métier) | analyse | FR-011 : taxonomie fermée — erreur JSON-RPC (params/méthode), `isError` (daemon injoignable/timeout), résultat métier à catégories fermées + `reason`, compat ascendante des catégories inconnues |
| 9 | SC-004/SC-005 non reproductibles | analyse | SC réécrits : blocs de prompt versionnés, comptage Unicode fixé, matrice comportementale = quickstart 007 §1-§4 rejoués, harness pinnés, fixtures automatisées + smoke test réel par type |

## Round 2

**Verdict** : `APPROVE_WITH_CHANGES` — **BLOCKED levé**, « passage au plan 010
sans nouvelle contre-revue de spec nécessaire ». 5 résidus, tous retenus
(5/5) et appliqués dans la spec :

| # | Résidu | Correction |
|---|---|---|
| 1 | contradiction FR-012 (`unsupported` possible) vs SC-004 (les 4 types branchés) | gate de support versionnée : soit les 4 passent, soit la spec est révisée avant implémentation — `unsupported` ne satisfait jamais SC-004 |
| 2 | Ack perdu / `cancelled` → issue inconnue, doublon possible au retry | id métier généré avant connexion, `outcome_unknown` distinct de l'échec avant écriture, annulation jamais révocatrice, retry même id → dedup daemon ; tests dédiés |
| 3 | localisateur `instance_id` incomplet ; naissance sans valeur de référence ; Key Entities désalignée | `BRIDGET_AGENT_NAME_FILE` = localisateur direct (instance_id seulement si requête daemon de résolution définie au plan) ; naissance comparée à la valeur capturée au dépôt du marqueur ; Key Entities réécrite sur l'ordre exact de FR-004 |
| 4 | « nettement inférieur au timeout harness » intestable ; `reply_timeout`+`reply=false` flou | valeur concrète au registre (10 s), gate de support exige timeout harness strictement supérieur ; `invalid_params` au schéma |
| 5 | cas sensibles absents de la matrice FR-009 | ajoutés : `cancelled` avant/après écriture daemon, réponse après annulation, complétions hors ordre, `tools/call` avant `initialized` |

**Bilan spec 010** : 2 rounds, 14 objections, 14 retenues, 0 rejetée. **Suite
actée** : plan 010 (sans nouvelle contre-revue de spec), puis reuse-audit et
tasks après les livraisons 007 dont dépend le branchement.

## Round 3 (plan.md)

**Verdict** : `APPROVE_WITH_CHANGES` — 8 points, tous retenus (8/8) :

| # | Point | Correction |
|---|---|---|
| 1 | la connexion éphémère daemon a deux phases (`Register`/`Registered` puis commande) — « un aller-retour » était faux | D-401 réécrit : deux phases sous budget 10 s, délais distincts, échec certain avant écriture vs `outcome_unknown(id)` après, `cancelled` jamais révocatrice |
| 2 | « sans état » n'autorise pas des threads/connexions illimités | état technique borné : limite basse d'appels simultanés (`isError busy`), table des appels en vol nettoyée, lecteur stdin toujours disponible ; tests saturation/hors-ordre/EOF |
| 3 | agent lancé avant la mise à niveau → marqueur ancien indéfiniment | erreur typée `legacy_marker` avec remédiation « redémarre l'agent », pas de migration paresseuse (invalidable), documenté et testé |
| 4 | le spike de branchement exige un serveur minimal | faux serveur MCP stdio jetable (outil `probe`), indépendant du daemon ; consignation complète, tâche de recherche et non implémentation |
| 5 | couche de lecture dans `cli.rs` = mélange rendu/données | module neutre typé sans formatage ; `cli.rs` garde ses renderers ; `ListRequests` réutilisé ; golden tests octet pour octet |
| 6 | « troisième usage JSON-RPC » factuellement faux (le protocole daemon est du JSONL) | D-405 corrigé : deuxième usage, primitives publiques seulement, extraction au reuse-audit sur preuve de duplication identique, machines d'état ACP/MCP jamais couplées |
| 7 | research R-201 contredisait FR-011 ; R-203 décrivait l'ancien format de marqueur | les deux passages alignés |
| 8 | `mcp.rs` dans `bridget-daemon` | confirmé bon choix par le relecteur (aucun changement) |

**Suite actée** : contrat `outils-mcp.md` + quickstart, puis reuse-audit —
« sans nouveau round long ».
