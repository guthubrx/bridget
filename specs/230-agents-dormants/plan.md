# Plan 230

1. Définir un contrat natif typé, une persistance transactionnelle, des reçus
   idempotents et un journal des choix. Réutiliser les principaux humains et
   mobiles existants. Conserver les liaisons et actions de synchronisation.
2. Appliquer l'état dans les chemins réels : envois historiques/idempotents,
   notifications de groupe, réveils programmés, sélection des spécialistes.
   Distinguer la collecte d'un résultat antérieur de sa notification active.
3. Ajouter la CLI, les lectures paginées et les actions Android. Corriger la
   conservation indéfinie des fiches absentes par les rafraîchissements discrets.
4. Raccorder l'adaptateur au contrat natif. Dédupliquer les transitions et
   confirmer les effets du client. Conserver une opération en attente lors
   d'une indisponibilité. Ne pas importer des fils non liés.
5. Vérifier les invariants avec tests ciblés et parcours réels sans inférence.
   Faire relire indépendamment. Réutiliser les preuves inchangées.
6. Livrer la version exécutable, les façades et Android ; vérifier les images
   chargées, les retours compatibles, le lien natif. Intégrer, pousser et nettoyer.

## Risques à résoudre avant activation

- La protection doit couvrir les voies historiques comme les nouvelles voies.
- Un retour métier ne doit pas se perdre parce que son propriétaire dort.
- Un événement client issu d'un message ne constitue pas un réveil explicite.
- Le retour logiciel doit comprendre les marqueurs natifs de sommeil, faute
  de quoi une ancienne version permettrait de solliciter un agent dormant.
- Le périmètre mobile est contrôlé avant pagination et avant écriture.
