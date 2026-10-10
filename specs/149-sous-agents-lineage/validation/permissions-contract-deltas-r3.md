# Revue G-P r3 — deltas du contrat permissions depuis le plan r2

Date : 2026-10-10. Revueuse : GLM 5.3 Flash, indépendante. Ronde r3 ciblée.
Périmètre : les deux deltas seulement, dans `contracts/permissions.md`
(delta 1 : union v1/v2 de l'introspection, lignes 40–47 ; delta 2 : observer
propriétaire temporaire du CLI, lignes 323–352), avec `test-strategy.md`,
`tasks.md`, `spec.md`, rapports r1/r2 (`plan-permissions-r1.md`,
`plan-permissions-r2.md`). Vérification exacte contre le CLI local 2.1.296 et
la documentation officielle des hooks.
Verdict demandé : APPROVE ou REQUEST_CHANGES sur les deux deltas.

**Verdict : REQUEST_CHANGES ciblé.** Aucune objection d'architecture. Les deux
deltas sont fidèles au wire réel et réalisables. Deux défauts nouveaux, bornés
aux deltas : une ambiguïté d'une ligne dans la condition du succès v1
(G-P-07, moyen), et l'absence totale d'oracle pour l'observer (G-P-08, majeur
de couverture). Les six findings r1 restent fermés ; aucun n'est rouvert.

Cadre de cette revue. Elle juge les deux deltas, pas le code. Le code natif
est en cours chez Sol ; il n'est pas review ici. Une seule lecture ciblée de
`native_permission_observer.rs` a servi à confirmer la réalisabilité, sans
jugement. Aucun test, aucun build, aucun lint, aucun modèle lancé. Aucun Git,
aucune config, aucune base touchés. Aucune tâche cochée. Ce rapport est le
seul fichier écrit. La revue n'a relancé ni la revue G-L, ni la revue code T3,
ni les tests T3 — trois travaux distincts en cours. Aucun secret imprimé ; le
launcher n'a pas été relu (empreinte et ligne `exec claude "$@"` déjà prouvées
en r1 et r2).

Aucun test vert n'est exigé avant le code. Les oracles demandés ci-dessous
restent des obligations de livraison : T015, T037, T038, relecture T041
(`tasks.md:31,62,63,66`). Leur présence ferme le gate ; leur exécution
conditionne la livraison. C'est le même standard que r2.

## 1. Faits vérifiés à la source (r3)

Lecture seule. CLI local et documentation officielle des hooks.

| Fait | Sonde r3 | Résultat |
|---|---|---|
| Version CLI locale | `claude --version` | `2.1.296 (Claude Code)`. |
| Sémantique `--settings` | `claude --help` | « Path to a settings JSON file or a JSON string to load **additional** settings from ». Composition avec les autres sources, pas remplacement. Le refus contrat (`permissions.md:336`) vise le bon risque. |
| Hooks d'un fichier settings | `claude --help` (option `--bare`) | `--bare` saute « hooks defined in settings ». Donc hors `--bare`, un hook posé via `--settings` s'applique. |
| Modes de permission CLI | `claude --help` | choices : `acceptEdits, auto, bypassPermissions, manual, dontAsk, plan`. Le CLI expose bien `manual` ; la normalisation `manual→default` du contrat (`permissions.md:172`) est vérifiable localement. |
| Refus non interactif | `claude --help` | `--permission-prompts <target>` avec `host` ou `none` ; `none` nie automatiquement ce qui aurait demandé un prompt. Fidèle au contrat (`permissions.md:380-386`). |
| Champs d'input d'un hook | Doc officielle hooks (code.claude.com/docs/en/hooks) | Champs communs : `session_id`, `prompt_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`. `prompt_id` existe ; il est absent seulement avant la première entrée utilisateur. Le contrat (`permissions.md:338`) est fidèle au wire. |
| Blocage par un hook | Doc officielle hooks | PreToolUse exit 2 bloque l'appel ; timeout 600 s pour un hook command. L'exigence « ACK avant continuation » (`permissions.md:342-343`) est réalisable. |
| Réalisabilité observer | Lecture ciblée de `crates/bridget-daemon/src/native_permission_observer.rs` (symboles, hors review) | Socket Unix 0600, fichier overlay 0600 `create_new`, nonce, ACK par identifiant d'observation, transmission `permission_mode/session_id/prompt_id/cwd`, suppression socket+overlay à la sortie. Mêmes primitives que le contrat. |

