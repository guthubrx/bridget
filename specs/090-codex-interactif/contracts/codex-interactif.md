# Contrat 090

Surface : `bridget codex [--name NOM] [options natives prises en charge]`.
Terminal stdin/stdout interactifs requis. `--equipier` garde son comportement.
Les paramètres de transport remote/listen sont possédés par Bridget, pas
redéfinissables via les arguments ; les paramètres refusés sont nommés à l'erreur.

Séquence : app-server Unix privé → initialize (experimentalApi) / initialized → thread/start (historyMode=legacy) →
thread/name/set → register/journal Bridget → TUI native resume du même threadId.
Les preuves réelles de conception établissent la nécessité de name/set sur 0.153.4.

Un seul client (TUI) répond aux approbations et élicitations. Le client Bridget
les observe, ne les refuse ni ne les accepte automatiquement. Un terminal réel
seul libère un tour externe. Envoi idempotent existant : la remise confirmée
provient de son message corrélé, jamais du seul état actif du fournisseur.

Fermeture TUI/terminal : arrêt du fournisseur de cette session, retrait de présence,
nettoyage de la socket possédée. Aucun arrêt d'une autre session Codex ni du daemon.
Une reconnexion au daemon ne démarre aucun nouveau thread/start.
Le wrapper annonce `TerminalSessionReady` après le lancement natif et le réémet
après Register lors d'une reconnexion. Le daemon garde ce fait par connexion,
le purge à sa fermeture et ne déduit jamais ce statut de Cli/journal/protocole.
Un client auxiliaire ou non enregistré ne peut pas le déclarer. Le fait préserve
la TUI lors d'un shutdown daemon ; un Disconnect explicite continue d'arrêter.

## Frontières vérifiables

- ACK : `item/completed` natif `userMessage`, `clientId` = id Bridget ET
  threadId/turnId = tour observé ; une seule émission. Ni entrée en file ni
  `turn/start` réussi ne valent consommation attestée.
- Une saisie humaine ajoutée pendant le tour ne vole pas sa corrélation Bridget.
  Le premier input est `turn_start` ; les suivants sont `user_message` (événement
  journal v1 additif, from/body et input natif conservés, rendu par attach).
- Le texte écran ne crée aucune réponse interagent implicite. La réponse liée
  passe par MCP et les transactions idempotentes existantes.
- Serveur neuf, un seul fil chargé toléré. Tout `thread/started` OU
  `thread/status/changed` étranger arrête la session, y compris sous-agent interne.
  Le second signal couvre la reprise d'un historique : Codex 0.153.4 ne publie
  pas `thread/started` sur cette voie. Pas de suivi silencieux d'un ancien fil.
- Le parent possède TUI et groupe app-server : fermeture TUI d'abord (libère
  l'élicitation native), puis arrêt borné TERM/INT et suppression socket après
  disparition confirmée seulement. Survivant = erreur explicite, pas faux succès.
- La socket et son parent sont privés au compte, longueur Unix bornée, chemin
  existant y compris symlink pendant refusé AVANT lancement. Pas de port TCP.
- Options natives du modèle/profil/config/permissions relayées explicitement aux
  deux côtés. Options remote/last/all et sous-commandes interdites ; cd/add-dir/
  image/oss refusées clairement. Pas de compatibilité universelle prétendue.
