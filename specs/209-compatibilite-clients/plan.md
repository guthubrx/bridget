# Plan209

Réutiliser AgentRegistry et sa politique de connexions, la version/identité locale et les ports installateur/updater existants. Ajouter un seul module de rapport local et une sonde bornée --version, sans dépendance ni accès fournisseur. Le candidat porte la politique, pas l’ancien binaire actif. Dédupliquer moteur+commande, plafonner à16 programmes ; budget global10s, sorties4Kio/programme, arrêtTERM des seules sondes créées.

Publier le diagnostic de compatibilité dans le statut d’update en conservant les anciens rapports sans diagnostic. Formats de tâches, workspaces et preuves inchangés. Tester ancien/tropancien/versioninconnue/absence, séparation des machines, commandes avec espaces, timeout/sortie excessive/texte privé, installation et activation sans effet quand incompatible.

Revue indépendante après tests. Construire et charger les exécutables validés sur les trois serveurs et le Mac, vérifier les processus exacts et prévenir les propriétaires. Aucun autre client à mettre à jour sans accord spécifique.
