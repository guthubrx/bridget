# Revue G-P r2 — plan permissions, attestation et héritage

Date : 2026-10-10. Revueuse : GLM 5.3 Flash, indépendante. Ronde r2.
Périmètre : `contracts/permissions.md`, `plan.md`, `tasks.md`, `spec.md`,
`data-model.md`, `test-strategy.md`, `research-native.md`, `research-t3.md`,
ADR149 (`docs/decisions/149-permissions-heritees-et-lineage-native.md`),
rapport r1 (`validation/plan-permissions-r1.md`), sondes ciblées du code et de
l'environnement en lecture seule.
Verdict demandé : APPROVE ou REQUEST_CHANGES sur le volet G-P.

**Verdict : APPROVE.** Les six findings r1 sont fermés par des changements
concrets, vérifiés dans les fichiers actuels, pas seulement déclarés. La
fermeture porte sur le plan : les oracles sont définis et alignés maintenant ;
leur écriture et leur exécution restent des tâches futures T015, T037, T038,
T041, avant livraison. Aucune objection architecturale nouvelle.

Cadre de cette revue. C'est une revue de plan avant tout code de permissions.
Aucun oracle exécuté n'est exigé ici. Le contrat le dit lui-même :
`contracts/permissions.md:374` — « Leur présence ferme le gate du plan. Les
preuves d'exécution sont attendues après implémentation, avant livraison. »
Exiger des tests verts maintenant créerait une dépendance circulaire.
Aucune tâche n'est cochée. Aucun test149 n'est déclaré vert.

Cette revue n'a lancé aucun test, aucun build, aucun lint, aucun modèle.
Elle n'a touché aucun Git, aucune config ou base de production, aucun service.
Elle n'a pas relancé la revue G-L. Le seul fichier écrit est ce rapport.
Deux agents Sol implémentent le lot Lineage approuvé ; leurs fichiers et les
documents existants n'ont pas été modifiés.

## 1. Faits re-vérifiés à la source (r2)

Lecture seule. Aucun secret, aucun contenu de config, aucun environnement
complet imprimé.

| Fait | Sonde r2 | Résultat |
|---|---|---|
| Service Bridget actif | `launchctl print gui/501/com.bridget.daemon` | `state=running`, `pid=58394`, `BRIDGET_HOME=/Users/moi/.cache/bridget-core`, `BRIDGET_SOCKET=/Users/moi/.cache/bridget-core/bridget.sock`. Identique à la preuve consignée. |
| Registre vivant sélectionné | Extraction `jq` ciblée de `/Users/moi/.cache/bridget-core/agents.json` | Entrée `glm` : `command=/Users/moi/.local/bin/gclaude`, `claude_config_dir=/Users/moi/.claude-glm`. Entrée codex-pro présente. |
| Registre non sélectionné | Empreinte de `/Users/moi/.config/bridget/agents.json` | SHA256 `6fee943e850817ee01138743f9c9af058ba9adf680b00316f596d676fcd33d5d`, valeur consignée. Ce fichier ne porte pas d'entrée `glm`. |
| Empreintes | `shasum -a 256` | Registre actif `c8642f4c08a5…539`, launcher `gclaude` `dd8dee5677e4…a8e`. Identiques aux valeurs de `research-native.md:85-93` et du rapport r1 §2. |
| Chaîne de sélection du registre | `crates/bridget-daemon/src/environment.rs:21-39`, `crates/bridget-daemon/src/registry.rs:913-916` | `Namespace::from_environment` lit `BRIDGET_HOME` (aucun repli temporaire) ; `config_path()` joint `root/agents.json`. Le daemon actif lit donc `/Users/moi/.cache/bridget-core/agents.json`. |

Conclusion G-P-05 : le fait est confirmé à la source, indépendamment du texte
des documents. La cible `glm` native existe dans le registre réellement
sélectionné. Le constat erroné de la r1 (registre `~/.config/bridget` sans
entrée `glm`) est corrigé et expliqué dans `research-native.md:83`.

## 2. Fermeture des findings r1

Sévérités r1 : G-P-01, G-P-02 majeurs ; G-P-03 à G-P-05 moyens ; G-P-06 mineur.

### G-P-01 — fermé — Oracle du refus corrélé `can_use_tool` et de l'échec explicite

