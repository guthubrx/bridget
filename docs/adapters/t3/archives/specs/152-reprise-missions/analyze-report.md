# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/analyze-report.md](../../../../../../specs/152-reprise-missions/analyze-report.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Restant | Traitement |
|---|---|---|
| R1 | H4 : l'instance du bail de reprise garde-t-elle le même `instance_id` que le bootstrap natif ? La flotte accepte-t-elle un lien dont le parent est une instance disparue (mission imbriquée) ? Non prouvé par lecture | test rouge T004 (a à d) ; `native_child_env` et `ownership_from_desired` s'adaptent (T024) |
| R2 | Les codes de refus de registre ne sont pas tous inventoriés | T029 avant l'écriture des reçus |
| R3 | Une saga de lancement en cours au début d'une pause de plus de 300 s peut expirer | limite documentée (FR-012) ; test T006 |
| R4 | Historique clos : la spec et le code 151 sont désormais livrés et relus | H1 réattestée ; RequestWait pris en compte dans T018, budget de l'appelant inchangé |
| R5 | Mission imbriquée : le propriétaire (un enfant) peut être relancé avec une autre instance. `recover_owner` existe (`N:172-190`) mais le cas n'est pas testé avec reprise | ajouter un cas imbriqué à T025 si T004 le révèle |
| R6 | La vue Lineage T3 montre `working` pendant la pause | question utilisateur 3, défaut non |
| R7 | Plusieurs sessions modifient `native_delegation.rs` (151, 153) | exécution séquentielle |
