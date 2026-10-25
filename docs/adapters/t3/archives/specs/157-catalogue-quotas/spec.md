# Archives techniques de l’adaptateur T3

Document d’origine : [specs/157-catalogue-quotas/spec.md](../../../../../../specs/157-catalogue-quotas/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Résumé:
- Contexte: `bridget_capabilities` annonce des capacités que `bridget_delegate` refuse ensuite pour le même appelant (scénario de code associé au signalement du2026-10-11 : `discovery: true` pour `codex-pro`, puis refus `permission_source_unavailable` depuis un fil T3 Claude ; aucun fait runtime corrélé retrouvé). Le catalogue ne dit ni si le binaire existe, ni quel quota reste. Si un fournisseur est indisponible, le parent n'a aucun moyen sûr de demander une alternative.
- Objectif: un catalogue qui dit la vérité pour CET appelant, des quotas visibles et respectés à l'admission, et un remplacement de fournisseur demandé par le parent, jamais silencieux. La reprise sur limite d'usage est reportée à la session 160 (voir plus bas).
- Dépendances: SPEC-148, SPEC-149, SPEC-150 (livrée). Aucune dépendance de 152 : la reprise sur quota est reportée en 160.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

**Mécanismes T3 repris.** T3 calcule `canRunChildTask` et `constraints` avec la
même fonction que l'admission (`apps/server/src/mcp/OrchestratorMcpService.ts:264-280`,
`:1146-1153`, `:1798-1800`). Bridget reprend ce principe : un seul verdict.
T3 relance aussi un fil limité à l'heure de réinitialisation
(`apps/server/src/orchestration-v2/UsageLimitRecoveryWorker.ts`,
`limitRecoveryCommand`). Cette reprise est reportée en 160.

<a id="extrait-4"></a>

## Extrait 4

**Où Bridget fait mieux que T3.** Le remplacement de fournisseur existe, mais il
est demandé par le parent, ordonné, borné à 3 candidats et toujours signalé.

<a id="extrait-5"></a>

## Extrait 5

Un agent T3 Claude appelle `bridget_capabilities`. Pour `codex-pro`, le
catalogue dit `can_run: false`, `discovery: false`, avec la contrainte
`permission_source_unavailable` en clair. L'appel `bridget_delegate` vers
`codex-pro` avec les mêmes paramètres rend le même code. Quand le catalogue dit
`can_run: true` pour une posture, l'admission de cette posture n'est refusée ni
pour les droits, ni pour le binaire, ni pour le quota. Un binaire absent est
visible dans le catalogue et refusé à l'admission, sans créer de tâche.

<a id="extrait-6"></a>

## Extrait 6

Un appel sans `fallback` produit les mêmes octets
canoniques et le même comportement qu'en 150. Les tâches actives gardent leurs
règles. La vue Lineage T3 garde ses états et ses clés. Les champs ajoutés au
catalogue sont additifs.

<a id="extrait-7"></a>

## Extrait 7

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
- Livraison autorisée : commit, fusion, push vérifié, binaire signé installé,
  activation Bridget après zéro mission native active, nettoyage. T3 reste ouvert.

<a id="extrait-8"></a>

## Extrait 8

La reprise sur limite d'usage est sortie de la 157 (Article XIX, convergence)
après deux rondes de contre-revue Codex (voir `adversarial-review-codex.md`).
Elle reste voulue : critère Résilience 4 vers 5. Elle est préparée dans
`specs/160-reprise-quota/` (accord utilisateur du 2026-10-11 : au moins aussi
bon que T3).
