# Journal d’implémentation — Session 063

**Branche** : `session-063-interruption-pilotage-tour-humain`  
**Démarré** : 2026-08-29

## État initial attesté

- La trame Claude est déjà intégrée dans la base ; aucune réimplémentation.
- `turn/steer` Codex est déjà intégré, mais l’accusé de remise n’est pas encore
  corrélé à une consommation attestée.
- Le déclenchement doit vivre dans les transports, non dans le daemon.

## Preuve finale attendue

Un horodatage de route réelle montrant qu’un message humain pilote ou
interrompt un tour actif après mise en service.

## Remise Codex bornée - 2026-08-29

- `turn/steer` reste non conclusif : seul `item/completed` de type
  `userMessage`, corrélé au `threadId`, au `turnId` actif et à l'identifiant du
  message, produit `PromptDispatched`.
- L'échéance d'un message humain non attesté déclenche le repli
  `turn/interrupt`. Les messages non attestés repartent en tête de FIFO avant
  les messages système déjà en attente. L'appel de steering et l'interruption
  sont bornés à trois secondes ; après dix secondes sans terminal d'interruption,
  le tour ancien devient un refus structuré au lieu de retenir la file.
- Le wrapper existant acquitte durablement seulement `PromptDispatched`. Une
  redélivrance d'un reçu `Seen` sans preuve est classée `DeliveryIndeterminate`
  et ne réinjecte pas un second prompt.

### Preuves isolées

- `TEMOIN_G_codex_steer_sans_consommation_interrompt_et_priorise_l_humain` :
  tour éternel, accusé de steering sans consommation, autorisation en attente,
  interruption, puis humain avant système.
- `TEMOIN_H_codex_steer_ack_sans_consommation_ne_solde_pas_la_remise` : un
  résultat JSON-RPC seul ne produit pas d'acquittement.
- `TEMOIN_I_codex_user_message_corrige_acquitte_la_remise_humaine` : la preuve
  corrélée produit l'acquittement.
- Les trente-cinq tests `codex_app_server` et les deux tests wrapper de reçus
  idempotents passent dans le worktree isolé.

### Limites nommées

- Le schéma généré par `codex app-server` expose `clientUserMessageId` et un
  `item/completed userMessage.id`, mais ne déclare pas formellement leur égalité.
  Le code reste conservateur : sans égalité exacte, il n'acquitte pas.
- Aucun démon, relais ni agent actif n'a été redémarré. La route réelle et son
  horodatage restent à produire après intégration contrôlée.
- `cargo test -p bridget-daemon --test codex_native_test` échoue déjà sur
  `main` inchangé et sur le lot de départ : annuaire vide puis `AgentUnknown`.
  Ces deux échecs préexistants ne sont ni corrigés ni masqués par cette session.
