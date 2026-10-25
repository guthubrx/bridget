# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/analyze-report.md](../../../../../../specs/159-edition-file/analyze-report.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Sévérité | Constat | Traitement |
|---|---|---|---|
| A1 | élevée | La latence de libération et la fiabilité du signal de tour de chaque porteur ne se prouvent pas par lecture | T001, T002(1), T006, T010 mesurent ; le réglage reste fermé tant que SC-004, SC-005 et SC-012 ne sont pas tenus |
| A2 | élevée | Migration à sens unique : un binaire antérieur refuse la base | T011 (répétition de la restauration, porte de livraison), précontrôle relevé en T003, procédure dans l'ADR (T025) |
| A3 | élevée | Modification du comportement de `TurnState` du wrapper (refus, réattestation) : touche tous les porteurs gérés | `turn_attest` limite l'effet aux instances qui le déclarent ; `busy_since` inchangé ; tests de non-régression de T001 ; T002(1) |
| A4 | moyenne | Réservation de tour : la durée de grâce de 30 s après l'accusé est un réglage empirique | testée aux deux bords (T007) ; à mesurer en T010 ; à ajuster avant l'ouverture du réglage |
| A5 | moyenne | Parité des causes d'écart du pont pour les retenus non relue en entier | T002(3), T008 |
| A6 | moyenne | Les porteurs gérés regroupent-ils des messages ? Si oui, « un par tour » les affecte | T002(6) ; sinon libération par lot borné |
| A7 | moyenne | Un envoi sans intention ne crée aucune exécution liée : à confirmer | T002(4) |
| A8 | moyenne | Après libération, garanties de toute remise seulement (`Seen` avant injection ; file ordinaire pleine ; doublon possible de Codex) | déclaré, testé (T010, T020), documenté (T012, T024) |
| A9 | basse | Fils T3 non couverts | hors périmètre, option 159c (R16) |
| A10 | basse | Les tests de 156 qui supposent l'ordre de reprise par `delivery_id` changent | T004 les réécrit |
| A11 | basse | 156 n'est pas livrée dans le checkout lu | T001, T002(7) |
| A12 | basse | Sans `turn_attest`, un porteur ancien ne bénéficie ni de la rétention ni des éditions | documenté ; mise à jour des wrappers avant l'ouverture du réglage |