- Contrat : nouvelle section « Oracles de validation »,
  `contracts/permissions.md:377-387`. Oracle précis : fixture émet une trame
  `can_use_tool` pour un outil hors politique avec `request_id` connu ; la
  seule control_response porte ce `request_id` et une décision deny ; la
  réponse corrélée contient `permission_denials` puis une fin de tour ; la
  tâche devient `failed` avec `provider_permission_denied` ou
  `permission_not_inherited` selon le point de refus ; elle n'atteint jamais
  `result_available` et n'émet aucun résultat de succès ; rejeu et lecture
  n'émettent ni réponse ni lancement supplémentaire ; compteur `launched`
  inchangé ; variante réelle optionnelle, limite consignée.
- Stratégie : S149-29 (`test-strategy.md:175`) reprend ces oracles un à un,
  y compris « zéro résultat corrélé de succès » même avec un texte de modèle
  « OK », et le mappage FR008/SC003 (`test-strategy.md:313`).
- Tâches : T015 (`tasks.md:31`) nomme S149-29 et l'objectif « aucun faux
  succès ni lancement supplémentaire ». T012 (`tasks.md:28`) porte le code :
  « un refus ne devient jamais faux succès » dans
  `crates/bridget-transport/src/claude_stream_json.rs`, le fichier qui ignore
  ces trames aujourd'hui (constat r1 confirmé, `research-native.md:36`).
- Le désaccord r1 entre contrat et stratégie a disparu : contrat, stratégie,
  tâches et modèle de données décrivent le même comportement
  (`data-model.md:113` — « Une sortie de processus ou une fin de tour seule
  ne prouve pas result_available »).

### G-P-02 — fermé — Oracle `settings_revision_changed`

- Contrat : oracle `contracts/permissions.md:389-398` : trois variantes
  indépendantes (source mutée, supprimée, créée), matrice rejouée après
  réouverture du magasin et avant reprise, variantes launcher remplacé, binaire
  CLI remplacé, symlink retargeté, résolution déplacée par PATH ; `launched`
  inchangé ; aucun retry n'adopte ; recette positive avec les deux digests
  réellement résolus.
- Mécanisme et précision demandée : `contracts/permissions.md:222-228`.
  L'owner refait une résolution fraîche depuis les inputs de lancement réels.
  Il compare aux couples figés. Il n'adopte pas la nouvelle résolution et ne
  lance aucun enfant. `data-model.md:66-71` : « sans cache ni adoption d'une
  nouvelle résolution », « sans remplacer le snapshot par le nouveau fichier ».
  La résolution fraîche sert donc uniquement à comparer au snapshot initial.
- Stratégie : S149-30 (`test-strategy.md:176`) : (a) unitaire — la fonction de
  recontrôle rend `settings_revision_changed` pour un fichier muté, supprimé,
  ajouté, sans re-résolution ; (b) fixture — mutation avant premier spawn puis
  à la reprise, symlink retargeté, priorité PATH, aucune adoption, compteur
  inchangé. Mappage FR019/SC003 (`test-strategy.md:314`). T015 le nomme aussi.

### G-P-03 — fermé — `permission_source_unavailable` dans un inventaire unique

- Contrat : nouvelle section « Inventaire normatif des refus de permissions »,
  `contracts/permissions.md:354-370`. Sept codes, dont
  `permission_source_unavailable` (`:362`), avec la cause « source nécessaire
  opaque/non observable » et la borne « Ne signifie pas refus de tous les
  profils Claude ».
- Tableau de mappage : `contracts/permissions.md:331` utilise désormais
  `permission_source_unavailable` pour ce cas. J'ai vérifié chaque code du
  tableau (`:323-333`) : tous figurent dans l'inventaire. Aucun code hors
  inventaire ne reste.
- Stratégie : §9.5 (`test-strategy.md:274`) déclare ce nom unique ; S149-31
  (`test-strategy.md:177`) est l'oracle dédié, avec le garde-fou « le chemin
  positif de même famille avec sources valides reste inchangé : aucun refus
  global des parents Claude ». Mappage FR007/FR008/FR019/SC003.
- Plan : `plan.md:70` cite l'inventaire comme source unique et interdit le
  refus global. Contrat `:257-260` borne aussi le refus à son contexte.

### G-P-04 — fermé — CLI résolu gelé et recontrôlé

