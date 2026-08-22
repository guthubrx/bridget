# Contre-revue adverse — cxbridget (codex) — session 009

## Round 1 (spec.md)

**Date** : 2026-08-22 · **Verdict** : `BLOCKED` — 11 points, **tous retenus
(11/11, 0 rejeté)**. Le relecteur valide l'architecture centrale (reprise
déclarative avec autorité unique, contre des processus détachés) ; le blocage
portait sur les invariants masqués par des mots absolus (« jamais »,
« exactement », « zéro ») indéfendables en cas de crash ou de course.

| # | Point | Correction appliquée |
|---|---|---|
| 1 | « jamais d'orphelins » faux en cas de SIGKILL/crash/OOM | deux régimes : arrêt coopératif = zéro orphelin ; brutal = réconciliation au démarrage (groupes de processus marqués pgid+naissance+instance, anti-pid-recyclé, terminaison des groupes périmés avant reprise, test SIGKILL) |
| 2 | spawn synchrone sans machine d'états ni corrélation | `Requested→Reserved→Starting→Connected\|Failed\|Cancelled`, `command_id` idempotent, réservation atomique nom+slot, délai absolu, `Register` tardif rejeté, tests à barrières (FR-001) |
| 3 | état désiré sous-spécifié | invariants fixés : source unique, schéma versionné, clé stable, écriture atomique temp+fsync+rename, permissions, ordre déterministe, quota dépassé, conflit wrapper-terminal, zéro retry infini, matrice de reprise (FR-010) |
| 4 | stop pendant reprise indéfini (résurrection possible) | FR-010bis : stop gagne — génération/tombstone atomique, retrait durable avant réponse, `Register` obsolète rejeté, test à barrière |
| 5 | « aucun fichier résiduel » vs stderr conservée | résiduel = état **opérationnel** seulement ; journal de lancement exempté avec politique de rétention (FR-005) |
| 6 | environnement/cwd non définis | FR-011bis : cwd absolu capturé du client et validé, environnement construit par politique documentée (jamais copié du client), validations avant réservation finale, cas cwd disparu et PATH minimal |
| 7 | stop ne couvrait pas les descendants ; réponse indéfinie | FR-003 : `StopOutcome` synchrone après cancel+grâce+kill du **groupe**, wait/reap, drain, sous délai borné ; refus sur agent non-daemon-géré ; tests annulation ignorée et descendant npx |
| 8 | parité « exactement » non mesurable | FR-008 : matrice versionnée de garanties, même corpus dans les deux modes, tolérances et N fixés, comparaison de suites de frames pour attach (SC-004 aligné) |
| 9 | devenir des abonnements attach non spécifié | FR-011ter : `End` typé par abonnement à stop/mort/arrêt, nouveau `subscription_id` après reprise, ancienne génération jamais servie, test attach-pendant-stop-puis-reprise |
| 10 | FR-013 anticipait un champ MCP inexistant | réduite : le superviseur lance depuis `AgentDefinition` sans liste fermée d'options ; la 010 reste propriétaire du registre MCP |
| 11 | architecture reprise-déclarative validée, promesse à reformuler | reformulée selon le point 1 |

## Round 2

**Verdict** : `BLOCKED` (le choix architectural central est déclaré
approuvable) — 6 résidus, tous retenus (6/6) :