La chaîne launcher/CLI résolu et le registre vivant ne sont pas refaits : les
empreintes et le mécanisme de sélection sont prouvés en r1 §2 et re-vérifiés
en r2 §1.

## 2. Delta 1 — union v1/v2 de l'introspection (`permissions.md:40-47`)

### Ce qui est correct

Le texte porte exactement la sémantique demandée. Le succès v1, identité 148
seule, est une branche de compatibilité, jamais un repli. Un fait invalide,
périmé, contradictoire ou révoqué produit un refus ; il ne retombe jamais en
v1 (`permissions.md:42-43`). Une nouvelle admission 149 inherit/development
exige v2 ; v1 donne `permission_attestation_unavailable`, sans grant ni
fallback discovery (`permissions.md:44-46`). Identité, status/cancel et
lectures 148 continuent (`permissions.md:44`). La section Compatibilité le
répète côté gate : aucun downgrade v1 permissif après une preuve 149 invalide
(`permissions.md:441-444`). Le registre lie le fait au credential courant et
le retire à end/fail/close (`permissions.md:272-275`). L'inventaire nomme le
refus (`permissions.md:400`).

### G-P-07 — moyen — La condition du succès v1 est ambiguë et l'union n'a aucun oracle

- Constat 1. `permissions.md:40-41` écrit : « l'enveloppe148 version1,
  identité seule, reste possible tant qu'aucun fait de permissions n'existe
  encore ». La lecture « n'existe plus » est possible. Or le registre retire
  le fait à end/fail/close (`permissions.md:274-275`). Un credential avec un
  tombstone satisfait donc « aucun fait n'existe ». La ligne 43 interdit
  pourtant le v1 pour un fait périmé ou révoqué. Les deux phrases se
  contredisent sous cette lecture. L'intention r3 est fixée : v1 seulement si
  **aucun fait n'a jamais existé pour ce credential**.
- Risque. Après end ou rotation, une introspection rendrait v1 « identité
  seule ». Ce serait une information fausse — « aucune politique connue » —
  au lieu d'un refus nommé. L'admission 149 refuse quand même (v2 exigée),
  donc aucun élargissement de droits. Mais le principe « refus nommé, jamais
  rétrograde silencieux » serait violé, exactement la classe de bug que le
  contrat veut supprimer.
- Fix 1 (une ligne, `permissions.md:41`) : remplacer « tant qu'aucun fait de
  permissions n'existe encore » par « tant qu'aucun fait de permissions n'a
  jamais existé pour ce credential ». La ligne 43 couvre déjà le refus des
  autres cas ; rien d'autre à changer.
- Constat 2. Aucune branche de l'union n'a de scénario. S149-26 prouve le
  retrait du fait côté registre, pas la sémantique d'introspection
  (`test-strategy.md:162`). S149-24(5) prouve une politique inconnue à
  l'admission, pas l'enveloppe v1 (`test-strategy.md:160`). Le test T3
  `BridgetSession.test.ts` prouve la v1 148 historique ; c'est une base de
  référence 148, pas un oracle 149 (`test-strategy.md:79`).
- Fix 2. Ajouter S149-32 à la stratégie (niveau : unitaire daemon + T3
  serveur, tests T3 en mission) et l'oracle G-P-07 dans la section Oracles du
  contrat. Texte exact en §4.

## 3. Delta 2 — observer propriétaire temporaire du CLI (`permissions.md:323-352`)

