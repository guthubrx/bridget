# Archives techniques de l’adaptateur T3

Document d’origine : [specs/160-reprise-quota/spec.md](../../../../../../specs/160-reprise-quota/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Résumé:
- Contexte: quand le quota d'un fournisseur est épuisé au milieu d'une mission native, la mission échoue pour de bon et le parent reçoit un avis d'échec. T3 reprend un fil limité à l'heure de réinitialisation. Bridget ne le fait pas. Une première conception de cette reprise (session 157) a été retirée après deux rondes de contre-revue : six défauts, tous dans la reprise.
- Objectif: une mission lancée avec `resume_on_limit` attend l'heure de réinitialisation du quota, sans avis d'échec, puis reprend avec son identifiant d'origine. L'attente est bornée (6 h au plus, 2 rondes), sans boucle, et le fournisseur ne change jamais.
- Dépendances: SPEC-150, SPEC-152 (livrée, primitives prouvées en T001), SPEC-157 (livrée : mémoire de limites par fenêtre, épuisement attesté)

<a id="extrait-2"></a>

## Extrait 2

Date : 2026-10-11. Accord utilisateur : être au moins aussi bon que T3, qui
reprend une mission après un quota épuisé. Cette session est la « session
future » annoncée dans `specs/157-catalogue-quotas/spec.md`. Critère visé :
Résilience 4 vers 5 (voir `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

<a id="extrait-3"></a>

## Extrait 3

**Ce que fait T3.** `limitRecoveryCommand` relance le fil limité quand l'heure de
retour est atteinte. Il exige une erreur de classe `usage_limit`, une heure de
retour postérieure à la fin du run (sinon « un retour déjà passé ne peut pas
démarrer une boucle »), et une identité `fil:run:retour`. Tout est dérivé de
faits persistés : aucune minuterie n'est à restaurer
(`apps/server/src/orchestration-v2/UsageLimitRecoveryWorker.ts`). La reprise est
un réglage global, T3 ne change jamais de fournisseur.

<a id="extrait-4"></a>

## Extrait 4

Un appel sans `resume_on_limit` produit les mêmes octets canoniques et le même
comportement qu'avant. La vue Lineage T3 ne change pas : la tâche reste
`working`. Le catalogue annonce la fonction dans `features`.

<a id="extrait-5"></a>

## Extrait 5

- Les sessions 150, 152 et 157 sont livrées. La porte T001 prouve chaque
  primitive utilisée sur le code livré. Sinon, la session s'arrête.
- Limite fournisseur : une limite Codex globale sans fenêtre attestée ne qualifie pas ; T002 n’a vu aucun refus de quota réel. Déclenchement réel Codex non prouvé, sans gate fournisseur supplémentaire ; limite reprise dans ADR T029.
- Un vrai refus de quota est visible par l'un des deux chemins A ou B, et le
  fait de quota est enregistré par la 157. Cela n'a pas été vu en réel. T002
  cherche un exemple dans le journal. Sans exemple, la validation le dit.
- L'enfant d'une mission en attente reste connecté (agent persistant). S'il
  disparaît, FR-014 s'applique.
- Le wrapper connaît l'exécution et la génération du message refusé dans la
  plupart des chemins (liaison `bindings`). Pour l'injection synchrone refusée,
  la liaison peut manquer : T036 le vérifie. Sans corrélation, une tâche relancée
  échoue explicitement (FR-006b).
- Quand l'enfant est connecté et libre, la remise de la mission reprise passe par
  la même voie que la reprise ordinaire (queue de `schedule_execution_recovery`,
  extraite en T019).
- Le gel d'un enfant ne gèle pas la tâche de son parent : si le parent expire
  avant l'enfant, l'annulation en cascade de 152 s'applique (limite notée).
- Les tests suivent la règle du projet : `umask 077`, environnement privé,
  racine temporaire courte (voir `quickstart.md`).
- Base livrée : `6907df9ba15b58df05c2041631005beb57a1bed6` (159B). Livraison Git/disque/activation Bridget par ROOT après les gates et absence de mission non terminale ; pas de redémarrage T3. Les anciennes bases et lignes sont des références historiques à revalider par nom.
