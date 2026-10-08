# Contre-revue locale du plan — SPEC147

Date : 2026-10-08. Périmètre : conception, lecture seule. Résultat relayé et validé par le principal : changements intégrés avant GO implémentation. Aucun audit de code147 ou verdict de recette revendiqué.

## Quatre points retenus

1. **Snapshot historique.** Un snapshot ancien masque un remplacement arrivé ensuite. Acquérir S par la tête history_recent, reconstruire uniquement le segment déjà chargé sous to_seq=S commun, puis publier atomiquement. Invalider la page ancienne en vol. Test : consigne page2 remplacée pendant chargement page3.
2. **Premier événement protégé.** ready seq0 doit être obligatoire, premier et non coalescible. Une rafale>16 avant l'écriture socket ne peut pas l'écraser. Seuls les changements suivants sont coalescés en resync dans la file bornée.
3. **Refus terminal.** Le supervisor peut reprendre le transport transitoire, mais pas reconnecter en boucle projet refusé, version incompatible ou demande invalide. Une visite ou un refresh manuel peuvent revalider. Aucun timer de contenu ajouté.
4. **Sélection et autorité.** Seul thread_unavailable confirmé par read du fil choisi efface son UUID. Un binding ou réseau indisponible garde le choix sans afficher des corps non vérifiés. Le refus de contexte arrête le flux et masque/purge les corps ; il ne prouve pas la suppression du fil.

## Corrections et tests associés

Les points sont présents dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md et plan/data-model/tasks du même dossier. T007/T008 couvrent ready/saturation ; T019–T021 couvrent refus/reprise/choix ; T022–T024 couvrent snapshot et gestes de lecture.

Le principal a relu les corrections et a donné le GO aux trois lanes après Analyze documentaire. Le signal global retenu est version/generation/seq/status, sans UUID de fil ni contenu.

Limite : aucun fournisseur adverse distinct joignable dans le même projet. Cette revue est locale et ne sera pas présentée comme inter-fournisseurs. Les tests réels restent à exécuter et à recevoir.