- Contrat : `contracts/permissions.md:139-141` ajoute
  `resolved_cli_path` et `resolved_cli_revision` à `launch_context` ;
  `:197-201` définit les deux couples (launcher et binaire exécuté),
  obligatoires ; `:213-220` décrit la résolution : environnement final de
  lancement, PATH effectif, publication du seul couple chemin/digest, jamais
  de l'environnement ; fraîche, sans cache ; cas `gclaude` documenté
  (`exec claude "$@"`), `realpath` puis SHA256 ; un wrapper seul n'est jamais
  une preuve ; une chaîne non observable rend `permission_source_unavailable`
  pour cette voie seulement ; `:222-228` impose le recontrôle des deux couples
  avant publication, admission, premier lancement et reprise, et interdit au
  PATH d'être un argument MCP ou un credential de tâche.
- Réalisabilité sans fuite ni adoption : vérifiée. La résolution reste locale
  au processus qui lance (launcher lisible, `realpath`, SHA256). Seuls chemins
  canoniques et digests sont publiés. Aucun cache. Toute divergence rend
  `settings_revision_changed` sans adoption.
- Diffusion cohérente : `data-model.md:64-71` (les deux couples vérifiés
  ensemble avant effet) ; ADR `docs/decisions/149-permissions-heritees-et-lineage-native.md:23-26` ;
  `plan.md:70` ; tâches T007, T011, T012 (`tasks.md:23,27,28`) ; stratégie
  §7 (`test-strategy.md:250`) et T038 (`tasks.md:63`) ; preuve locale initiale
  dans `research-native.md:92-104`, avec sa limite honnête : cette lecture de
  fichiers ne prouve pas le PATH effectif d'un tour réel ; l'owner doit
  résoudre sous son environnement final.

### G-P-05 — fermé — Registre source réel de la recette, consigné

- Preuve propriétaire consignée dans trois documents :
  `research-native.md:59-104` (section dédiée « correction G-P-05 » :
  launchctl, mécanisme de sélection avec sources de code, tableau des deux
  registres, empreintes, correction explicite du constat r1 `:83`) ;
  `plan.md:72` ; `test-strategy.md:251`. Contrat : `permissions.md:247-253`
  documente le cas réel `gclaude` + `/Users/moi/.claude-glm`.
- Re-vérification indépendante : voir §1. Le couple launcher/profil de
  l'entrée `glm` du registre vivant est exactement celui du contrat et de la
  voie same-family.
- Tâches : T003 (`tasks.md:16`) consigne le registre source réel (chemin
  absolu, entrée `glm` exacte, digests launcher et CLI résolu) et note
  qu'aucune entrée de production n'est ajoutée sans décision humaine. T038
  (`tasks.md:63`) fait de même pour la recette standalone. Le registre de
  recette reste un clone privé 0o600 avec le modèle exact `glm-5.3-flash`
  (`test-strategy.md:251-252`) ; le modèle déclaré du registre de production
  n'autorise aucun repli hors Flash.
- Aucune modification de production : la sonde r2 n'a lu que des JSON et des
  empreintes. Aucun daemon relancé.

### G-P-06 — fermé — Marqueurs de spec réconciliés

- `spec.md:19-21` : les trois fichiers sont marqués ✓. `spec.md:10-11` :
  « Tâches: 0/45 (0%) », « Tests: 0/31 (0%) », cohérent avec les 45 tâches de
  `tasks.md` et les 31 scénarios de la stratégie.
- FR018 (`spec.md:190-193`) porte la forme corrigée : préserver les
  permissions effectives du parent, sa révocation et ses opt-outs ; ne retirer
  que la garde du grant humain supplémentaire dans l'héritage autorisé.
  FR019 (`spec.md:194-197`) fige droits, modèle et définition avant premier
  effet, sans élargissement au rejeu, sans effet implicite d'un changement de
  mode UI.
- La phrase ambiguë de `plan.md:39` signalée en r1 a disparu ; `plan.md:39`
  décrit FR018 au présent, conforme à la spec. `tasks.md:6` et `plan.md:5`
  portent l'état exact des gates : G-L APPROVE r3 acquis, G-P REQUEST_CHANGES
  r1 en réconciliation, gate en attente — état que cette r2 clôt si le
  principal suit.
- FR019 et le mapping complet vers les nouveaux scénarios sont en place
  (`tasks.md:107-109`, `test-strategy.md:283-318`).

## 3. Couverture des exigences demandées

