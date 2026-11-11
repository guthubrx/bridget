# Diagnostic266

L’aide `send --help` sortait en erreur avant toute connexion : le parseur de la sous-commande ne traitait aucun indicateur d’aide. La correction ajoute une aide locale et réemploie la même syntaxe pour les erreurs d’usage.

La façade refuse le neuvième appel parce que son registre borne à huit les appels simultanés. Ce refus intervient avant la résolution d’identité et la connexion au démon. Le nouveau message donne cette portée, les limites inchangées et la reprise du seul appel refusé. Les plafonds de quatre attentes et les identifiants déjà actifs gardent leur refus distinct. Aucune mission n’est annulée ou recréée.

Le texte « budget JSONL dépassé » désigne un délai de lecture, parce que les trois branches concernées rendent TimedOut sur l’échéance ou poll. La nouvelle formulation nomme le délai. Le budget de dix secondes et la limite de taille restent inchangés. Les traces historiques disponibles ne permettent pas d’attribuer la cause de la lenteur.

Les refus steer cités datent de 09:30 UTC, avant l’activation 263. Le guidage vers task_continue est déjà présent et qualifié. La tâche 9422ee1a a depuis un résultat disponible. Le refus CPU 94 % pour un seuil 90 % est attendu sur la cible imposée. Les revues 263 des descripteurs et de la garde ABI sont déjà intégrées ; le test ABI est branché en CI. Aucun effet métier n’est rejoué.

Le code reste portable : aucun hôte, répertoire personnel ou identifiant d’incident ajouté. La construction reprend la recette 265 qui inclut 264, puis applique uniquement ce delta. La qualification et la portée réelle de l’activation sont décrites ci-dessous.


## Qualification et revue

140 scénarios ciblés réussissent par plateforme Mac et Debian 12 : 136 pour la façade MCP, un CLI, trois JSONL ; un scénario préexistant reste ignoré. Analyse statique stricte réussie. La première recette d’aide a révélé l’initialisation prématurée : elle est maintenant traitée avant namespace et identité, y compris avec HOME absent. Le test d’inventaire a ensuite révélé la ligne loop-control manquante du lot 264 ; la ligne documentaire est ajoutée, sans changement du mandat.

Revue indépendante : ea20e49f-7093-473d-97bc-b4b83d5c5fa4, lue et accusée, garde propriétaire close et vue conservée. Aucun défaut bloquant ; l’ordre du message de refus est corrigé pour privilégier la fermeture ou le doublon même si quatre attentes sont ouvertes. La régression couvre désormais quatre attentes et quatre lectures, un doublon à plafond atteint, le neuvième appel puis sa reprise seule. Le code technique du refus reste inchangé. La remarque json-stdin concerne handoff send, pas send ; aucune option étrangère ajoutée. from reste une option de compatibilité, absente de l’usage normal centré sur l’identité courante.

Le diaporama autonome de l’aide est vérifié dans un navigateur local isolé : texte avant/après, précédent, suivant, lecture/pause et rendu visuel. Aucun clic métier ni capture reconstituée. La construction cumulative reproduit les huit fichiers attendus ; les modes 264 et les corrections 265 restent présents. Linux exige glibc 2.34, sous le plafond 2.36. L’APK 8033 garde le certificat de production.


## Activation et publication

Les sept services des quatre hôtes chargent 0.8.33 / 266-facade-guidance-r1. Les pointeurs privés de retour désignent 265-r2/0.8.32, image et formats vérifiés dans les deux sens sous verrou natif ; aucun retour exécuté. Coupure centrale de 2,54 secondes ; douze superviseurs Cartae préservés avec mêmes PID et dates de création. Les autres workers ont conservé leurs marqueurs. Les six actifs de v0.8.33 sont publiés, téléchargés anonymement et vérifiés par taille et empreinte. Source b07b4ca6 poussée ; trois checkouts distants propres et alignés avant la clôture.

T3 a été fermé normalement puis ouvert sans remplacement d’application : serveur 77762, version 0.1.7-local.20261111.bridget-toolbar inchangée. Neuf façades MCP nouvelles ont leur mapping exécutable 266 attesté. Le statut natif rend 266 / running=true ; l’annuaire lié à la session répond. Cinq reprises de maintenance encore en file ont été retirées après la reprise automatique ; les reprises déjà commencées et les messages utilisateur sont conservés. Les retours 266 aux auteurs Parix, mobilité et 264 sont stored_without_wakeup, reçus mcp-86189-6af469ca-1, mcp-86189-6af469cc-2 et mcp-86189-6af469cd-3. stored ne prouve pas une lecture.

Un client MCP autonome reste chargé en 265-r2, PID 47878, sous Claude 47847, tâche 6f301c37-ae8f-449f-a763-c8df818e51d9, working ronde 1. Le transport chargé ne propose pas de rechargement MCP à chaud. Le terminer imposerait une interruption du fournisseur d’une mission métier active ; cette mission et ses fichiers sont conservés. Son propriétaire e6476f91 est informé du guidage et de cette conservation. Les contrats restent compatibles, mais ce client n’a pas encore le nouveau texte de refus. L’activation de toutes les sessions natives ne peut donc pas être présentée comme complète. Les services et les sessions T3 sont activés ; aucune mission d’issue inconnue n’est rejouée pour obtenir une session neuve.

## Ménage

788716226 octets logiques retirés, environ 789 Mo : neuf exports propres terminés et une ancienne image SYS 265-r1. Les preuves et la revue sont conservées. Les exports de compilation Mac et SYS n’existent plus ; la recette versionnée permet leur reconstruction. La version active 266 et le retour 265-r2 restent installés.

Ménage partiel : Mac garde les images et caches lorsque l’observation des processus est incomplète, notamment le cwd du PID 33346 d’un autre travail. Cartae et KS gardent les images après refus de la garde d’observation ; Cartae conserve son export Android et ses transferts, KS sa copie de transfert après contrôle lsof non soldé. SYS garde 246 non privé sans observation globale privilégiée. Aucun de ces processus n’est arrêté pour nettoyer. Le client métier 265 encore chargé et son image restent conservés pour leur usage réel. Aucun worktree ni branche de correction n’a été créé.
