# Session157 - Catalogue fiable, quotas et remplacement de fournisseur

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 157-catalogue-quotas
Titre: Catalogue fiable, quotas visibles et remplacement explicite
Statut: Spécifiée
Priorité: P1
Tâches: à exécuter (26 tâches générées, 0 faite)
Tests: à écrire en rouge d'abord (voir tasks.md)

Résumé:
- Contexte: `bridget_capabilities` annonce des capacités que `bridget_delegate` refuse ensuite pour le même appelant (constat du 2026-10-11 : `discovery: true` pour `codex-pro`, refus `permission_source_unavailable` depuis un fil T3 Claude). Le catalogue ne dit ni si le binaire existe, ni quel quota reste. Si un fournisseur est indisponible, le parent n'a aucun moyen sûr de demander une alternative.
- Objectif: un catalogue qui dit la vérité pour CET appelant, des quotas visibles et respectés à l'admission, et un remplacement de fournisseur demandé par le parent, jamais silencieux. La reprise sur limite d'usage est reportée à la session 160 (voir plus bas).
- Dépendances: SPEC-148, SPEC-149, SPEC-150 (livrée). Aucune dépendance de 152 : la reprise sur quota est reportée en 160.

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-150-socle-delegation` (exécution séquentielle dans le worktree unique, sans commit automatique, décision 150b)
**Created**: 2026-10-11
**Status**: Spécifiée
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-150

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles
en communication et délégation entre agents »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).
Critères visés (révisés après la contre-revue, ronde 2) : lancement 4 vers 5 ;
résilience : 157 apporte l'évitement des échecs prévisibles à l'admission
(binaire absent, quota épuisé connu, remplacement explicite). La note de
résilience reste donc celle de 152 (4). Le passage à 5 est reporté à la session 160.

Les lignes de code citées viennent de la base `1f2c661d` (avant la session 150).
La session 150 déplace certaines lignes. Les tâches citent les fonctions.

## Besoin et périmètre

Un agent appelle `bridget_capabilities`, choisit un fournisseur, puis appelle
`bridget_delegate`. Quatre défauts rendent ce parcours peu fiable.

**Défaut 1 - Le catalogue promet ce que l'admission refuse.** Le catalogue
calcule `discovery` avec deux tests : le registre accepte la posture, et le
parent a un fait attesté ou une permission
(`crates/bridget-daemon/src/daemon/native_delegation.rs:381` et `:388`). Il ne
calcule pas la politique de l'enfant. L'admission, elle, calcule cette
politique avec `child_policy` dès qu'un fait attesté existe (`:431-440`).
Pour un parent Claude de T3 et une cible Codex, `child_policy` exige un
`launch_context` (`crates/bridget-daemon/src/native_permissions.rs:253-254`).
Ce champ est optionnel dans une politique Claude
(`crates/bridget-transport/src/native_permissions.rs:75`). Un fait Claude venu
de T3 peut donc ne pas en avoir. Le catalogue dit alors `discovery: true`, et
l'admission rend `permission_source_unavailable`. Le même catalogue contient
déjà `inherit: false` et `inherit_refusal`, mais `discovery` les ignore.
Ce scénario explique le constat, mais il reste une hypothèse. Un fait Claude
qui porte un `launch_context` peut aussi donner ce code si sa source est
illisible ou a changé (`recheck_context`, `native_permissions.rs:229`, et
`claude_inputs_without_rules`, `:255`). Deux tests rouges séparent les deux
causes (T007). L'incident réel n'est confirmé qu'avec un fait réel expurgé
(T002) ; sans lui, la spec dit « scénario de code ».

**Défaut 2 - Aucune disponibilité réelle.** Le lancement vérifie le binaire, la
garde de facturation, l'environnement et les capacités dans `submit_spawn`
(`crates/bridget-daemon/src/lifecycle.rs:209-226`), donc après l'admission.
Un binaire absent crée une tâche, puis la fait échouer. Le catalogue ne le
sait pas. L'authentification n'est vérifiée nulle part. Le catalogue ne la
vérifiera pas non plus, car cela exigerait un appel au fournisseur (voir
question 3).

**Défaut 3 - Les quotas observés ne servent à rien.** Les faits `rate_limits`
arrivent par la présence (flux Claude `stream-json`, Codex app-server, hook
`claude-statusline`). Ils vivent dans `Presence.rate_limits`
(`crates/bridget-daemon/src/daemon.rs:566`, mise à jour `:6641`). Ils sont
informatifs seulement. Ils disparaissent avec la présence : quand un enfant
échoue sur un quota, sa présence part, et la preuve avec elle. Rien n'empêche
de déléguer vers un fournisseur dont le quota est épuisé. Une mission qui
touche la limite échoue pour de bon (`native_delegation.rs:1197`, état `failed` lu dans le tick).
Ce dernier point est traité en session 160, pas ici.

**Défaut 4 - Pas de remplacement.** Si la cible est indisponible, le parent
doit deviner une alternative et rappeler. Le contrat 148 interdit la
substitution silencieuse (`model_unavailable` sans remplacement). T3 n'a pas
de bascule automatique de fournisseur non plus (comparaison T3, mécanisme A10).

Cette session corrige les défauts 1, 2 et 4, et la partie « visibilité et refus
à l'admission » du défaut 3. Elle ne change pas le contrat
d'appel existant : un appel sans nouveau paramètre garde le même comportement
qu'en 150, sauf deux refus plus précoces et plus précis, listés dans
`contracts/delegation157.md`.

**Mécanismes T3 repris.** T3 calcule `canRunChildTask` et `constraints` avec la
même fonction que l'admission (`apps/server/src/mcp/OrchestratorMcpService.ts:264-280`,
`:1146-1153`, `:1798-1800`). Bridget reprend ce principe : un seul verdict.
T3 relance aussi un fil limité à l'heure de réinitialisation
(`apps/server/src/orchestration-v2/UsageLimitRecoveryWorker.ts`,
`limitRecoveryCommand`). Cette reprise est reportée en 160.

**Où Bridget fait mieux que T3.** Le remplacement de fournisseur existe, mais il
est demandé par le parent, ordonné, borné à 3 candidats et toujours signalé.

**Hors périmètre** :
- authentification vérifiée par appel au fournisseur (question 3) ;
- bascule vers le remplacement en cours de mission (question 2) ;
- reprise sur limite d'usage (`resume_on_limit`, attente, relance) : session
  future 160, voir plus bas ;
- persistance des quotas après un redémarrage du daemon (dette notée) ;
- budgets de tokens ou de coût (session158) ;
- réveils programmés (session158) ;
- reprise d'une mission après panne ou après pause, suspension par la pause,
  borne de reprises après panne (session152) ;
- admission d'une mission en attente d'un quota futur sans remplacement.

## Scénarios utilisateur

### US1 - Le catalogue ne ment plus (P1)

Un agent T3 Claude appelle `bridget_capabilities`. Pour `codex-pro`, le
catalogue dit `can_run: false`, `discovery: false`, avec la contrainte
`permission_source_unavailable` en clair. L'appel `bridget_delegate` vers
`codex-pro` avec les mêmes paramètres rend le même code. Quand le catalogue dit
`can_run: true` pour une posture, l'admission de cette posture n'est refusée ni
pour les droits, ni pour le binaire, ni pour le quota. Un binaire absent est
visible dans le catalogue et refusé à l'admission, sans créer de tâche.

### US2 - Les quotas sont visibles et respectés (P1)

Le catalogue montre, pour chaque fournisseur, les fenêtres de quota connues :
pourcentage utilisé, heure de réinitialisation, épuisé ou non. Il montre
`quota: null` quand rien n'est connu. Un quota épuisé et non réinitialisé donne
`can_run: false` et la contrainte `provider_quota_exhausted`. L'admission rend
alors `provider_quota_exhausted` avec l'heure de réinitialisation. Un fait dont
l'heure est passée est ignoré. La connaissance survit à la fin de la présence
qui l'a observée.

### US3 - Remplacement explicite par le parent (P2)

Le parent fournit `fallback`, une liste ordonnée d'au plus 3 cibles. Si la cible
demandée est indisponible ou à quota épuisé, l'admission essaie les cibles dans
l'ordre. La première qui passe tous les contrôles est utilisée. La réponse
contient `substitution` : cible demandée, cible choisie, motif, cibles écartées.
Sans `fallback`, rien ne change. Un modèle absent du catalogue de la cible
(`model_unavailable`) n'est jamais remplacé. Si aucune cible ne passe, le refus
est `fallback_exhausted` avec la liste des essais.

### US4 - Aucun changement pour l'existant (P1)

Un appel sans `fallback` produit les mêmes octets
canoniques et le même comportement qu'en 150. Les tâches actives gardent leurs
règles. La vue Lineage T3 garde ses états et ses clés. Les champs ajoutés au
catalogue sont additifs.

## Exigences fonctionnelles

**Verdict unique et disponibilité**

- **FR-001** : Le catalogue et l'admission utilisent la même fonction de verdict
  par fournisseur : préflight de lancement, politique de l'enfant (droits
  hérités ou discovery), quota. Aucun test du catalogue n'existe hors de cette
  fonction.
- **FR-002** : Pour chaque fournisseur et chaque posture (`inherit`,
  `discovery`, `development`), le catalogue rend `true` seulement si
  l'admission, pour le même appelant, la même posture, le même contexte et un
  état inchangé, ne rendrait aucun refus de la famille verdict (liste fermée
  dans le contrat). Le contexte est celui que le catalogue déclare dans
  `verdict_context` : répertoire = racine du fait attesté (`cwd_root`), fait non
  limité à une demande. Un refus qui vient d'un autre répertoire, d'un fait
  limité à une demande (`request_id`) ou d'un changement d'état entre les deux
  appels (binaire retiré, quota atteint) n'est pas une contradiction. Le
  catalogue le dit, et l'admission reste l'autorité.
- **FR-003** : Chaque entrée du catalogue contient `can_run` (au moins une
  posture possible, binaire présent, quota non épuisé) et `constraints`, liste de
  `{code, message}`. Le message est une phrase courte en français.
- **FR-004** : Le préflight de lancement est extrait de `submit_spawn` ET de
  `prepare_spawn_parts` (type connu, capacités, observation fournisseur, garde
  de facturation, protocole géré, environnement, binaire) et partagé. Il
  s'exécute sur la définition épinglée de la tâche. Une erreur de
  `build_environment` est propagée, jamais ignorée. L'admission le joue avant de
  créer la tâche. Son refus reprend le code existant (`command_missing`,
  `billing_guard`, `env_unfit`, `unsupported_capability`, `negotiation_failed`).
  Le répertoire de travail n'en fait pas partie (contrôle du parent).
- **FR-005** : Le catalogue contient `checks` : ce qu'il vérifie (`binary: true`,
  `quota: "observed"`) et ce qu'il ne vérifie pas (`auth: false`).
- **FR-006** : Le catalogue reste en lecture seule. Il n'appelle aucun
  fournisseur, n'écrit rien et reste disponible pendant la pause.

**Quotas**

- **FR-007** : Le daemon garde la dernière limite connue par type d'agent et par
  fenêtre, avec l'instant d'observation. Elle est mise à jour par les trois
  sources existantes et survit à la fin de la présence. Elle est volatile : un
  redémarrage du daemon l'efface.
- **FR-008** : Une fenêtre est épuisée seulement si son épuisement est attesté
  pour elle-même et si son heure de retour est dans le futur. Attesté veut dire :
  statut `rejected` d'un fait Claude `stream-json` (le fait porte sa fenêtre), ou
  pourcentage utilisé de 100 ou plus (toute source). Le statut d'un fait Codex
  est global : `rateLimitReachedType` est recopié sur les deux fenêtres
  (`codex_app_server.rs:2639-2665`). Il ne prouve l'épuisement d'aucune fenêtre
  en particulier. Un statut global « atteint » sans fenêtre à 100 % donne
  `quota.limit_reached: true`, `exhausted: false` et une contrainte
  d'information qui ne bloque pas. Le daemon n'invente jamais la fenêtre
  responsable. Sans heure de retour connue, une fenêtre n'est jamais épuisée.
- **FR-009** : Le catalogue rend `quota` : liste de fenêtres (`window`,
  `used_percent`, `resets_at`, `exhausted`), `exhausted`, `resets_at`
  (le plus tardif des fenêtres épuisées), `observed_at`. Sans fait connu :
  `null`. Aucune valeur n'est inventée.
- **FR-010** : L'admission refuse `provider_quota_exhausted` avec
  `detail.resets_at` quand le quota de la cible effective est épuisé. Aucune
  tâche n'est créée. La clé de rejeu reste libre.
- **FR-011** : Un fait dont l'heure de retour est passée est ignoré à la lecture
  et retiré à l'écriture.

**Remplacement**

- **FR-012** : `bridget_delegate` accepte `fallback` : au plus 3 objets
  `{agent_type, model, effort?}`. Le champ fait partie de l'enveloppe
  idempotente. Une entrée vide, en double ou égale à la cible demandée rend
  `invalid_request`.
- **FR-013** : Le remplacement ne se déclenche que pour les motifs de la liste
  fermée « indisponibilité » : `command_missing`, `billing_guard`, `env_unfit`,
  `unsupported_capability`, `permission_source_unavailable`,
  `permission_mapping_unavailable`, `provider_confinement_unavailable`,
  `provider_quota_exhausted`. `model_unavailable`, `effort_unavailable` et tout
  refus de droits du parent ne déclenchent jamais de remplacement.
- **FR-014** : Chaque candidat passe le contrôle complet dans cet ordre : type
  connu, modèle, effort, puis préflight, politique, quota. Modèle et effort
  viennent AVANT tout motif remplaçable : un modèle absent ou un effort invalide
  est rendu tel quel, même si le binaire manque ou si le quota est épuisé. La
  politique de l'enfant est calculée depuis le parent pour chaque candidat : un
  remplacement ne gagne aucun droit. Le premier candidat valide, dans l'ordre,
  est choisi.
- **FR-015** : La substitution est signalée dans la réponse de l'admission, dans
  `bridget_task_status` et dans la tâche durable : `substitution` avec
  `requested`, `selected`, `reason` et `tried`. La tâche porte la cible
  effective dans `agent_type`, `model` et `effort`.
- **FR-016** : Avec `fallback` et aucun candidat valide, le refus est
  `fallback_exhausted` avec `detail.tried` (code par candidat). Sans `fallback`,
  les codes de refus restent ceux d'avant.
- **FR-017** : Le rejeu d'une clé rend la tâche existante avec sa substitution
  d'origine. Le verdict n'est pas recalculé.
- **FR-018** : Il n'y a aucun remplacement après l'admission (pas de bascule en
  cours de mission).

**Compatibilité et documentation**

- **FR-019** : Un appel sans `fallback` produit les mêmes
  octets canoniques qu'en 150.
- **FR-020** : Les nouveaux champs et refus sont documentés dans
  `docs/delegation-native.md` et dans la skill Bridget. Le catalogue annonce
  `features` : `fallback` et `fallback_max`.

## Critères de succès

- **SC-001** : Sur une matrice parent × cible × posture (au moins 3 types de
  parent, 4 cibles, 3 postures), 0 contradiction entre le catalogue et
  l'admission.
- **SC-002** : Le cas du 2026-10-11 est reproduit en test : parent Claude sans
  `launch_context`, cible Codex. Avant : `discovery: true` et refus. Après :
  `can_run: false`, même code dans le catalogue et dans le refus.
- **SC-003** : Binaire absent : 0 tâche créée à l'admission, 100 % de refus
  `command_missing`. Avant : 1 tâche créée puis échouée.
- **SC-004** : Quota épuisé connu : 100 % de refus `provider_quota_exhausted` avec
  l'heure de retour. Fait périmé : 0 refus. Aucun fait : `quota: null`, 0 refus.
- **SC-005** : Avec `fallback`, une cible indisponible donne la première
  alternative valide dans 100 % des cas, et 100 % des tâches substituées portent
  `substitution`. 0 substitution sans `fallback`. 0 substitution pour
  `model_unavailable`.
- **SC-006** : Tests de non-régression 148, 149 et 150 verts. Octets canoniques
  identiques sans nouveau champ.
- **SC-007** : Le temps de réponse du catalogue (10 fournisseurs de test) reste
  au plus 2 fois la mesure de référence prise avant le changement (T001, T012).

## Entités et hypothèses

**Entités**

- **Verdict de cible** : résultat du contrôle d'un fournisseur pour un appelant
  et une posture : accepté, ou refus avec un code.
- **Limite connue** : dernière fenêtre de quota observée pour un type d'agent.
- **Substitution** : trace durable du remplacement d'une cible.

**Hypothèses**

- Le type d'agent d'une présence (`Presence.agent_type`) est égal au nom du
  fournisseur dans le registre. Un quota est donc indexé par ce nom. Deux
  fournisseurs qui partagent un compte ne partagent pas leur quota (limite
  notée). Un test vérifie le nom enregistré par les trois sources.
- Les tâches de cette session ne dépendent pas de 152. L'ordre d'arrivée d'un
  fait de quota et d'un échec de mission n'est pas garanti : ce point concerne
  la 160.
- Un modèle absent du catalogue est une erreur d'appelant, pas une
  indisponibilité. Le contrat 148 le dit (`model_unavailable` sans substitution).
- Les tests suivent la règle du projet : `umask 077`, environnement privé,
  racine temporaire courte (voir `quickstart.md`).
- La livraison suit les sessions 147 à 150 : fichiers sur disque, sans relance
  automatique des services.

## Questions pour l'utilisateur

1. **Reprise sur limite d'usage reportée en 160 : d'accord ?** Défaut retenu :
   oui. La 157 donne la visibilité des quotas, le refus à l'admission et le
   remplacement. La résilience reste à 4 (note de 152). La 160 la porte à 5
   quand 152 est livrée et ses primitives prouvées.
2. **Remplacement seulement à l'admission ?** Défaut retenu : oui. Une bascule
   en cours de mission perd la session du fournisseur et exige une carte de
   reprise ; elle est traitée avec la 160.
3. **Authentification : accepter qu'elle ne soit pas vérifiée ?** Défaut retenu :
   oui. Le catalogue le dit (`checks.auth: false`). Une vraie vérification
   demande un appel au fournisseur, ce que cette session s'interdit. Un échec
   d'authentification au lancement reste un échec de tâche nommé.

## Session future 160 - reprise sur quota

La reprise sur limite d'usage est sortie de la 157 (Article XIX, convergence)
après deux rondes de contre-revue Codex (voir `adversarial-review-codex.md`).
Elle reste voulue : critère Résilience 4 vers 5. Elle est préparée dans
`specs/160-reprise-quota/` (accord utilisateur du 2026-10-11 : au moins aussi
bon que T3).

**Condition de démarrage** : 152 livrée, et chaque primitive de 152 que la 160
réutilisera prouvée par un test sur le code livré (pas seulement planifiée dans
les artefacts de 152). La 157 doit être livrée (elle fournit la mémoire de
limites `provider_limits` et l'épuisement attesté par fenêtre).

**Exigences d'entrée** (défauts de la conception abandonnée, tous à résoudre
avant d'écrire du code) :

1. **Compteur de rondes durable.** Le compteur `round` était porté par
   `limit_resume`, effacé après l'incrément. Le compteur doit survivre à
   l'effacement de l'attente, sinon la borne de 2 rondes n'existe pas.
2. **Reconstruction d'une exécution `failed`.** Les requêtes de sélection et de
   mise à jour de la reprise ordinaire excluent l'état `failed`
   (`execution_store.rs:1325` et `:1393`). Étendre `may_resume` ne suffit pas :
   la 160 doit définir et prouver comment relancer une mission dont
   l'exécution est `failed`.
3. **Identité de relance vérifiée et solde de l'attente.** L'identité
   `limit-resume:<tâche>:<ronde>:<reset>` doit être vérifiée avant l'appel, et
   un rejeu après crash doit solder l'attente (pas de seconde relance, pas
   d'attente orpheline).
4. **Borne de pannes de 152 non consommée.** Une continuation due au quota ne
   doit pas consommer la borne de 2 pannes de 152 (`resume_attempts`, 152
   `data-model.md` I4).
5. **Définition de `suspended_total_secs` et pas de double comptage.** Définir
   l'usage de `suspended_total_secs` pour une attente de quota, sans double
   comptage quand une pause et une attente de quota se chevauchent.
6. **Chemin `rejected()`.** Après `rejected()`, le gestionnaire général supprime
   le suivi de la demande et notifie le parent (`daemon.rs:18095`). La décision
   d'attendre doit empêcher cette notification et cette suppression, pas
   seulement la mise à `failed` (`native_delegation.rs:810`). La 160 doit aussi
   traiter le refus de remise émis par le wrapper (`wrapper.rs:6218-6262`).

**Autres exigences connues** : ordre d'arrivée non garanti entre le fait de
quota et l'échec (prévoir un délai de décision) ; échec réel de quota jamais vu
en réel (chercher un exemple dans le journal avant de concevoir) ; l'attente ne
doit pas réutiliser `suspended_at` ni `resume_attempts` de 152 (pause et panne) ;
un redémarrage pendant l'attente ne doit pas reconstruire l'enfant avant l'heure
de retour ; paramètre d'appel `resume_on_limit` opt-in (défaut `false`), 6 h
d'attente au plus.
