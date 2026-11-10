# Plan 259

Réutilisation : delegation_mcp::pre_dispatch_failure existe depuis230, avec deux usages de précontrôle. Cherché par nom et responsabilité, puis lu avant édition. Le rendre visible au module MCP, sans nouveau helper, endpoint, table ou dépendance. Appliquer le même guidage au raccordement auxiliaire et à la lecture CallerScope, avant le dispatch de l'outil. Aucun budget modifié.

Tests : reprendre registered196/drop_scope196 et les fixtures IPC existantes. Une vraie attente Unix de10s reproduit le délai du contrôle : observer la fermeture sans trame ACK et sans nouvelle connexion. Vérifier aussi une coupure pour lecture/mutation. Les régressions151/196/230 couvrent ACK, refus, reprise de lecture et issue inconnue après écriture.

Analyze manuel : FR01=T002/T003 ; FR02=T003 ; FR03=T002/T003 ; FR04=T004/T005. Aucune ambiguïté bloquante. Pas de rendu UI modifié ; preuve par réponse MCP et oracle IPC, sans diaporama de composant.

Revue indépendante automatique avant livraison. Construction par build.py, caches reconnus conservés ; sources et preuves identiques Mac/Linux. Réutiliser les scripts de maintenance258 après vérification, nouveau build259-preflight-guidance-r1 en0.8.26 ; retour258. Aucune relance T3 ni mission métier. Refus de délégation : ne pas contourner, continuer les contrôles locaux.

La suite MCP élargie a révélé deux défauts de qualification déjà présents dans065f54f0 : le champ optionnel quota_fallback_authorized manquait dans un attendu160 ; le parseur094 assimilait les sous-commandes imbriquées à des commandes racines, et sentinel manquait à la table. Les champs/entrées de production sont inchangés dans ce lot. Attendu et inventaire corrigés, sans autoriser la campagne244 arrêtée. Rejouer la suiteMCP avant clôture.