### Ce qui est correct et vérifié

Le contrat borne l'overlay aux seuls outils Bridget (`permissions.md:325`).
Il le sépare de `settings_overrides` : l'overlay n'ajoute qu'un hook command,
ne remplace pas les sources sélectionnées, ne fusionne aucune règle
(`permissions.md:326-336`). Il transmet seulement mode courant, session_id,
cwd, prompt_id et l'identifiant de la demande Bridget, avec une preuve opaque
propre au lancement (`permissions.md:338-340`). Le hook ne décide pas des
permissions fournisseur ; un auxiliaire ne publie jamais le fait ; le daemon
ackusé avant que le hook laisse continuer (`permissions.md:341-344`). L'overlay,
le nonce et la socket ne rejoignent ni snapshot, ni journal, ni enfant ;
sortie = fait retiré, socket fermée, overlay supprimé ; la reconnexion ne
récupère rien (`permissions.md:328,346-352`). Aucune policy autodéclarée :
le fait vient de la connexion primaire propriétaire, jamais d'un JSON
d'agent auxiliaire (`permissions.md:297-298`).

J'ai vérifié chaque primitive contre le CLI local et la doc officielle (§1) :
`--settings` est compositionnelle, les hooks de settings s'appliquent,
`prompt_id`/`session_id`/`cwd`/`permission_mode` sont dans l'input réel,
exit 2 bloque, le timeout laisse la place à l'ACK. La lecture ciblée du
fichier natif en cours montre les mêmes primitives côté implémentation.
Réalisabilité : confirmée. La recette reste obligatoire et le contrat ne
présume rien : « La recette doit confirmer la composition réelle des sources
par ce CLI. Un CLI qui remplace ces sources au lieu de composer l'overlay est
refusé » (`permissions.md:335-336`), et la voie PTY ne peut être déclarée
validée avant cette recette (`permissions.md:318-321`).

### G-P-08 — majeur (couverture) — L'observer n'a aucun oracle, contrat ni stratégie

- Constat. La stratégie ne contient aucun scénario sur le hook, l'overlay,
  l'ACK, la corrélation de la demande ou la suppression lifecycle. Un grep
  sur `hook|overlay|PreToolUse|observer|--settings` ne retourne que la
  constitution. La section Oracles du contrat porte G-P-01 et G-P-02
  (`permissions.md:416-438`) et rien pour l'observer. S149-14(a) et T038 ne
  nomment ni l'observer, ni l'ACK, ni la corrélation, ni la composition
  `--settings` (`test-strategy.md:131`, `tasks.md:63`). Le plan tient pourtant
  la recette PTY pour gate bloquante.
- Risque. C'est le défaut exact de G-P-01/G-P-02 en r1 : une exigence
  contractuelle centrale sans preuve définie. La recette pourrait être
  déclarée conforme sans prouver l'observer du tout — la partie du contrat
  qui porte le mode courant du fournisseur sur la voie Claude/GLM PTY
  (`permissions.md:307`).
- Fix 1. Contrat : ajouter l'oracle G-P-08 dans la section Oracles, après
  G-P-02 (`permissions.md`, avant `## Compatibilité et gate`, ligne ~439).
  Texte exact en §4.
- Fix 2. Stratégie : ajouter S149-33 (nouvelle §4.10) avec les quatre
  branches ci-dessous, plus les lignes de mapping FR/SC et une limite.
- Fix 3. `tasks.md` : T015 (`tasks.md:31`) nomme S149-32 et S149-33 ; T038
  (`tasks.md:63`) exige la preuve observer en recette : ACK avant appel,
  corrélation request_id, composition réelle `--settings` avec les sources
  sélectionnées, suppression lifecycle, overlay jamais dans l'enfant.
- Fix 4. Compteurs : deux scénarios nouveaux portent `spec.md:11`
  « Tests: 0/31 (0%) » à 0/33 et la stratégie passe de 31 à 33 scénarios
  dans son en-tête et sa table. Sans ce point, on recrée un G-P-06
  (compteurs périmés).

