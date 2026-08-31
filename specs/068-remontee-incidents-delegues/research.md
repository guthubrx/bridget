# Recherche - SPEC-068

## Décision 1 - Séparer avertissement et échec terminal

**Décision**: conserver deux catégories fermées de faits: `warning` pour un
incident d'outil récupérable et `failed` pour une exécution enfant terminée en
erreur.

**Rationale**: un refus d'outil laisse l'agent libre d'adopter un repli. Il ne
prouve pas l'échec de sa délégation. En revanche, le terminal fournisseur
`Failed` est déjà une preuve d'échec d'exécution dans Bridget. La
spécification MCP distingue aussi les erreurs de protocole et les erreurs
d'exécution récupérables, ce qui confirme que ces états ne doivent pas être
aplatis dans un unique « échec ».

**Sources**:
- https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- https://modelcontextprotocol.io/specification/2025-06-18/schema

**Alternatives écartées**:
- Un message libre « l'enfant a échoué »: non corrélable et faux pour un
  avertissement récupérable.
- Déduire le statut depuis la présence ou l'inactivité: ce ne sont pas des
  terminaux d'exécution.

**Impact mainteneur**: deux mots fermés rendent la lecture et les tests
mécaniques. Aucun texte fournisseur ne devient une règle métier.

## Décision 2 - Étendre le registre de liens, pas le guichet Maicie

**Décision**: le nouveau fait appartient au même stockage runtime que les
`agent_links`, mais dans une collection dédiée aux incidents runtime plutôt
que dans les transitions de cycle de vie existantes.

**Rationale**: `agent_link_events` contient aujourd'hui seulement une
transition d'état (`reserved`, `open`, `transferred`, `closed`,
`orphaned`). Lui faire porter un code, une référence et un accusé mélangerait
deux sémantiques. Une collection dédiée reste limitée au lien concerné et ne
crée ni backend d'observabilité ni dépendance.

**Preuves dans l'existant**:
- `crates/bridget-daemon/src/idempotency.rs`: `AgentLinkEvent` est
  exclusivement un changement de propriété.
- `crates/bridget-daemon/src/fleet.rs`: la flotte sait déjà trouver le lien
  de l'enfant et réveiller les lecteurs après un fait durable.
- `crates/bridget-daemon/src/store.rs`: le projet possède le précédent d'un
  flux cursé, durable et accusé, sans déduire une transition métier.

**Impact mainteneur**: une table étroite, indexée par parent et curseur, peut
être inspectée et supprimée indépendamment sans fragiliser les liens.

## Décision 3 - Réutiliser le protocole daemon-wrapper et accuser après injection

**Décision**: le daemon remet un fait typé au wrapper du parent. Le wrapper
fabrique une notification système non intrusive et accuse seulement après que
le transport du parent a effectivement accepté son injection. En cas de refus
ou de déconnexion, le fait reste non accusé pour une reprise.

**Rationale**: un coordinateur ne doit pas être interrompu par l'incident de
son enfant. Les adaptateurs Claude et ACP refusent explicitement le pilotage
`SteerCurrent`; la notification doit donc utiliser la file normale et ne pas
simuler une interruption. L'accusé est une preuve de remise technique au
wrapper, pas une réponse de mission.

**Preuves dans l'existant**:
- `crates/bridget-daemon/src/wrapper.rs`: les messages daemon-wrapper sont
  déjà traduits vers le transport géré.
- `crates/bridget-transport/src/codex_app_server.rs`,
  `crates/bridget-transport/src/acp.rs` et
  `crates/bridget-transport/src/claude_stream_json.rs`: `SteerCurrent`
  n'est pas portable.
- `crates/bridget-daemon/src/daemon.rs`: les remises idempotentes possèdent
  déjà la distinction entre injection et issue fournisseur.

**Alternative écartée**: interrompre le parent ou créer une commande spécifique
par fournisseur. Cela étendrait la SPEC-063/064 et dégraderait le travail du
coordinateur.

**Impact mainteneur**: la remise respecte les files existantes. La garantie
reste explicite: le parent reçoit la notification à sa prochaine frontière
fournisseur, sans promesse de réponse métier.

## Décision 4 - Ne projeter que code et référence redacted

**Décision**: persister et remettre un code fermé et une référence
pseudonymisée. Le détail brut des adaptateurs ne sort jamais du journal local.

**Rationale**: le rejet Codex récent fournit déjà le modèle
`unsupported_provider_request` avec une référence. Un détail d'outil peut
contenir chemins, arguments, contenu utilisateur ou secret. La réponse
JSON-RPC reste cohérente avec la recommandation MCP d'avoir un code et un
message courts.

**Preuves dans l'existant**:
- `crates/bridget-transport/src/codex_app_server.rs`:
  `unsupported_provider_request_response`.
- `crates/bridget-daemon/src/attach.rs`:
  `provider_request_rejected_summary`.

**Impact mainteneur**: le coordinateur a une clé de diagnostic sans que les
données sensibles se propagent entre sessions.

## Décision 5 - Aucune autorité Maicie

**Décision**: le fait est un constat Bridget. Il n'appelle aucun contrat
`ServiceHello`, guichet ou transition de délégation Maicie.

**Rationale**: la séparation d'autorité est une contrainte de la SPEC-064 et
de l'ADR 015. Le coordinateur humain ou Maicie peut ultérieurement décider de
relancer, remplacer ou clore la délégation, mais ce choix reste externe au
runtime.

**Impact mainteneur**: une recherche de `Guichet` ou `ServiceHello` dans le
diff final doit rester vide hors tests de non-régression.
