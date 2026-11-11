# Diagnostic266

L’aide `send --help` sortait en erreur avant toute connexion : le parseur de la sous-commande ne traitait aucun indicateur d’aide. La correction ajoute une aide locale et réemploie la même syntaxe pour les erreurs d’usage.

La façade refuse le neuvième appel parce que son registre borne à huit les appels simultanés. Ce refus intervient avant la résolution d’identité et la connexion au démon. Le nouveau message donne cette portée, les limites inchangées et la reprise du seul appel refusé. Les plafonds de quatre attentes et les identifiants déjà actifs gardent leur refus distinct. Aucune mission n’est annulée ou recréée.

Le texte « budget JSONL dépassé » désigne un délai de lecture, parce que les trois branches concernées rendent TimedOut sur l’échéance ou poll. La nouvelle formulation nomme le délai. Le budget de dix secondes et la limite de taille restent inchangés. Les traces historiques disponibles ne permettent pas d’attribuer la cause de la lenteur.

Les refus steer cités datent de09:30UTC, avant l’activation263. Le guidage vers task_continue est déjà présent et qualifié. La tâche9422ee1a a depuis un résultat disponible. Le refus CPU94% pour un seuil90% est attendu sur la cible imposée. Les revues263 des descripteurs et de la garde ABI sont déjà intégrées ; le test ABI est branché en CI. Aucun effet métier n’est rejoué.

Le code reste portable : aucun hôte, répertoire personnel ou identifiant d’incident ajouté. La construction reprend la recette265 qui inclut264, puis applique uniquement ce delta. Qualification et activation à compléter.


## Qualification et revue

140 scénarios ciblés réussissent par plateforme Mac et Debian12 : 136 façadeMCP, un CLI, trois JSONL ; un scénario préexistant reste ignoré. Analyse statique stricte réussie. La première recette d’aide a révélé l’initialisation prématurée : elle est maintenant traitée avant namespace et identité, y compris avec HOME absent. Le test d’inventaire a ensuite révélé la ligne loop-control manquante du lot264 ; la ligne documentaire est ajoutée, sans changement du mandat.

Revue indépendante ea20e49f-7093-473d-97bc-b4b83d5c5fa4, lue et accusée, garde propriétaire close et vue conservée. Aucun défaut bloquant ; l’ordre du message de refus est corrigé pour privilégier la fermeture ou le doublon même si quatre attentes sont ouvertes. La régression couvre désormais quatre attentes et quatre lectures, un doublon à plafond atteint, le neuvième appel puis sa reprise seule. Le code technique du refus reste inchangé. La remarque json-stdin concerne handoff send, pas send ; aucune option étrangère ajoutée. from reste une option de compatibilité, absente de l’usage normal centré sur l’identité courante.

Le diaporama autonome de l’aide est vérifié dans un navigateur local isolé : texte avant/après, précédent, suivant, lecture/pause et rendu visuel. Aucun clic métier ni capture reconstituée. La construction cumulative reproduit les huit fichiers attendus ; les modes264 et les corrections265 restent présents. Linux exige glibc2.34, sous le plafond2.36. L’APK8033 garde le certificat de production.
