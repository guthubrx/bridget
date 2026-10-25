# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md](../../../../../../../specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Cadre de cette revue. Elle juge les documents livrés, pas le code. Le code
natif est en cours chez Sol ; le code T3 et sa revue sont en cours ailleurs
(`validation/t3-permissions-code-review-r1.md`, hors périmètre). Cette revue
n'a lancé aucun modèle de travail, aucun test, aucun build, aucun lint. Aucun
Git, aucune config, aucune base touchés. Aucune tâche cochée. Aucune relance
de la revue G-L, de la revue code T3 ou des tests T3 — trois travaux distincts.
Ce rapport est le seul fichier écrit. Aucun secret imprimé.

<a id="extrait-2"></a>

## Extrait 2

1. Phrase corrigée. `contracts/permissions.md:40-41` porte maintenant :
   « reste possible tant qu'aucun fait de permissions n'a jamais existé pour
   ce credential ». C'est le texte exact du fix r3 §4. La contradiction avec
   le registre est levée : un tombstone (end/fail/close) laisse un fait qui
   **a existé**, donc la branche v1 est exclue. La ligne 43 couvre le reste :
   fait invalide, périmé, contradictoire ou révoqué → refus, jamais retombée
   en version1. La section Compatibilité répète le principe
   (`permissions.md:467-468`) : « Aucun downgradev1 permissif n'est admis
   après une preuve149 invalide ».
2. Oracle G-P-07. `permissions.md:440-448`, section Oracles (ligne 412),
   après G-P-02 et avant « Compatibilité et gate » (ligne 463). Texte
   identique mot à mot au §4 r3 : trois branches (a) (b) (c), enveloppe148 v1
   pour un credential sans fait jamais existé, refus nommé
   `permission_attestation_unavailable` pour tombstone/révocation/rotation,
   admission 149 avec la seule v1 → même refus, sans grant, sans fallback
   discovery, sans spawn, identité et status/cancel continuent.
3. S149-32. `test-strategy.md:187` (§4.10, ligne 181). Trois branches mot à
   mot, niveau « Unitaire daemon + T3 serveur », limite explicite : les tests
   T3 prouvent la sémantique de réponse, jamais l'entrée du Rust autonome.
   Mapping FR007, FR008, FR019 / SC003.
4. T015. `tasks.md:31` nomme S149-32 avec la sémantique complète du refus.

<a id="extrait-3"></a>

## Extrait 3

- Revue documentaire bornée aux deux findings et à la cohérence de la carte.
  Le code natif, le code T3, la revue G-L et les tests T3 ne sont pas jugés
  ici.
- Les sondes CLI 2.1.296 et doc hooks de r3 ne sont pas refaites ; la r3 les
  a confirmées et aucune écriture ne les touche.
- Les numéros de ligne valent pour les fichiers lus le 2026-10-10 dans le
  worktree `149-sous-agents-lineage`.
