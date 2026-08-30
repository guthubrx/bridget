# Recherche technique - SPEC-070

## Faits mesurés

1. Le message humain `e74721eab4164` a reçu `turn_start` puis
   `prompt_dispatched` à 09:12:10 CEST. Huit demandes
   `item/commandExecution/requestApproval` ont été acceptées jusqu'à 09:12:56.
   Le journal porte ensuite `turn_failed` à 09:13:10 avec
   « échéance Codex dépassée ».
2. La définition Codex courante fixe `notify_timeout_secs` à 2700 secondes.
   Le chemin `handle_idempotent_send` pose néanmoins pour une demande avec
   réponse l'échéance `issued_at + reply_timeout`, dont la valeur implicite est
   60 secondes.
3. Les tours de ronde suivants ont terminé normalement. Il ne s'agit donc pas
   d'une indisponibilité générale du fournisseur.

## Réutilisation confirmée

| Besoin | Existant | Décision |
|---|---|---|
| Activité d'outil | `projectTimeline`, `JOURNAL_ACT_KINDS`, `formatPermissionAct` dans `assets/ui/app.js` | Réutiliser la projection et en dériver le dernier acte non terminal. |
| Bouille agent | apparence et rendu d'avatar dans `assets/ui/app.js` et `theme.css` | Réutiliser l'avatar sélectionné, sans nouvel asset. |
| Défilement hors du bas | `renderThread`, puce `new-messages`, SPEC-069 | Étendre la cible de message sans changer la politique existante. |
| Journal live | EventSource de `connectWatch` | Réutiliser. Aucun endpoint ni stockage supplémentaire. |
| Fin de tour et erreur | événements `turn_end` et `error` déjà projetés | Relier au message source et enrichir le texte visible. |
| Notifications | aucune utilisation locale de `Notification` | Créer une petite adaptation client dans `app.js`, sans dépendance. |

## Cause du plafond de 60 secondes

`BridgetMessage.reply` a deux sens qui se recouvrent aujourd'hui : le suivi
métier d'une réponse et l'échéance de l'exécution transport. Cette dernière ne
doit pas hériter de la valeur par défaut de soixante secondes.

La correction doit conserver le suivi de réponse existant, mais laisser le
transport appliquer l'échéance du type d'agent. Un contrôle explicite
`SteerCurrent` ou `InterruptAndStart` garde son délai court et son traitement
spécifique.

## Notification navigateur

La norme Notifications impose une permission explicite. Elle définit un clic,
mais l'indique comme meilleur effort selon la plateforme. Le plan prévoit donc
deux chemins équivalents : clic de notification quand il est livré et cible
interne persistante dans le fil.

- Source normative : https://notifications.spec.whatwg.org/
- Source complémentaire : https://www.w3.org/TR/2020/SPSD-notifications-20200602/

Le navigateur ne peut notifier que tant que cette page est ouverte. La Push API
et le service worker sont volontairement hors périmètre, car ils exigeraient un
abonnement persistant et un serveur d'envoi.

## Risques et garde-fous

- Une trace de remise n'est pas une activité agent : seuls les événements
  fournisseur sont éligibles à l'indicateur.
- La ligne compacte ne montre qu'un libellé sûr déjà projeté. Les arguments de
  commande et les sorties restent dans le détail volontairement ouvert.
- Une erreur ne doit jamais être attribuée au mauvais message : la corrélation
  par `message_id` reste obligatoire.
- Les notifications ne doivent pas s'accumuler : une cible de tour remplace la
  notification précédente du même message.

## Recherche DevKMS

La commande `mem` n'est pas installée sur le serveur. Aucune connaissance
existante n'a donc pu être consultée ni enrichie. Ce manque ne bloque pas la
SPEC ; les sources et preuves sont conservées dans cet artefact.
