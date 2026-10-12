# Transport155 — validation ciblée

Les quatre tests transport prévus T012/T014 passent après implémentation T013/T015. Les trois tests du classifieur sont rejoués car son champ `reason` a changé. Résultat : 7 PASS, 0 FAIL, RC0, 29,04 s, sources inchangées pendant exécution.

La suite complète155 reste réservée à T039. Aucune preuve d’exécution fournisseur n’est déduite de write+flush. Le rouge MCP initial avait un oracle erroné ; le rejeu du seul test corrigé démontre le motif de refus manquant. Voir transport155-red-gate.json.

Le build a supprimé 20,21 Gio de cache au franchissement du plafond20 Gio. Le plafond du processus suivant est40 Gio. Aucune configuration persistante ni base n’a été modifiée.
