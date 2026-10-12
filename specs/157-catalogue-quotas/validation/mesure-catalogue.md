# Mesure catalogue 157

Même banc privé, 10 entrées, 50 appels, sans préchauffage.

Avant : médiane 40,201 ms ; p95 42,698 ms. Première implémentation : 183,236 ms / 206,447 ms ; seuil échoué. Après R8 : médiane 23,623 ms ; p95 25,244 ms. Rapports 0,588 / 0,591 ; SC-007 réussi.

R8 prépare une entrée une seule fois. Chaque posture garde ses propres branches ; aucun résultat ne survit à l’appel du catalogue. L’admission fait une nouvelle préparation. La matrice et les quotas passent sur 19 tests ciblés. Aucun succès de fournisseur réel déduit.
