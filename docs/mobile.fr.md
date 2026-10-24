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