| Exigence | Couverture r2 | Preuves principales |
|---|---|---|
| FR007 — bornes parent, aucun gain | Couverte | S149-01, 02, 06, 07, 10, 15, 24, 25, 31 ; invariant 5 du plan. |
| FR008 — lecture/écriture GLM+Codex, sans grant, héritage, refus nommés | Couverte | S149-06, 07, 10, 14, 15, 24, 29, 31 ; T011-T015, T037-T038. Le refus en cours de mission a désormais son oracle (S149-29). |
| FR009 — identique sans T3 | Couverte | S149-12, 14 variante (a), T038 ; SC004 ; invariant 6. |
| FR011 — opt-outs et révocation, aucune projection sans preuve | Couverte | S149-03, 13, 19, 20, 21, 25, 26 ; contrat `permissions.md:32-34`. |
| FR012 — choix humains MCP conservés, grants hors MCP | Couverte | S149-03, 06, 21, 22 ; T005-T009, T013 ; règle transversale correction 4 : le grant pty reste hors de tout chemin positif 149. |
| FR018 — permissions préservées, retrait de garde limité à l'héritage | Couverte | Texte corrigé ; S149-02, 15, 20, 24 ; wrapper limité à la définition enfant (`plan.md:78`). |
| FR019 — figeage avant effet, retry sans élargissement, mode UI sans effet | Couverte | S149-09, 24, 26, 30, 31 ; contrat `:26-30` et `:222-228` ; `data-model.md:64-71`. La divergence de digest a désormais ses deux oracles. |
| SC003 — écriture permise réussit, hors politique refusée, découverte = lecture | Couverte | S149-10, 14, 15, 24, 29, 30, 31 ; découverte = réduction explicite (contrat `:15-16`, registre `--restricted --permission-mode plan`). |
| SC004 — même délégation sans T3 | Couverte | S149-12, S149-14(a), T038, avec séparation premier parent externe / descendant managed. |
| SC005 — opt-outs masquent, preuve révoquée ferme, moteur survit | Couverte | S149-03, 18, 19, 20, 21, 25, 26. |

Point de besoin « standalone, premier parent externe réel Codex et GLM, PTY
distinct des descendants managed et des fixtures » : la table des sources
standalone (`permissions.md:294-300`), le gate (`:407-411`), S149-14 variante
(a) et T038 couvrent ce cas. La règle correction 3 de la stratégie interdit de
le remplacer par un descendant managed ou une fixture.

Point de besoin « preuve relue hors verrou puis binding recontrôlé » :
`permissions.md:275-280` (réattestation hors verrou, HTTP sans redirection,
timeout 3 s, 64 KiB, session transport unique puis DELETE ; sous verrou,
recontrôle route primaire, identité/instance, binding, projet, révocation ; la
course ferme l'admission). T009 porte le code.

Point de besoin « aucune fuite credential/config/env » : `permissions.md:197-207`
(seuls chemin/digest publiés), `:213-215` (résolution sans publication
d'environnement), `:282-285` (Debug masqué, aucun token, endpoint, credential,
prompt ou environnement complet dans tâche, définition, CLI visible, erreur,
log ou enfant). T009, T013, S149-11.

## 4. Axes constitution — delta depuis r1

### Complexité (article XVIII)

Conforme, comme en r1. Les ajouts r2 n'introduisent aucune structure nouvelle :
le recontrôle des digests est une comparaison bornée par source ; les trois
scénarios S149-29 à S149-31 sont des tests, pas de l'architecture. Aucune
double boucle ni N+1 proposé.

### Minimalisme et Frugalité (article XIX)

Amélioré. Le fix structurel demandé en r1 est appliqué : l'inventaire
normatif (`permissions.md:354-370`) est devenu la source unique des codes de
refus. Les autres documents citent l'inventaire sans redéfinir (`plan.md:70`,
`test-strategy.md:274`). La divergence qui produisait G-P-03 n'existe plus.
La redondance documentaire générale reste, acceptée comme checkpoint de gate.

### Vertus LLM et Responsabilité future (article XX)

Renforcé. Le gel du CLI résolu supprime une classe entière de divergences
silencieuses : une rotation d'auth ou une mise à jour du CLI produit un refus
conservateur nommé, jamais des droits supplémentaires (`permissions.md:205-207`).
Les refus restent nommés et limités à leur contexte. Un manque de preuve reste
un refus, pas une invention.

## 5. Remarques de suivi — non bloquantes

