# Revue G-P r1 — plan permissions, attestation et héritage

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-permissions-r1.md#extrait-1).

**Verdict : REQUEST_CHANGES.** Le contrat est solide et fidèle au wire réel.
Deux exigences contractuelles n'ont aucun oracle de test. Deux preuves
d'environnement manquent. Aucun blocage d'architecture. Les corrections sont
documentaires et de couverture, pas de conception.

Ronde r1. Aucun test exécuté, aucun build, aucune recette réelle. Aucun code ni
document de contrat modifié. Ce rapport est le seul fichier écrit par cette revue.
Aucune tâche n'est cochée par cette revue. Aucun compteur de relecture n'est inventé.

## 1. Preuves vérifiées à la source

Lecture seule. Les valeurs de secrets ne sont jamais imprimées.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-permissions-r1.md#extrait-2).

## 2. Preuves d'environnement local (conception, pas preuve fournisseur)

Ces constats décrivent cette machine. Ils ne prouvent pas un comportement
fournisseur réel. Ils ne couvrent pas tous les contextes futurs.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-permissions-r1.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-permissions-r1.md#extrait-4).

## 3. Findings

Sévérités : majeur = bloque la gate concernée ; moyen = corriger avant code du
volet ; mineur = à réconcilier avant clôture T001.

### G-P-01 — majeur — Aucun oracle pour le refus corrélé `can_use_tool` et l'échec explicite `permission_denials`

- Contrat : `contracts/permissions.md:313-321` exige un refus automatique corrélé
  de toute trame `can_use_tool` non couverte, et un échec terminal nommé
  (`permission_not_inherited` ou `provider_permission_denied`). La fin de tour
  seule ne prouve jamais un succès.
- État réel : le pilote natif ignore ces trames (`claude_stream_json.rs`, zéro
  occurrence). C'est un comportement nouveau, prévu par T012 (`tasks.md:28`).
- Trou : la stratégie n'a aucun scénario pour ce chemin. S149-14 écrit un fichier
  autorisé. S149-15 refuse avant spawn. Rien ne prouve le refus en cours de
  mission ni l'impossibilité d'un faux succès.
- Risque : une mission bloquée par une permission se termine « normale ».
  SC003 « une action hors politique est refusée côté enfant » reste sans preuve.
- Fix concret : ajouter un scénario fixture E2E (S149-28). Le fournisseur factice
  émet une trame `can_use_tool` pour une action hors politique. Oracle : refus
  corrélé au `request_id` de la trame, état tâche `failed` avec code nommé,
  zéro résultat de succès, compteur de lancements inchangé. Mapper FR008/SC003.
  Une variante réelle reste optionnelle si la trame n'est pas déclenchable chez
  le fournisseur réel ; la limite serait consignée.

### G-P-02 — majeur — Aucun oracle pour `settings_revision_changed`

- Contrat : `contracts/permissions.md:211-216` fige launcher, profil, cwd,
  sources et digests. Toute divergence avant premier lancement ou à une reprise
  rend `settings_revision_changed`, sans adopter le nouveau fichier.
- État de la stratégie : S149-09 vérifie le snapshot inchangé au rejeu.
  S149-26 vérifie la revision et le mode Claude. Aucun scénario ne modifie un
  digest (launcher ou fichier source) entre admission et lancement ou reprise,
  puis n'attend le refus nommé.
