# Implementation Plan : Outils MCP Bridget

**Branch**: `session-10-mcp` (après 008 et 009 — spec et plan anticipés) | **Date**: 2026-08-22 | **Spec**: [spec.md](./spec.md)
**Input**: spec contre-revue (2 rounds, 14 objections intégrées, approbation avec passage au plan acté)

## Summary

Une sous-commande `bridget mcp` : serveur MCP stdio par agent, exposant
`bridget_send`, `bridget_who`, `bridget_ledger` — traducteur mince entre le
protocole d'outils et le protocole daemon existant. Identité résolue à chaque
appel (fichier de nom courant, puis filiation validée), branchement éphémère
par session via le registre 007, prompt allégé pour les agents branchés.
Aucune dépendance nouvelle, aucun état persistant, aucune écriture de config
utilisateur.

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget`
**Primary Dependencies**: aucune nouvelle (JSON-RPC stdio maison — troisième
application du motif D-201/D-204 après le protocole daemon et le client ACP)
**Storage**: aucun nouveau ; lecture des marqueurs `agent-pids/` (enrichis) et
du fichier de nom courant
**Testing**: `cargo test` ; matrice de conformité FR-009 (15 cas) sur fixtures
officielles ; gate de support versionnée par harness (spike)
**Target Platform**: macOS et Linux
**Project Type**: extension CLI/daemon existants
**Constraints**: pureté stdout absolue ; timeout d'appel 10 s (registre) ;
zéro écriture persistante de configuration utilisateur
**Scale/Scope**: 1 module serveur nouveau, ~500-700 lignes, 8 tâches prévues

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | schémas d'outils, descriptions et erreurs en français |
| VII — ADR | ✅ | ADR 005 « serveur MCP comme façade du protocole daemon » en première tâche |
| IX — recherche | ✅ | `research.md` R-201..R-205 (sous-ensemble MCP, formes de branchement constatées, filiation npx) |
| XVIII — complexité | ✅ | serveur = boucle de dispatch O(1) par message ; remontée d'ancêtres bornée ; aucune boucle imbriquée |
| XIX — minimalisme | ✅ | 3 outils (plafond validé en revue), zéro dépendance, zéro état ; réutilisation du protocole daemon, du registre, des marqueurs de filiation ; la couche de lecture ledger/requests est partagée avec le binaire (pas de duplication) |
| XX — responsabilité | ✅ | limites écrites : identité = attribution en environnement de confiance, pas une authentification ; catégories de refus fermées mais extensibles |

**Point de vigilance** : le serveur parle à deux mondes à la fois (harness via
stdio, daemon via socket). Toute écriture parasite sur stdout corromprait le
canal MCP — d'où la règle de pureté stdout testée par fixtures, et des logs
exclusivement stderr.

## Project Structure

```text
specs/010-mcp/
├── plan.md              # ce fichier
├── research.md          # R-201..R-205
├── contracts/
│   └── outils-mcp.md    # schémas exacts des 3 outils + taxonomie d'erreurs
├── quickstart.md        # scénarios de validation par type d'agent
└── tasks.md             # après reuse-audit (dépend des livraisons 007)

crates/bridget-daemon/src/
├── mcp.rs               # NOUVEAU : serveur stdio (dispatch, outils, identité)
├── cli.rs               # sous-commande mcp ; couche de lecture ledger/requests partagée
├── registry.rs          # extension du schéma : branchement mcp par type
├── wrapper.rs           # dépôt des marqueurs enrichis + injection mcpServers/config
└── daemon.rs            # requêtes typées ledger/requests si manquantes