1. Matrice S149-30 : l'oracle contrat G-P-02 (`permissions.md:394-396`)
   énumère quatre variantes launcher/CLI (remplacé, binaire remplacé, symlink
   retargeté, PATH déplacé). S149-30 nomme explicitement le retarget de
   symlink et la priorité PATH ; les remplacements de launcher et de binaire
   sont couverts par la famille « chemin/digest du launcher CLI » et par
   l'oracle unitaire (a). Au moment d'écrire les tests (T015), énumérer les
   quatre variantes nommément, pour égaler mot à mot la matrice du contrat.
2. Champ « modèle déclaré » du registre vivant : ma sonde `jq` ciblée n'a pas
   ressorti de clé `model` à l'endroit projeté ; la valeur `glm-5.3` reste
   celle consignée par l'owner natif (`research-native.md:82`). Sans incidence
   : Bridget fige le modèle de mission exact et la recette exige
   `glm-5.3-flash` sans repli. À relire en T003 lors de la consignation du
   registre source réel.
3. T001 (`tasks.md:14`) reste la clôture formelle documentaire au moment du
   GO. Les corrections G-P-06 sont déjà en place dans les fichiers actuels.
4. Note pour le principal : un texte de correction antérieur affirmait que les
   contrats restaient à corriger. Les fichiers actuels montrent déjà le CLI
   résolu, l'inventaire unique et les oracles. Ce rapport juge les fichiers
   lus, pas ce texte déclaratif.

## 6. Limites de cette revue

### Conception

- Revue documentaire et sondes en lecture seule. Aucun test, aucun build,
  aucune recette exécutés. Aucun code de permissions écrit.
- Les contrats restent non figés jusqu'au GO principal. Les numéros de ligne
  valent pour les versions lues le 2026-10-10 dans le worktree
  `149-sous-agents-lineage`.
- L'attestation standalone Claude PTY ne peut pas être déclarée validée par
  un plan. Le contrat l'exige (`permissions.md:310-313`) et le plan la tient
  pour gate bloquante (`plan.md:150`). La recette du premier parent GLM ferme
  ce point après code.

### Fixtures

- S149-29, S149-30, S149-31 prouvent des refus nommés, des corrélations et des
  compteurs sur des fixtures. Ils ne prouvent aucun comportement du fournisseur
  réel, aucun confinement OS. La trame `can_use_tool` réelle reste optionnelle,
  avec limite consignée (`test-strategy.md:175,187`).
- Les attestations parents des fixtures sont des signatures ; elles ne valent
  jamais preuve externe (règle correction 3).

### Modèles réels

- Le refus corrélé réel, l'écriture réelle GLM `glm-5.3-flash` et Codex, la
  recette standalone avec T3 absent restent attendus en T015, T037, T038, puis
  relecture T041. Aucun n'est exigé avant le GO code, aucun n'est affirmé ici.
- Le modèle exigé reste exactement `glm-5.3-flash`, sans repli
  (`test-strategy.md:252`). Toute autre valeur fait échouer le test.
- Les constats d'environnement du §1 décrivent cette machine à cette date.
  Ils ne prouvent pas un comportement fournisseur au-delà des inputs mesurés.

## 7. Conclusion

REQUEST_CHANGES r1 est levé. Les conditions posées en r1 (§8 du rapport r1)
sont remplies une à une :

1. G-P-01 : oracle fixture complet, corrélé par `request_id`, faux succès
   impossible, compteur de lancements surveillé. S149-29 + T015.
2. G-P-02 : deux oracles, unitaire et fixture, avant lancement et à la
   reprise, sans adoption. S149-30 + T015 + `data-model.md:66-71`.
3. G-P-03 : un seul nom par cas, dans l'inventaire unique, le tableau de
   mappage et un oracle dédié. S149-31.
4. G-P-04 et G-P-05 : CLI résolu gelé et recontrôlé partout ; registre source
   réel consigné avec preuve re-vérifiée indépendamment le 2026-10-10.
5. G-P-06 : fiche, compteurs, FR018, FR019 et phrase de plan réconciliés.

La gate G-P peut passer au GO du principal. Le code de permissions et
d'héritage (T004-T015) peut alors démarrer. Les oracles restent des
obligations de livraison : leur exécution prouvée en T015, T037, T038 et leur
relecture T041 conditionnent la clôture de la session, pas ce GO.
