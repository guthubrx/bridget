# Bridget pour Android

[English](mobile.md) · [Français](mobile.fr.md)

Retrouvez vos agents existants, envoyez une instruction, lisez les résultats,
répondez aux questions et utilisez les actions de tâche disponibles. Choisissez
une machine ou toute la fédération. Le téléphone possède sa propre autorisation
humaine révocable. Il n'emprunte aucune identité d'agent. Il ne crée pas directement
de nouveaux agents.

## Associer le téléphone

1. Installez `bridget-android.apk`, signé et publié dans la version officielle de
   Bridget. Android demande une fois l'autorisation d'installer depuis le navigateur.
2. Dans un terminal interactif de l'installation maître de confiance, exécutez
   `bridget mobile pair --all`. Le QR apparaît dans ce terminal. Pour limiter les
   droits, utilisez `--scope-file` avec le chemin absolu d'un fichier privé.
3. Ouvrez Bridget sur Android, choisissez **Scanner le QR**, puis scannez ce code.
   Il expire après cinq minutes et n'associe qu'un appareil.
4. Choisissez la machine, le projet puis un agent précis. Écrivez votre instruction.
   Enregistré, reçu, remis et terminé sont des états différents.

L'administrateur peut exécuter `bridget mobile devices` et
`bridget mobile revoke DEVICE_UUID`. Les réglages Android proposent aussi la gestion
des appareils. Oublier une autorisation locale ne révoque pas son enregistrement serveur.

## Reprendre après une coupure

La commande est enregistrée avant l'envoi. Si la réponse est perdue, consultez son
reçu. Un rejeu strict conserve clé, date et contenu. L'application ne fabrique pas
une nouvelle clé pour masquer une issue inconnue. Une protection contre les doublons
expirée exige une décision explicite. Le stockage Android chiffre brouillons,
commandes et association avec une clé Android Keystore. Les pièces jointes sont
limitées à 8 Mio par fichier et 32 Mio au total sur le téléphone. Aucun exécutable
ni chemin libre vers le disque n'est accepté.

Une action peut être une instruction au parent plutôt qu'une action fournisseur
confirmée. Lisez son reçu et l'état de la tâche. Une réponse enregistrée diffère
de son acquittement par l'agent. Les lacunes d'historique et les machines déconnectées
restent visibles. Une observation en cache n'est pas un état actuel.

## Notifications et période silencieuse

Les notifications sont facultatives. Activez-les dans les réglages et acceptez
la demande Android. La période silencieuse suit l'heure locale du téléphone.
Les messages restent consultables pendant cette période. La notification ouvre
Bridget ; l'application relit l'état autorisé. Elle ne prouve pas la fin d'une tâche.

Firebase Cloud Messaging reçoit seulement une version et un condensat de liaison
d'association. Aucun texte de message, fichier, nom d'agent ou secret fournisseur
n'est transmis. Google assure néanmoins le transport. La configuration publique
Firebase de l'APK doit correspondre au projet du compte serveur. Aucun SDK Analytics
n'est inclus. Une configuration ou une permission absente est signalée ; l'accès
interactif reste utilisable.

## Administrer le serveur

Un seul maître permanent permet de joindre les agents serveur quand un poste est
hors ligne. Un relais HTTPS seul ne déplace pas cette autorité. Une migration doit
conserver identités, reçus natifs, placements des tâches et attestations d'origine.
Ne copiez pas une base ouverte. Ne démarrez pas deux maîtres pour la même fédération.
Terminez les vérifications de migration avant de remplacer le maître actif.

Dans un terminal interactif local du maître existant :

```sh
bridget mobile configure --public-url https://bridget.example.net
bridget mobile configure-notifications --service-account-file /home/operator/private/fcm-account.json
bridget mobile serve
```

Le compte reste dans un fichier privé en 0600, hors Git. Utilisez un compte dédié
avec le seul rôle `roles/firebasecloudmessaging.admin`. FCM ne demande pas de
facturation. Le proxy de confiance publie uniquement l'écoute loopback, normalement
`127.0.0.1:9467`. Il doit remplacer l'en-tête d'adresse source :

```caddyfile
bridget.example.net {
    request_body {
        max_size 12MB
    }
    reverse_proxy 127.0.0.1:9467 {
        header_up X-Bridget-Client-IP {remote_host}
    }
}
```

