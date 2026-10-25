# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Cadre de cette revue. Elle juge les deux deltas, pas le code. Le code natif
est en cours chez Sol ; il n'est pas review ici. Une seule lecture ciblée de
`native_permission_observer.rs` a servi à confirmer la réalisabilité, sans
jugement. Aucun test, aucun build, aucun lint, aucun modèle lancé. Aucun Git,
aucune config, aucune base touchés. Aucune tâche cochée. Ce rapport est le
seul fichier écrit. La revue n'a relancé ni la revue G-L, ni la revue code T3,
ni les tests T3 — trois travaux distincts en cours. Aucun secret imprimé ; le
launcher n'a pas été relu (empreinte et ligne `exec claude "$@"` déjà prouvées
en r1 et r2).

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

Les preuves d'interop T3↔Rust déjà acquises (interop R4, régression
d'identité T3, tests T3 en mission) attestent le transport entre processus
déjà établis. Elles ne constituent pas la preuve d'entrée du Rust autonome.
La voie standalone, premier parent externe réel avec dans un environnement natif isolé, reste prouvée
par la recette S149-14(a) et T038 seulement. S149-32 et S149-33, au niveau
T3/fixture, prouvent la sémantique des refus et des corrélations ; ils ne
remplacent jamais cette recette. Le contrat le dit déjà
(`permissions.md:318-321,446-450`) ; les deux oracles nouveaux respectent
cette limite.

<a id="extrait-5"></a>

## Extrait 5

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
