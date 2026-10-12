# Contre-revue adverse - Session157

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : cause du catalogue incohérent, remplacement et contrat 148,
  reprise sur quota, quotas, extraction du préflight.
- **Verdict reçu** : BLOCKED (7 objections). La revue confirme la cause comme
  scénario de code (pas encore prouvée sur l'incident réel). Elle confirme que
  `fallback` et la séparation `canonical` / `request` préservent en principe le
  contrat 148 et le rejeu, sans gain de droits si chaque candidat reste borné
  par les droits du parent.
- **Vérification** : chaque objection a été relue dans le code du checkout
  principal (`1f2c661d`) et dans les specs 150 et 152 du worktree.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | BLOQUANT. `attendu-152.md` contredit la spec 152 réelle : 152 laisse les tâches terminales inchangées, son compteur vise les pannes, son gel vise la pause, elle ne promet ni relance générale de `failed` avec clé fournie, ni motif de suspension ouvert | lu `specs/152-reprise-missions/spec.md` FR-013 à FR-017, « Hors périmètre » (reprise sur quota = 157), `data-model.md` I1 à I4, `contracts/reprise152.md` (`suspended` booléen) | oui | `attendu-152.md` supprimé. `contracts/reprise-quota157.md` : la 157 possède `limit_resume`, son compteur de rondes, son gel et sa relance. Réutilisé et cité : `latest_in_chain`, `may_resume`, `schedule_execution_recovery`, `native_child_env`, `NativeMissionDispatch`, règle de pause de `rejected()`, schéma de décalage I2. Ni `suspended_at` ni `resume_attempts`. Spec (hypothèses, FR-019 à FR-024), plan, data-model, R7, tâches T022 à T028 corrigés |
| 2 | HAUTE. L'ordre préflight, politique, quota, modèle, effort masque `model_unavailable` derrière un motif remplaçable | lu `registry.rs:335-341` (modèle vérifié dans `for_inherited_delegation`, après `child_policy` dans `native_delegation.rs:435-438`) | oui | ordre fixe : type, modèle, effort, puis préflight, politique, quota (FR-014, R10, D-H). Tests : modèle absent + binaire absent, effort invalide + quota épuisé, modèle absent + politique indisponible : zéro substitution (T017) |
| 3 | HAUTE. Extraction du préflight incomplète : `submit_spawn` n'a pas la garde de facturation et ignore l'erreur de `build_environment` | lu `lifecycle.rs:209-226` (`if let Ok(env)`), `:342-380` (`prepare_spawn_parts` : `forbidden_environment_variable`, protocole géré) | oui | extraction depuis les deux fonctions, définition épinglée, erreur propagée, `negotiation_failed` ajouté à la famille verdict (FR-004, R3, T005). Cas facturation interdite et environnement invalide avec binaire présent |
| 4 | HAUTE. Une fonction commune ne garantit pas l'égalité : le fait attesté dépend de `request_id`, le `cwd` change la politique, le binaire peut changer | lu `native_delegation.rs:377` (`None`) contre `:420` (`Some(request_id)`), filtre `scope == request_id` `:56`, `native_permissions.rs:230` (`parent.cwd != cwd`) | oui | garantie bornée : mêmes entrées, fait non limité à une demande, `cwd` du `verdict_context`, état inchangé (FR-002, R2, D-I, contrat). Le catalogue déclare son contexte. Tests hors garantie : fait limité à une demande, sous-répertoire, binaire retiré (T007) |
| 5 | HAUTE. `rateLimitReachedType` est global, recopié sur les deux fenêtres : FR-008 lisait chaque copie comme un épuisement | lu `codex_app_server.rs:2639-2665` (même `status` sur `primary` et `secondary`) | oui | épuisement attesté par fenêtre seulement : `rejected` Claude (par fenêtre) ou 100 % utilisé. Statut Codex global sans fenêtre à 100 % : `limit_reached`, jamais `exhausted`. La mémoire garde la source du fait (FR-008, R4, D-J). Test : statut global + fenêtre courte à 100 % + hebdomadaire à 61 % (T004) |
| 6 | HAUTE. La reprise manque `DeliveryRejected` puis `rejected()`, qui terminalise sans passer par `tick` | lu `wrapper.rs:6218-6262` (transition `failed` puis `DeliveryRejected`), `daemon.rs:18080`, `native_delegation.rs:810-825`, et la parade de 152 (T013, D4) | oui | `fail_or_wait` unique, appelée avant toute terminalisation par `rejected()`, la branche d'exécution de `tick` et les refus de remise de mission, derrière les règles de pause de 152. Délai de 5 s car l'ordre fait/échec n'est pas garanti. Tests des deux ordres et zéro avis d'échec (FR-020, FR-021, R7, T023, T024) |
| 7 | MOYENNE. La cause reste une hypothèse : `launch_context` présent avec source illisible donne aussi `permission_source_unavailable` | lu `native_permissions.rs:229` (`recheck_context`), `:255`, `:185` | oui | spec et R1 : « scénario de code », deux variantes. Le diagnostic cite la cause dans `constraints[].message`. Deux tests rouges (T007). Confirmation de l'incident seulement avec un fait réel expurgé (T002) |

**Rejetée** : aucune objection.

**Corrections faites au passage**
- Le contrat citait `usage_limit` comme refus d'admission : c'est une erreur de
  tâche. Reformulé.
- `negotiation_failed` ajouté aux codes de disponibilité.
- Nombre de tâches : 32 puis 33 en ronde 1, puis 26 après la ronde 2.

**Reste ouvert**
- Les primitives de 152 ne sont pas livrées et 152 est en révision (F1).
- La reconstruction de 152 accepte-t-elle une exécution `failed` ? (F6)
- Un vrai refus de quota et son chemin d'échec n'ont pas été vus en réel (F2).
- L'incident du 2026-10-11 n'est pas confirmé sur un fait réel (F3).
- Une ronde 2 de contre-revue sur les artefacts corrigés n'a pas eu lieu.

## Ronde 2 (plan révisé)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 2, 3, 4, 5 et 7 fermées ; objection 1 partielle ;
  objection 6 ouverte. Les primitives de 152 citées existent dans la 152 révisée,
  mais sont planifiées, pas livrées. La révision a créé 4 défauts nouveaux, tous
  dans la reprise sur quota.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | BLOQUANT. Le compteur `round` est perdu : il était porté par `limit_resume`, effacé après l'incrément | lu `data-model.md` (relance : « `round += 1`, `limit_resume` effacé ») | oui | défaut de conception. Reprise sortie de la 157 ; exigence d'entrée 1 de la 160 |
| 2 | BLOQUANT. Reconstruire une exécution `failed` est impossible : la sélection et la mise à jour SQL de la reprise l'excluent | cité `execution_store.rs:1325` et `:1393` ; cohérent avec la partie « Point non prouvé » du contrat retiré | oui | étendre `may_resume` ne suffit pas. Exigence d'entrée 2 |
| 3 | L'identité de relance n'est pas vérifiée ; un rejeu après crash ne solde pas l'attente | lu le contrat retiré (`reprise-quota157.md`, section Relance) | oui | exigence d'entrée 3 |
| 4 | Une continuation due au quota consomme la borne de 2 pannes de 152 | lu 152 `data-model.md` I4 (`resume_attempts` n'augmente que par reconstruction après panne) | oui | exigence d'entrée 4 |
| 5 | `suspended_total_secs` est indéfini pour l'attente de quota et peut être compté deux fois (pause et quota) | lu 152 `data-model.md` (somme des pauses subies) | oui | exigence d'entrée 5 |
| 6 | OUVERTE. Après `rejected()`, le gestionnaire général supprime le suivi et notifie le parent | cité `daemon.rs:18095` ; `rejected()` lu en `native_delegation.rs:810` | oui | ne pas terminaliser ne suffit pas à empêcher la notification. Exigence d'entrée 6 |

**Décision (Article XIX, convergence)** : la reprise sur limite d'usage sort de
la 157. Retirés : US4 d'origine, `resume_on_limit`, `limit_resume`,
`fail_or_wait`, `contracts/reprise-quota157.md`, tâches T022 à T028. Aucune
partie de `fail_or_wait` n'est nécessaire au remplacement, qui ne joue qu'à
l'admission. Conservés : catalogue fiable, mémoire de limites par fenêtre,
refus `provider_quota_exhausted`, `fallback`.

**Où sont les exigences** : `spec.md`, section « Session future 160 - reprise
sur quota », avec la condition de démarrage (152 livrée et ses primitives
prouvées sur le code livré).

**Effets** : tâches 33 vers 26 ; plus aucune dépendance de 152 ; critère
Résilience visé : reste à 4 (note de 152), le 5 est reporté à la 160.

**Reste ouvert**
- L'incident du 2026-10-11 n'est pas confirmé sur un fait réel (T002).
- Le nom `agent_type` des présences pour les trois sources de quota (T013).
- Le coût du catalogue avec le 3e calcul de politique (T001, T012).
- Pas de ronde 3 de contre-revue sur les artefacts réduits.

## Revue157-r1 préparatoire du 2026-10-12

Verdict reçu : APPROVE_WITH_CHANGES. M1 trois passes sans cible propagée,
M2 gardes workspace hors verdict et M3 détail perdu dans Err ont été vérifiés
sur MAIN c24f6e3a. L’affirmation du reviewer «155 fusionnée» est corrigée :
T000155 seul est livré, pas la feature155. Les propositions sont documentaires,
à appliquer au GO157 après155/156, sans nouveau type de refus ou scénario.
Choix retenus : candidat épinglé, Commit sans resélection, gardes d’options,
code:JSON via failure, information quota non bloquante, éviction9 déterministe,
unknown seul non probant et SC007. Aucun Analyze ou code157 frais validé.
Le reviewer n’a pas relu Analyze/reuse/quickstart/checklist ni le code155/156.
La proposition optionnelle discovery sans cwd n’est pas adoptée comme garantie
avant revalidation du contexte : cwd reste requis/autorisé à l’admission.