Exécutez `bridget mobile serve` avec le gestionnaire de services natif existant.
Gardez le namespace et la socket du maître. L'authentification précède lectures
privées et effets. L'association possède un quota par source. Le renouvellement
possède un quota distinct par appareil connu. Connexions, travaux, uploads et suivis
sont bornés. L'en-tête du proxy n'est pas un identifiant d'authentification.

Pour raccorder un poste au maître serveur, l'installateur de fédération existant
accepte `--direction to-master`. `--root` et `--socket` désignent alors le namespace
client local. `--remote-root` et `--remote-socket` désignent le maître distant.
Le raccordement utilise une socket Unix privée et les flux SSH authentifiés.
Aucun port TCP natif n'est public. `socat` est requis sur les deux machines.
Une liaison existante garde son sens ; une direction explicite divergente est
refusée. La commande affiche la direction réutilisée. Les autres paramètres sont
décrits dans le [guide de fédération SSH](federation-services.fr.md).

## Mise à jour et récupération

Les réglages consultent les versions publiques du dépôt GitHub officiel. Seul
l'actif stable `bridget-android.apk` de ce dépôt est proposé. Aucun identifiant de
téléphone n'est envoyé lors de cette lecture. Une version indisponible diffère
d'une application à jour. Le navigateur télécharge l'APK. Android vérifie sa
signature et demande l'accord d'installation. Conservez les données de l'application.

Sauvegardez la clé de signature hors Git. Ne désinstallez pas l'application pour
résoudre une différence de signature : cela supprimerait les données locales.
Une installation de développement exige une transition de signature vérifiée
avant sa première mise à jour de production. La version Android suit celle du
workspace. Le build utilise `BRIDGET_FIREBASE_CONFIG` si les notifications sont
activées. Ce fichier contient les options publiques de l'application, jamais la
clé du compte serveur. Gardez la version installée et un paquet de retour vérifié.

## Conversation et remise des messages

Les projets affichent le nom du dépôt, pas son dossier interne `.git`. Touchez un
projet pour lire son identité complète. La conversation ouvre les messages
récents ; les messages précédents se chargent à la demande. Le bouton vers les
derniers messages et l’ouverture du clavier gardent la fin visible, sauf si vous
avez fait défiler la conversation pour lire ailleurs.

**Envoyer** utilise la file existante. **Transmettre maintenant** demande une
remise pendant que l’agent travaille. Si sa connexion ne le permet pas, le même
message attend dans la file. Cela ne demande aucun redémarrage de l’agent et ne crée aucun
message de remplacement. Selon le fournisseur, la prise en compte immédiate peut
interrompre un outil en cours. Codex natif peut recevoir ce guidage immédiat. Le
connecteur de bureau actuel le permet pour Claude ; ses autres fournisseurs
utilisent la file. Claude natif et ACP conservent également le message en file
quand le guidage immédiat est indisponible.

Un message enregistré ne prouve pas sa remise. L’application affiche **Message
remis** seulement après avoir observé son entrée correspondante dans le journal.
Relire un reçu n’envoie rien. Une réponse réseau perdue conserve l’enveloppe
d’origine. Un refus avant admission conserve le message sur le téléphone pour
un envoi explicite après rétablissement.

La liste des conversations utilise des lignes courtes : repère de projet coloré,
titre, nom court du projet et étoile de favori. Les chemins complets et les
informations de connexion restent dans les détails de conversation. L’icône de
l’application utilise le logo Bridget.

Les outils utilisent le même journal Bridget que les messages. Les groupes et
outils sont repliés par défaut. Dépliez un outil pour lire sa commande ou ses
paramètres, son résultat et son code de sortie, lorsqu’ils sont disponibles.
Le connecteur de bureau lit seulement les outils du fil principal. Il récupère
une sortie omise par l’opération de détail déjà fournie par la source, au plus
un élément par passage. Android contacte seulement Bridget. Les autres
transports Bridget restent autonomes. Un extrait de sortie longue est borné et
signalé ; une absence de détail n’est pas présentée comme un résultat vide réussi.
Les champs d’un outil restent sous 10 Kio pour que son événement complet tienne
dans la limite de lecture de 16 Kio. La sortie n’est pas copiée caractère par caractère.

Après une nouvelle association de ce téléphone, l’activation des notifications
peut réutiliser son jeton FCM. Seul l’enregistrement de réception passe à
l’association courante. Les identifiants, portées et droits ne sont pas transférés.