| # | Résidu | Correction |
|---|---|---|
| 1 | point de linéarisation état-désiré↔spawn non fixé (résurrection d'un Failed ou succès non persisté selon l'ordre d'écriture) | sémantique recommandée adoptée : pas de `Connected` avant écriture durable (fsync temp + rename + fsync répertoire) ; échec avant = aucune entrée, après = retrait durable avant `Failed` ; tests à points de crash |
| 2 | propriété du fichier d'état désiré ambiguë | daemon seul écrivain ; édition manuelle daemon arrêté seulement ; chargement au démarrage ; pas de hot reload (hors périmètre) |
| 3 | FR-013 encore spéculative (« toute extension future ») | réduite : réutilisation du chemin partagé 007 sans duplication de champs ; zéro promesse sur le futur ; la 010 modifiera le chemin partagé |
| 4 | SC-001 « 100 % » sans protocole | N=20, p95 < 10 s, 20/20 échanges, environnement gelé, timeout global |
| 5 | SC-003 partiel vs « tout échec » | table fermée de 8 familles, chacune motif typé + zéro état opérationnel |
| 6 | assumption contradictoire (« jamais d'orphelins ») | reformulée : coopératif = zéro ; crash = survivants bornés jusqu'à réconciliation |

## Round 3

**Verdict** : `APPROVE_WITH_CHANGES` — **BLOCKED levé**, feu vert pour le plan
à condition qu'il traite explicitement deux verrous (tous deux inscrits dans
la spec et à détailler au plan) :

1. **Invariant de couverture des marqueurs** : aucun enfant n'exécute avant la
   durabilité de son marqueur pgid (suggestion du relecteur : bootstrap retenu
   par pipe, libéré après fsync) — tests de crash aux trois frontières.
2. **Portée de l'idempotence de `command_id`** : registre borné d'issues à
   rétention fixée, survivant au redémarrage pour les persistants (mise à jour
   état désiré + issue transactionnelle) ; vie du daemon pour les éphémères.

**Bilan spec 009 : 3 rounds, 19 objections, 19 retenues, 0 rejetée.** Suite :
plan 009 (les deux verrous en décisions dédiées).

## Round 4 (plan.md)

**Verdict** : `BLOCKED` — D-501 approuvé sur le fond ; 10 objections, toutes
retenues (10/10). Les plus lourdes : (1) fermeture de pipe = signal ambigu
(EOF volontaire vs crash indistinguables) → protocole à octet `RELEASE` ; (2)
fork nu dans un daemon Rust multithread non async-signal-safe → sous-mode
exécutable `managed-bootstrap` (setsid, `Ready`, attente octet, exec) ; (3)
canal de statut structuré `Ready`/`StartupFailed` + handshake d'arrêt complet
(stderr jamais parsée) ; (4) `Connected` synthétisé depuis `fleet.json`
interdit → retry rattaché à la génération, attente du vrai terminal ; (5)
table de vérité fleet×issue×marqueur avec recovery déterministe +
`IdempotencyExpired` ; (6) `waitpid` → `ECHILD` sur non-enfants → `killpg` +
polling borné, marqueur supprimé après disparition confirmée ; (7) phase
`Recovering` visible, ordre commandes/reprises défini, stop-pendant-reprise
testé à barrière ; (8) `forbidden_env` sur l'environnement **source** avant
construction (sinon disparition silencieuse au lieu du refus T710) + baseline
portable + `pass_env` déclaratif par entrée, validé par quickstarts réels ;
(9) résolution `stop` primaire par table superviseur nom→génération,
marqueurs en secours ; (10) découpage en 3 modules
(fleet/managed_process/desired_state).

## Rounds 5-7 (plan)

- **Round 5** : `BLOCKED` — 4 points (2 contradictions de protocole : FD de
  statut vs CLOEXEC → discipline des FDs + hook `managed-status` +
  `BootstrapReady`≠`Connected` ; règle fermée `DaemonRecovering` ; 2
  nettoyages : ligne FR-003 et références périmées fork/module unique).
- **Round 6** : `BLOCKED` résiduel — 2 nettoyages normatifs locaux (D-502
  encore incohérent sur les FDs et `Ready`, « tel quel » restant).
- **Round 7** : **`APPROVE`** — plan clos.

**Bilan 009 : spec 3 rounds (19 objections) + plan 4 rounds (16 objections) =
35 objections, 35 retenues, 0 rejetée.** Suite : data-model + contrat +
quickstart, puis reuse-audit et tasks sur la base 008 finale.
