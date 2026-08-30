# Vérification opérateur - SPEC-074

## Installer et ouvrir le client

1. Ouvrir `/Users/moi/Applications/Bridget Desktop 0.1.0.app`.
2. Comme ce premier paquet est signé ad hoc localement, macOS peut le bloquer une première fois. Dans le Finder : clic droit sur l'application, puis `Ouvrir` et confirmer.
3. La page initiale ne doit afficher aucun agent distant avant l'ajout d'un profil.

Le client n'expose aucun nouveau port serveur. Les agents, le daemon et le relais restent sur le serveur choisi.

## Choisir comment joindre un relais

1. Ouvrir Bridget Desktop sur le Mac.
2. Ajouter un profil nommé, par exemple `cartae.app`.
3. Si un endpoint existe déjà sur le Mac, choisir `Endpoint déjà accessible depuis ce Mac`, renseigner `127.0.0.1` ou `localhost` et son port réel, par exemple `17893`. Le client ne cherche pas à savoir si ce port vient d'un tunnel SSH, d'un proxy ou d'un autre chemin local.
4. Sinon choisir `Bridget Desktop ouvre un tunnel SSH`, renseigner `cartae.app`, le port SSH `2222`, le compte, puis choisir l'agent SSH du Mac ou un chemin de clé déjà existant. Ne jamais coller une clé privée.
5. Pour un endpoint existant, fournir le jeton du relais à la connexion. Pour le tunnel géré, vérifier l'empreinte affichée à la première connexion avant de l'accepter.
6. Vérifier que la vue affiche le relais vérifié puis ouvre le panneau du serveur.
7. Cliquer `Serveurs`, puis `Déconnecter` : le panneau doit disparaître sans supprimer le profil.

## Vérifier un refus honnête

1. Modifier volontairement l'empreinte connue dans un environnement de test, ou présenter un serveur différent sous le même profil.
2. Vérifier que l'application bloque avant de charger le relais et explique le changement d'identité.
3. Vérifier que le diagnostic ne révèle aucune clé ni jeton.

## Vérifier deux serveurs

1. Ajouter deux profils distincts et les connecter.
2. Connecter puis ouvrir les deux profils.
3. Vérifier que chaque panneau porte son serveur, ses agents et ses notifications propres.
4. Couper le premier tunnel de test : le premier profil doit afficher `Tunnel interrompu - réessayez explicitement`, sans dégrader le second. Le bouton `Réessayer` relance uniquement ce profil.

## Vérifier un endpoint existant

1. Ajouter un endpoint loopback sur un Mac sans relais : l'application doit signaler précisément l'absence du relais, sans erreur SSH.
2. Lorsqu'un endpoint existe, y compris derrière un tunnel SSH déjà ouvert, ouvrir le profil et vérifier qu'aucun tunnel SSH n'est lancé par Bridget Desktop.

## Limites assumées

Cette SPEC ne lance ni ne montre de navigateur distant. Une telle session sera créée dans une SPEC ultérieure, isolée par exécution et transportée sur un canal local du serveur à travers SSH.

La distribution à un autre Mac nécessitera plus tard une signature Developer ID et une notarisation Apple. Elles ne sont pas nécessaires au test local de cette version.
