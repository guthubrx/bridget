# 177 — Accès mobile indépendant à une fédération Bridget

Date : 2026-10-24
Statut : conception acceptée ; déploiement à effectuer
[English](177-mobile-federation.md) · [Français](177-mobile-federation.fr.md)

## Contexte

Une application Android packagée doit piloter les agents de plusieurs machines, y compris quand le poste de bureau est hors ligne. La fédération existante possède une autorité unique et des reçus durables de message et de tâche. Certaines preuves de droits dépendent du runtime fournisseur sur sa machine d'origine.

## Décision

Créer une interface Android originale et une autorisation humaine d'appareil, limitée et révocable. Réutiliser l'annuaire, les projets canoniques, les reçus, le contrôle des tâches et les journaux de Bridget. Publier un composant en écoute locale derrière le proxy HTTPS existant sur un sous-domaine dédié.

Conserver une autorité de fédération unique. Préparer son déplacement vers un serveur permanent. Maintenir les validations de droits sur la machine d'origine grâce à une liaison authentifiée et négociée avant activation de cette migration. Un proxy ne suffit pas à prouver l'indépendance du poste de bureau.

L'autorisation de l'appareil est explicite. Elle n'emprunte pas une identité d'agent et ne fabrique aucune politique fournisseur. Les contrôles natifs d'admission restent l'autorité métier.

## Conséquences

Un seul moteur métier et aucune base de tâches répliquée. Le nouveau code porte l'authentification des appareils, la frontière HTTP et l'état Android. Les opérations exigent des charges bornées, des droits limités et un rendu fidèle des issues inconnues. La migration reste bloquée tant que les tests de préservation d'autorité et de récupération ne passent pas.

La première implémentation vise Android. Le basculement automatique d'autorité, un IDE complet et les autres plateformes relèvent de travaux distincts.

### Remise immédiate limitée au message autorisé

Le droit Message du téléphone associé autorise l’option `guide_now` après les
contrôles du téléphone, de la machine, du projet et du destinataire. Cette preuve
interne vaut pour cet envoi seul. Elle ne rend pas la socket administratrice,
n’usurpe aucun agent et n’autorise ni interruption ni redémarrage. L’intention et
le marqueur de repli font partie du canon d’origine. Le transport peut convertir
sa seule copie de remise en message ordinaire en file. Identifiant, date, texte
et reçu durable restent inchangés. Les demandes strictes de guidage hors mobile
conservent leurs contrôles et refus existants.

## Contenu des outils dans le journal natif

À la demande de l’utilisateur, les commandes, paramètres et résultats disponibles
des outils du fil principal peuvent être conservés dans le même journal privé
que la conversation. L’autorisation d’appareil et la portée de lecture restent
obligatoires. Le téléphone ne reçoit aucun identifiant d’accès de bureau et ne
contacte aucune API de bureau. Les notifications ne contiennent toujours aucun
contenu de conversation ou d’outil. Les longs extraits sont explicitement bornés.
