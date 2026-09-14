# ADR034 — Adaptateur t3code sans modification de t3code

2026-09-14 · Accepté · Session098

## Contexte

L'humain pilote certains agents dans t3code et veut les joindre depuis Bridget
sans « trifouiller » t3code, avec un module installable et retirable. t3code
0.0.40 publie un serveur HTTP local (`server-runtime.json`), un snapshot des
fils, un détail paginé par fil et une commande `thread.turn.start` dédupliquée
par `commandId` ; son CLI `t3` émet des sessions porteuses. Il n'expose ni
séquence par fil, ni portées restreintes, ni hook de sortie.

## Décision

Un pont côté Bridget (`bridget t3 install|status|uninstall|serve`) qui :
- n'écrit jamais dans t3code : session obtenue par la commande officielle, état
  du pont sous `<BRIDGET_HOME>/t3code` en 0600, révocation au retrait ;
- présente chaque fil vivant comme un agent Bridget (une connexion daemon par
  fil, transport `t3code`, mode `cli`, identité dérivée de l'identifiant du fil
  et présentée en UUID v4 canonique parce que le daemon l'exige, nom humain =
  titre du fil par `DisplayNameSet`) ;
- remet un message par `thread.turn.start` après attente bornée d'un fil sans
  tour actif, avec `commandId` = identifiant du message Bridget ;
- renvoie la réponse liée par rang, **et seulement quand le rang est prouvé** :
  t3code ne lie pas un message utilisateur à un tour (`turnId` nul côté
  utilisateur), donc l'ordre observé (k-ième message utilisateur ↔ k-ième tour
  assistant) n'est utilisé que si, après l'ancre, il y a exactement autant de
  tours assistants que de messages utilisateur. Un tour interrompu sans texte
  ou un tour de sous-agent casse cette égalité : la demande reste alors sans
  réponse, ce qui est préférable à lui attribuer le tour d'autrui ;
- renouvelle la session une seule fois sur 401, partagée entre la boucle et les
  liens par une génération, puis pose un verrou définitif si la session neuve
  est refusée à son tour : sans ce verrou, chaque incident rouvrirait le droit
  de renouveler et le pont émettrait une session administrative par minute ;
- écrit l'attente de réponse **avant** le dispatch, avec des identifiants
  déterministes, pour qu'une panne entre le 200 et l'écriture ne perde pas la
  corrélation, et ne laisse jamais une session non révoquée sans trace ;
- ne rejoue pas l'historique antérieur au premier regard sur un fil.

Le service de fond est un LaunchAgent / une unité `systemd --user` écrits par
le module lui-même : le script 095 est spécialisé SSH (tunnel, clé d'hôte,
runner) et l'y greffer aurait coûté plus de couture que quarante lignes dédiées.

## Conséquences

Positives : installation et retrait en une commande ; t3code intact ; fils
visibles dans `who` sous leur titre ; réponses liées sans que l'agent du fil
dispose de Bridget ; tests sans compte grâce à un faux serveur fidèle au
contrat relevé.

Négatives et limites assumées : session administrative (t3code n'en émet pas
d'autre) ; sondage HTTP toutes les trois secondes ; une demande reste sans
réponse quand le fil ne permet plus d'apparier avec certitude (tour interrompu,
tour de sous-agent), l'ambiguïté étant journalisée ; l'agent du fil ne peut pas émettre de lui-même vers
Bridget en v1 ; un tour occupé plus de deux minutes rend la remise refusée et
nommée ; la localisation n'est pas affichée (le daemon ne la retient que
lorsque tmux l'atteste).