- Risque : la garde centrale de FR019 (« un rejeu retourne la tâche initiale et
  n'élargit pas les droits » côté sources) n'a pas de preuve négative.
- Fix concret : deux oracles. (a) Unitaire : fonction de recontrôle des digests
  testée avec un fichier muté, un fichier supprimé et un fichier ajouté →
  `settings_revision_changed` dans les trois cas, aucune re-résolution.
  (b) Fixture E2E : mutation du fichier source de la fixture entre admission et
  premier lancement, puis à une reprise → refus nommé, compteur de lancements
  inchangé. Mapper FR019/SC003.

### G-P-03 — moyen — Code de refus `permission_source_unavailable` hors inventaire

- `contracts/permissions.md:229` nomme `permission_source_unavailable` pour les
  sources managed opaques. Le tableau de mappage (`permissions.md:301`) rend les
  mêmes cas avec `permission_attestation_unavailable` ou
  `permission_mapping_unavailable`. La stratégie (§9.5) liste quatre codes et
  aucun scénario ne nomme `permission_source_unavailable`.
- Risque : un implémenteur rend un code que les tests n'attendent pas, ou
  l'inverse. La divergence entre deux sections du même contrat est déjà réelle.
- Fix concret : soit définir `permission_source_unavailable` dans l'inventaire
  des codes du contrat, l'ajouter à la ligne 301 du tableau et à un oracle
  dédié ; soit supprimer ce nom et unifier sur `permission_mapping_unavailable`
  limité au contexte managed opaque. Une seule source de vérité pour les codes.

### G-P-04 — moyen — `cli_revision` ne couvre pas le CLI résolu

- `contracts/permissions.md:189` fige « SHA256 des octets du launcher choisi ».
  Le launcher réel `gclaude` termine par `exec claude "$@"` (ligne 30), résolu
  via PATH. Le binaire réellement exécuté (`/Users/moi/.local/bin/claude`,
  empreinte relevée en §2) n'est pas dans la preuve.
- Risque : une mise à jour légitime du CLI entre admission et lancement, ou
  entre deux reprises, change l'application des règles sans déclencher
  `settings_revision_changed`. La promesse « mêmes inputs » perd un input.
- Fix concret : ajouter dans `launch_context` le binaire résolu avec son
  `sha256`, capturé par l'adaptateur propriétaire, recontrôlé comme les autres
  digests. Alternative minimale : épingler le chemin absolu du CLI dans la
  définition native (le modèle existe : l'entrée `claude` du registre vivant
  utilise déjà un chemin absolu). Le critère « hash de commande ET hash du CLI
  réel » est alors couvert sur les deux contextes.

### G-P-05 — moyen — Aucune cible `glm` dans le registre natif vivant

- Le contrat (`permissions.md:208-223`) rapproche `cli_path` et `config_dir` de
  « la définition native autorisée ». Le registre vivant
  `/Users/moi/.config/bridget/agents.json` ne contient pas d'entrée `glm`.
  La cible glm n'existe que dans des registres privés de recette passés par
  variable d'environnement 148.
- Risque : la recette réelle (registre privé) et l'admission (définition native)
  peuvent porter des couples launcher/profil différents. Le contrat exige le
  même couple.
- Fix concret : T003 (`tasks.md:16`) consigne le registre source réel de la
  recette : chemin absolu, entrée exacte `glm` (`command=gclaude`,
  `claude_config_dir=/Users/moi/.claude-glm`, empreinte du launcher et du CLI).
  T038 fait de même pour le standalone. Les documents affirment que le registre
  vivant de production ne reçoit pas d'entrée `glm` sans décision humaine
  (aucune config de production modifiée).

### G-P-06 — mineur — Marqueurs de spec périmés et phrase de plan ambiguë

- La fiche synthèse de `spec.md:19-21` marque `tasks.md: ✗` et `plan.md: ✗`
  alors que les deux fichiers existent. `plan.md:39` écrit que le plan attend
  une « correction documentaire » de FR018, alors que le texte actuel de FR018
  (`spec.md:190-193`) porte déjà la forme corrigée demandée.
- Risque : une double lecture du statut de la spec à la gate.
- Fix concret : T001 (`tasks.md:14`) réconcilie la fiche et reformule
  `plan.md:39` au passé ou au conditionnel exact. Aucun changement de fond.

## 4. Couverture des critères G-P

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/plan-permissions-r1.md#extrait-5).

## 5. Couverture FR / SC demandée

| Exigence | Couverture | Remarque |
|---|---|---|
| FR007 (bornes parent, aucun gain) | Couverte | S149-01, S149-02, S149-06, S149-07, S149-15, S149-24, S149-25. |
| FR008 (lecture/écriture GLM+Codex, sans grant, héritage, refus nommés) | Couverte avec trou | Le chemin positif réel est S149-14 et T037. Trou : refus en cours de mission (G-P-01). |
| FR009 (identique par le parcours natif Bridget) | Couverte | S149-12, S149-14, T038. |
| FR011 (opt-outs et révocation, aucune projection sans preuve) | Couverte | S149-03, S149-20, S149-21, S149-25, S149-26. |
| FR012 (choix humains MCP conservés, grants hors MCP) | Couverte | S149-21, T005-T009, T013. |
| FR018 (permissions effectives préservées, retrait de garde limité à l'héritage) | Couverte | S149-02, S149-06, S149-15, S149-20, S149-24 ; texte FR018 actuel conforme. |
| FR019 (figeage avant effet, retry sans élargissement, mode UI sans effet) | Couverte avec trou | S149-09, S149-26. Trou : divergence de digest (G-P-02). |
| SC003 (écriture permise réussit, hors politique refusée, découverte = lecture) | Couverte avec trous | G-P-01 et G-P-02 portent tous deux sur SC003. |
| SC004 (même délégation par le parcours natif Bridget) | Couverte | S149-12, T038, avec séparation premier parent externe / descendant managed. |
| SC005 (opt-outs masquent la projection, preuve révoquée ferme, moteur survit) | Couverte | S149-20, S149-21, S149-25, S149-26, S149-18. |

## 6. Axes constitution

### Complexité (article XVIII)

Conforme. Aucun finding. Index `root_owner_agent_id`/`parent_task_id` et
`child_agent_id` unique (`data-model.md:33-39`). Pagination `O(log N + P)`,
P≤100. Réconciliation `O(n)` une fois. Séquence incrémentée dans la transaction
d'état. Registre `O(log n + a)` documenté (`registry.rs:196`). Aucune double
boucle ni N+1 proposé.

### Minimalisme et Frugalité (article XIX)

Potentiel minimalisme : ~70 lignes de redondance documentaire, aucun code
encore écrit. Les invariants de `permissions.md`, `research-native.md`,
`plan.md` et l'ADR149 répètent les mêmes règles trois à quatre fois. Ce coût est
accepté comme checkpoint de gate, mais il produit déjà G-P-03 (un code nommé
deux fois différemment). Fix structurel : une table unique des codes de refus et
des sources dans `permissions.md`, citée par les autres documents sans
reformulation. Les unions du contrat ne sont pas spéculatives : chaque valeur
est vérifiée dans le wire réel (schéma généré Codex, défauts SDK). Rien à
retirer du contrat sur ce plan. Le callback `permission_callback` ne transporte
aucune fonction : c'est un fait, pas un wrapper. Bien.

### Vertus LLM et Responsabilité future (article XX)

Charge future réduite si G-P-03 et G-P-04 sont appliqués : les refus nommés, le
refus d'inventer une fusion SDK et le refus d'assertions de confinement
suppriment des classes entières de débogage futur. Le contrat reste explicable
en intention, invariants et limites : la preuve porte sur des inputs, le
fournisseur applique ses règles, un manque de preuve est un refus nommé. Les
limites sont déclarées (recette PTY exigée avant validation, contrats non figés
jusqu'à la gate). Le point de vigilance est la multiplication des documents qui
répètent les refus : sans source unique, la prochaine évolution risque une
divergence silencieuse.

## 7. Demandes réalistes pour les tests réels

La revue valide ces exigences et demande qu'elles restent non négociables :

1. Écrivain réel GLM `glm-5.3-flash`, modèle exact. Toute autre valeur, y
   compris un repli `glm-5.3`, échoue le test (stratégie correction 8, S149-14,
   §6 des commandes). Aucun repli de modèle.
2. Écrivain réel Codex, un enfant, une écriture, un refus hors politique
   (T037).
3. Premier parent GLM réel dans un terminal, voie PTY par sa source propriétaire,
   avec dans un environnement natif isolé (S149-14 variante (a), T038). Ce cas ne peut être remplacé ni
   par un descendant managed ni par une fixture (correction 3).
4. Registre privé de recette cohérent avec le couple attesté : `gclaude` +
   `/Users/moi/.claude-glm`, consigné par chemin absolu (G-P-05).
5. Empreintes gelées et recontrôlées : launcher ET CLI résolu (G-P-04), settings
   et commandes sources inchangés en fin de recette.

Ces demandes sont réalisables avec les harnais existants (`idempotent.rs`,
recette 148 étendue) et sans aucun redémarrage ni config de production.

## 8. Conditions de levée du verdict

REQUEST_CHANGES levé quand :

1. G-P-01 : un scénario fixture prouve le refus corrélé `can_use_tool` et
   l'échec explicite, avec l'oracle exact ci-dessus.
2. G-P-02 : deux oracles prouvent `settings_revision_changed` (unitaire +
   fixture, avant lancement et à la reprise).
3. G-P-03 : un seul nom par cas de refus managed opaque, présent dans le
   contrat, le tableau de mappage et un oracle.
4. G-P-04 et G-P-05 : digests du CLI résolu et registre glm de recette consignés
   dans le contrat ou les preuves T003/T038.
5. G-P-06 : réconciliation documentaire T001.

La stratégie doit aligner ses oracles sur le contrat final avant la gate, comme
le prévoit déjà `plan.md:13`. Aucun code de permission ou d'héritage ne démarre
avant la fermeture de ces points et le GO principal.

## 9. Limites de cette revue

- Revue documentaire et de lecture de code. Aucun test, aucun build, aucune
  recette réelle exécutés.
- Les constats de §2 décrivent cette machine à cette date. Ils ne sont pas une
  preuve de comportement fournisseur et ne couvrent pas tous les contextes.
- Le volet Lineage (G-L, `contracts/lineage.md`) n'est pas jugé ici.
- Les numéros de ligne valent pour les versions des fichiers à la date de cette
  revue (base 148 `6807c22b`, worktrees 149 inchangés).
