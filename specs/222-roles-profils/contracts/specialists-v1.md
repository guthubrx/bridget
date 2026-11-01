# Contrat222 — opérations natives
Version1, requêtes fermées, identifiants opaques et références de version exactes.
Négocier SpecialistsV1 avant une nouvelle opération ; ancien pair : refus nommé.

## Agent
list/search/read/export : lecture autorisée par périmètre.
declare_roles/withdraw_roles : seulement sa propre identité attestée.
propose_profile/propose_revision : conservation sans activation permanente.
request_role : demande idempotente, rôle+périmètre+travail ; responsable figé.
memory_recall/write/correct/forget : identité et accès aux sources recontrôlés.
conversion_prepare : proposition sourcée sur conversation que l'appelant possède.
Une action n'est pas administrative du seul fait qu'un champ actor l'affirme.

## Humain
create/update/approve_revision/activate_durable/disable/mandate/budget/convert/import
passent par le contrôle humain natif. Une conversation peut demander cette décision
dans la boîte humaine existante ; le jeton de décision est lié à la commande exacte.
La lecture du catalogue ne lance rien ; l'import ne réveille rien.
Annuler une mission reste une opération native distincte.

## États et refus
Définition : proposée/active/retirée. Spécialiste : conservé/durable/démarrage
automatique désactivé ; présence : observée/hors_ligne/inconnue.
Demande : conservée/en_attente/remise/prise_en_charge/résultat_disponible,
avec les faits natifs distincts. Livrable validé est une preuve supplémentaire.
Refus : cause constatée ou inconnue, portée, références, prochaine action
et décision humaine ciblée. Aucun changement de moteur/droits/comptes implicite.
Clé connue + même canon : mêmes références ; divergence : envelope_mismatch.
Une absence de résultat ne prouve pas une absence d'exécution.

## Provenance et limites
Un souvenir marque source et projet ; un résultat rapporté n'est pas une preuve
de réussite métier. La consultation interprojets reste autorisée par source.
Budget par défaut désactivé (valeur absente), ni zéro déguisé ni limite cachée.
Les garde-fous de transport sont des contraintes techniques distinctes.