## 4. Texte exact des oracles à ajouter

Pour le contrat, section « Oracles de validation à implémenter par GLM »
(insérer après G-P-02, avant « ## Compatibilité et gate ») :

```text
G-P-07 : union d'introspection v1/v2. Trois branches. (a) Un credential
sans aucun fait de permissions jamais existé reçoit l'enveloppe148 v1,
identité seule, quatre champs camelCase, parseur fermé. (b) Un credential
dont le fait a été retiré (end/fail/close), révoqué ou tourné reçoit un
refus nommé permission_attestation_unavailable ; jamais l'enveloppe v1 ;
lecture et rejeu inchangés. (c) Une admission 149 inherit/development
présentée avec la seule enveloppe v1 rend permission_attestation_unavailable,
sans grant demandé, sans fallback discovery, sans spawn ; status, cancel et
identité148 continuent de fonctionner.

G-P-08 : observer propriétaire du chemin PTY. (a) Un lancement avec
l'overlay --settings publie un fait portant permission_mode, session_id,
cwd, prompt_id et l'identifiant de la demande Bridget ; le daemon accuse
réception avant que le hook laisse continuer l'appel observé ; la demande
délégataire correspond à l'identifiant observé. (b) Preuve absente,
non corrélée ou observer indisponible : l'appel Bridget est bloqué avec
son refus nommé ; aucun effet fournisseur, aucun grant, compteur launched
inchangé. (c) Un fait d'une autre demande ne donne aucun droit.
L'overlay, son nonce, la socket et la preuve n'apparaissent ni dans le
snapshot enfant, ni dans la définition ou l'environnement enfant, ni dans
le journal ; fichier et socket 0600 sont supprimés en sortie ; la
reconnexion ne récupère aucun fait d'une ancienne connexion.
```

Pour la stratégie, nouvelle §4.10 (table + mapping FR/SC + limite) :

```text
S149-32 | Union d'introspection v1/v2 (G-P-07) | Unitaire daemon + T3 serveur
| Branches (a), (b), (c) de G-P-07, mot à mot. Mapper FR007, FR008, FR019 /
SC003. Les tests T3 prouvent la sémantique de réponse ; ils ne prouvent pas
l'entrée du Rust autonome (voir §5).

S149-33 | Observer propriétaire du chemin PTY (G-P-08) | Fixture wrapper
+ recette réelle | Branches (a), (b), (c) de G-P-08 en fixture. Branche (d)
en recette réelle S149-14(a)/T038 : le CLI réel compose le fichier --settings
avec les sources sélectionnées — les règles des sources attestées restent
effectives avec l'overlay — et le hook s'applique en PTY autonome sans
approbation humaine ; un CLI qui remplace les sources est refusé. Mapper
FR007, FR008, FR019 / SC003, SC004.
```

Ajustements associés : en-tête et table de la stratégie (31 → 33 scénarios),
table mapping FR/SC (`test-strategy.md:309-315`), §9.5 inchangé (aucun code
nouveau), `spec.md:11` (0/31 → 0/33), T015 et T038 comme en G-P-08/Fix 3.

## 5. Interop T3 et preuve d'entrée du Rust autonome

Les preuves d'interop T3↔Rust déjà acquises (interop R4, régression
d'identité T3, tests T3 en mission) attestent le transport entre processus
déjà établis. Elles ne constituent pas la preuve d'entrée du Rust autonome.
La voie standalone, premier parent externe réel avec T3 absent, reste prouvée
par la recette S149-14(a) et T038 seulement. S149-32 et S149-33, au niveau
T3/fixture, prouvent la sémantique des refus et des corrélations ; ils ne
remplacent jamais cette recette. Le contrat le dit déjà
(`permissions.md:318-321,446-450`) ; les deux oracles nouveaux respectent
cette limite.

## 6. Axes constitution — delta depuis r2

### Complexité (article XVIII)

Conforme. L'observer est un fichier, une socket et un thread par lancement,
bornés, nettoyés en sortie. Aucun état global, aucune boucle, aucun serveur
d'autorité nouveau. L'union v1/v2 est une branche de compatibilité, pas une
structure nouvelle.

### Minimalisme et Frugalité (article XIX)

Conforme. La séparation overlay / `settings_overrides` évite de mêler le
mécanisme d'observation à l'attestation de permission. Deux scénarios
nouveaux, pas davantage. L'oracle G-P-07 se règle avec une ligne de contrat.

### Vertus LLM et Responsabilité future (article XX)

Renforcé par les deux deltas. L'interdiction de downgrade supprime la classe
« rétrograde silencieux après tombstone ». L'observer supprime la classe
« mode fournisseur supposé sans source ». G-P-07 est nécessaire pour que la
règle de l'union soit exécutable par un lecteur futur sans interprétation.

## 7. Remarques non bloquantes

1. `permissions.md:330` dit « dans son namespace ». macOS n'a pas de
   namespaces. L'intention est un répertoire privé du lancement, avec fichier
   et socket 0600. Une reprise en deux mots éviterait une fausse lecture OS.
2. La doc officielle note que `prompt_id` est absent avant la première entrée
   utilisateur. Un PreToolUse s'exécute toujours après cette entrée, donc le
   champ est présent en pratique. Le contrat peut consigner en une ligne le
   traitement du cas absent : refus nommé, jamais une publication partielle.
3. Le help local confirme que `--settings` accepte aussi une chaîne JSON. Le
   choix du chemin de fichier temporaire (`permissions.md:330-333`) est le bon
   : auditable, privé 0600, supprimable. Rien à changer.
4. Le hook `__native-permission-observer` observé dans le code en cours
   transmet aussi `tool_name` et `tool_input` au-delà des cinq champs du
   contrat (`permissions.md:338`). C'est un détail d'implémentation du code
   natif, hors périmètre de cette revue ; le contrat dit « uniquement » pour
   les champs du fait publié, ce qui reste la règle qui compte.

## 8. Limites de cette revue

- Revue ciblée des deux deltas. Le reste du contrat, la revue G-L, la revue
  code T3 et les tests T3 ne sont pas jugés ici.
- Les sondes CLI et doc décrivent le CLI 2.1.296 de cette machine et la doc
  publique à cette date. La composition réelle `--settings` + sources
  sélectionnées reste à prouver par la recette ; elle n'est pas présumée
  positive, par le contrat comme par cette revue.
- Le code natif en cours n'est pas review. La lecture de
  `native_permission_observer.rs` a servi seulement à confirmer que les
  primitives du contrat existent.
- Les numéros de ligne valent pour les fichiers lus le 2026-10-10 dans le
  worktree `149-sous-agents-lineage` (`permissions.md` 09:59,
  `test-strategy.md` 09:18, `tasks.md` 09:18, `spec.md` 09:17).

## 9. Conclusion et conditions de levée

REQUEST_CHANGES ciblé sur les deux deltas. Les six findings r1 restent fermés.

REQUEST_CHANGES levé quand :

1. G-P-07 : `permissions.md:41` porte « n'a jamais existé pour ce
   credential », et S149-32 est ajouté à la stratégie avec les trois
   branches, nommé dans T015.
2. G-P-08 : l'oracle G-P-08 est ajouté au contrat, S149-33 à la stratégie
   avec ses quatre branches, T015 et T038 nomment l'observer et la
   composition `--settings` réelle, et les compteurs (`spec.md:11`,
   en-tête stratégie) passent à 33.

Aucun autre point des deltas n'est à rouvrir. Les fermetures sont
documentaires et de couverture ; aucune architecture ne bouge. Après ces
deux points, le volet G-P peut suivre le GO du principal, avec les oracles
restés obligatoires en T015, T037, T038 et la relecture T041.