Le lanceur affiche le b Bridget sur fond noir. Les lignes d’outil affichent directement leur nom ou commande ; le dépliage montre l’entrée et le résultat.


## Markdown, code et aperçu HTML

Les réponses affichent titres, listes, tableaux, cases à cocher, texte barré et
blocs de code. Le code Python, JavaScript/JSON, TypeScript, Rust, Kotlin, Java,
C/C++, C#, Go, PHP, Ruby, Swift, Dart, Perl et les commandes shell peuvent être
colorés. Un langage inconnu reste du texte lisible. La coloration est limitée
à 32 768 caractères par bloc ; le reste du texte demeure complet.

Sous un bloc HTML, touchez **Aperçu HTML**. Pour un fichier partagé, ouvrez le
menu de conversation, puis **Fichiers** et **Aperçu**. La liste se charge à son
ouverture. Les fichiers Markdown sont rendus ; les fichiers de code conservent
leur texte. L’aperçu accepte au plus 256 Kio de texte UTF-8. **Enregistrer une
copie vérifiée** conserve le fichier complet, dans la limite générale de 8 Mio.

Le rendu est embarqué dans Android et ne contacte aucune interface de bureau.
L’aperçu propose **Interactif**, **Statique** et **Code**. Le mode interactif exécute
les scripts intégrés dans un document isolé. Les boutons, graphiques et formulaires
locaux fonctionnent. Le réseau, la navigation, les téléchargements, le stockage,
les fichiers, le microphone et la caméra restent interdits. Un document qui charge
une bibliothèque externe doit l’intégrer dans sa source. Le mode statique désactive
les scripts. Un moteur WebView ancien ou inconnu ne reçoit que le code et le guidage
de mise à jour. Le bouton Fermer appartient à Bridget.

Chaque bloc de code racine affiche son langage et **Copier**. Le texte copié conserve
ses accents, ses espaces et ses sauts de ligne. Dans les listes et citations, le code
garde sa structure et sa sélection habituelles.

## Volet, attention et saisie

Depuis le fil, touchez le menu en haut à gauche. Le volet affiche l’annuaire existant,
les favoris et une recherche par nom, machine ou projet. Les filtres de machine et de
projet restent visibles. **Plus** charge les pages suivantes. La recherche porte sur
les agents chargés et les favoris conservés, pas sur une liste distante invisible.
Le changement de fil conserve son brouillon et sa position de lecture pendant cette
session de l’application.

Les marqueurs possèdent un libellé : **Réponse attendue**, **À consulter**, **En cours**
ou **Observation à actualiser**. Une question fraîche a priorité, même sur un favori ancien. Une question non relue porte « Question à vérifier » ; une réponse déjà soumise porte « Réponse en attente de confirmation ». Une réponse terminale reçue,
ou une transition d’activité fraîche dans la même génération de connexion, peut
signaler un élément à consulter. Ce marqueur ne certifie pas une réussite. Les
activités très courtes d’un agent non suivi peuvent ne pas être observées. Le volet
seul ne marque rien comme lu : il faut afficher la fin du bon fil avec un suivi vivant.
Un échec de rafraîchissement conserve les données avec leur statut ancien.

La loupe recherche dans les messages déjà chargés. Le nombre de résultats et les
boutons précédent/suivant permettent de les atteindre. Chargez des messages plus
anciens pour élargir cette recherche. Le bouton de dictée ouvre la reconnaissance
vocale du système. Son texte revient dans le brouillon et reste modifiable. Elle
n’envoie rien. Un changement d’agent ou d’association empêche l’insertion ailleurs.
La disponibilité et le traitement audio dépendent du service vocal choisi par Android.

Le menu **+** prépare une photo ou un fichier sur cet appareil. Sa carte peut être
retirée avant transmission. La flèche envoie explicitement les pièces jointes, puis
le message avec leurs références. Les reçus des fichiers et du message restent
séparés. Après une erreur partielle, le même fichier reprend sa commande ; il n’est
pas recréé. Retirer une carte déjà transmise ne supprime pas un artefact du serveur.

Quand le producteur fournit un événement structurel correspondant au tour courant,
l’en-tête affiche **Compactage du contexte…**. La fin de l’item ou du tour ferme ce
texte. La perte de suivi ou le changement de fil le ferme aussi. Un ancien début
mis en cache n’invente pas une activité en cours. Aucun pourcentage n’est estimé.
