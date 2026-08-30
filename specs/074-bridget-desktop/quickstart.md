# Vérification opérateur - SPEC-074

## Installer et ouvrir le client

1. Ouvrir `/Users/moi/Applications/Bridget Desktop 0.1.0.app`.
2. Comme ce premier paquet est signé ad hoc localement, macOS peut le bloquer une première fois. Dans le Finder : clic droit sur l'application, puis `Ouvrir` et confirmer.
3. La page initiale ne doit afficher aucun agent distant avant l'ajout d'un profil.

Le client n'expose aucun nouveau port serveur. Les agents, le daemon et le relais restent sur le serveur choisi.

## Préparer une connexion distante

1. Ouvrir Bridget Desktop sur le Mac.
2. Ajouter un profil nommé, par exemple `cartae.app`.
3. Renseigner `cartae.app`, le port SSH `2222`, le compte, puis choisir l'agent SSH du Mac ou un chemin de clé déjà existant. Ne jamais coller une clé privée.
4. Vérifier l'empreinte affichée à la première connexion avant de l'accepter.
5. Vérifier que la vue affiche le relais vérifié puis ouvre le panneau du serveur. L'en-tête doit toujours montrer `cartae.app - SSH`.
6. Cliquer `Gérer les serveurs`, puis `Déconnecter` : le panneau doit disparaître sans supprimer le profil.

## Vérifier un refus honnête

1. Modifier volontairement l'empreinte connue dans un environnement de test, ou présenter un serveur différent sous le même profil.
2. Vérifier que l'application bloque avant de charger le relais et explique le changement d'identité.
3. Vérifier que le diagnostic ne révèle aucune clé ni jeton.

## Vérifier deux serveurs

1. Ajouter deux profils distincts et les connecter.
2. Connecter puis ouvrir les deux profils.
3. Vérifier que chaque panneau porte son serveur, ses agents et ses notifications propres.
4. Couper le premier tunnel de test : le premier profil doit afficher `Tunnel interrompu - réessayez explicitement`, sans dégrader le second. Le bouton `Réessayer` relance uniquement ce profil.

## Vérifier le local

1. Ajouter un profil local sur un Mac sans relais local : l'application doit signaler précisément l'absence du relais, sans erreur SSH.
2. Lorsqu'un relais local est disponible, ouvrir le profil et vérifier qu'aucun tunnel SSH n'est lancé.

## Limites assumées

Cette SPEC ne lance ni ne montre de navigateur distant. Une telle session sera créée dans une SPEC ultérieure, isolée par exécution et transportée sur un canal local du serveur à travers SSH.

La distribution à un autre Mac nécessitera plus tard une signature Developer ID et une notarisation Apple. Elles ne sont pas nécessaires au test local de cette version.