docs/decisions/005-serveur-mcp.md   # NOUVEAU : ADR
```

**Structure Decision** : `mcp.rs` dans `bridget-daemon` (il consomme le
protocole daemon et le registre) ; pas de crate nouveau.

## Approche par exigence

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001 | sous-commande `bridget mcp` ; `initialize`/`initialized`/`tools/list`/`tools/call`/`ping` ; schémas en français au contrat | `mcp.rs`, `contracts/outils-mcp.md` |
| FR-002 | `Send` daemon existant ; id généré avant connexion ; `outcome_unknown` si Ack perdu ; timeout 10 s du registre ; `invalid_params` si `reply_timeout` sans `reply` | `mcp.rs` |
| FR-003, FR-010 | mêmes messages daemon que le binaire ; couche de lecture typée sans formatage dans un **module neutre** (D-404), `cli.rs` garde ses renderers, golden tests de sortie | module neutre, `daemon.rs` |
| FR-004 | résolution par appel : fichier de nom (`BRIDGET_AGENT_NAME_FILE`) → ancêtres bornés contre `agent-pids/` typé (naissance + instance) → erreur ; marqueurs enrichis déposés par le wrapper | `mcp.rs`, `wrapper.rs` |
| FR-005, FR-012 | champ `mcp` au registre ; équipiers : `mcpServers` de `session/new` ; interactifs : forme d'injection par session propre au CLI, constatée au spike-gate — jamais d'écriture de config persistante | `registry.rs`, `wrapper.rs` |
| FR-006 | prompt réduit pour agents branchés (blocs avant/après versionnés en fixtures) | `wrapper.rs` |
| FR-007 | `bridget_who(domain?)`, `bridget_ledger(view, limit?)` avec schémas séparés | `mcp.rs` |
| FR-008 | lecteur stdin unique + dispatch, writer stdout sérialisé (motif D-204), connexion daemon par appel, `cancelled` locale, EOF propre | `mcp.rs` |
| FR-009 | matrice de 15 cas en fixtures + pureté stdout | tests `mcp.rs` |
| FR-011 | taxonomie à trois niveaux (JSON-RPC / `isError` / métier fermé+extensible) au contrat | `contracts/outils-mcp.md` |

## Décisions de conception

**D-401 — Le serveur MCP est une façade, pas un acteur.** Il ne possède aucun
état **métier** ; chaque `tools/call` suit le protocole de connexion éphémère
réel du daemon, en **deux phases** : `Register`/`Registered` puis
commande/réponse — sous un **budget total de 10 s** avec délais distincts
(connexion, enregistrement, commande). Sémantique d'échec : avant l'écriture
du `Send` = échec certain (retry sûr) ; après écriture sans accusé =
`outcome_unknown(id)` ; une `notifications/cancelled` après écriture ferme
seulement l'attente locale et produit la même sémantique `unknown` — jamais
une annulation métier. Le serveur possède en revanche un **état technique
borné** : table des appels en vol (nettoyée sur réponse/annulation/EOF),
writer stdout, et **limite configurable basse d'appels simultanés**
(dépassement → `isError busy` avant toute connexion daemon) ; le lecteur
stdin reste disponible pendant les workers. Tests : saturation, réponses hors
ordre, EOF avec workers actifs. Tout comportement métier (garde-fous, cycle de
vie, refus) reste dans le daemon — c'est ce qui garantit SC-006.

**D-402 — Marqueurs de filiation enrichis, compatibles.** `agent-pids/<pid>`
passe d'un nom nu à une entrée typée (JSON : naissance du processus capturée
au dépôt, `instance_id`, chemin du fichier de nom). Le wrapper écrit le nouveau
format ; le lecteur traite l'ancien (nom nu) par une **erreur typée
`legacy_marker`** avec remédiation explicite : « agent lancé avant la mise à
niveau — redémarre l'agent pour activer MCP ». Pas de migration paresseuse :
sans naissance capturée à l'origine, une entrée migrée serait invalidable.
Conséquence documentée : **mise à niveau = redémarrage de l'agent requis** ;
l'ancien format est couvert par un test.

**D-403 — Spike-gate de branchement avant tasks, avec un faux serveur
jetable.** Les quatre voies de branchement ne sont pas testables sans un
serveur qui réponde au moins à `initialize`/`tools/list` : le spike livre donc
un **faux serveur MCP stdio minimal et jetable**, indépendant du daemon,
annonçant un unique outil `probe`. Chaque harness (versions pinnées) doit le
lancer par injection **strictement éphémère**, rappeler `tools/list` et
exécuter `probe`. Consignés : commande exacte, version, stdout/stderr,
**absence de diff des configurations utilisateur**, nettoyage. C'est une tâche
de recherche préalable — pas l'implémentation de `mcp.rs`. Un échec révise la
spec (gate FR-012), pas l'implémentation.

**D-404 — La couche de lecture est extraite dans un module neutre, pas dans
`cli.rs`.** Le rendu texte CLI et les données métier sont deux responsabilités :
les fonctions de lecture **typées et sans formatage** vivent dans un module
neutre du crate ; le daemon les utilise pour la requête ledger typée,
`mcp.rs` ne parle **qu'au daemon**, et `cli.rs` garde ses renderers. Pour les
demandes suivies, `ListRequests` existant est réutilisé tel quel. Non-régression
verrouillée par **golden tests** : stdout/stderr octet pour octet, ordre,
limite par défaut, ledger vide, erreur de base.

**D-405 — Deuxième usage JSON-RPC : primitives réutilisées, pas de couche
commune.** (Correction factuelle de la revue : le protocole daemon est du JSONL
sérialisé, pas du JSON-RPC — MCP n'est que le **deuxième** usage JSON-RPC
après le client ACP.) Le serveur MCP réutilise uniquement les primitives
**déjà publiques** du workspace (`BufReader` lignes, `serde_json`, écriture
sérialisée). Aucune factorisation en 010 ; au reuse-audit, extraire
`JsonRpcId`/enveloppe/écriture sérialisée **seulement si le diff prouve une
duplication identique** avec `acp.rs` — et ne jamais coupler les machines
d'état du client ACP et du serveur MCP (rôles inverses).

## Complexity Tracking

Aucune violation : zéro dépendance, un fichier source nouveau, extension du
registre existant. Le seul ajout de protocole daemon est la ou les requêtes
typées de lecture (si `ListRequests` ne suffit pas — à trancher au
reuse-audit avec preuve).
