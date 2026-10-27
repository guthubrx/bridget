# Plan
1. Reproduire le rejet de Released après retrait de l'attente corrélée.
2. Porter l'attente de Release à trente secondes, comme Start ; ne modifier aucun permis.
3. Tolérer l'accusé tardif authentifié d'une commande connue. La réconciliation existante reste propriétaire de la progression.
4. Tester un accusé retardé de six secondes, les identités et les régressions pertinentes.
5. Faire relire indépendamment, intégrer, pousser et livrer via la maintenance native.
Complexité inchangée : lookup des attentes O(1), validation de la commande existante ; aucune file ou reprise nouvelle.
