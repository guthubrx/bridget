# Plan 068 - Remonter les incidents des délégations

## Contexte technique

Le dépôt est un workspace Rust. Les liens parent-enfant sont déjà persistés
dans `IdempotencyStore` et projetés sur le protocole daemon-wrapper. Le
wrapper ignore actuellement `ManagedEventKind::Error`; les terminaux
`ManagedTerminal::Failed` ne remontent qu'à l'exécution locale.

La synchronisation SpecKit a été tentée mais `specify` n'est pas installé sur
le serveur et le dépôt ne contient pas `.specify/`. Les artefacts sont donc
produits avec les conventions de SPEC déjà présentes dans `specs/`.

## Constitution check

- Réutilisation: le registre de liens, la flotte, le protocole daemon-wrapper
  et les journaux existants sont étendus.
- Minimalisme: une collection runtime étroite est justifiée parce que les
  transitions de cycle de vie ne peuvent pas représenter un code, une référence
  et un accusé sans perdre leur sens.
- Autorité: aucun appel Maicie, guichet, objectif ou délégation métier.
- Sécurité: aucun détail brut fournisseur, code ou argument d'outil dans la
  persistance ou la projection parent.
- Complexité: insertion O(1), reprise indexée O(k) pour les k faits non
  accusés du parent, sans balayage de flotte.

## Architecture retenue

1. Ajouter au stockage idempotent un flux durable de faits runtime délégués,
   indexé par parent et curseur, avec accusé technique.
2. Étendre le protocole interne daemon-wrapper avec:
   - soumission d'un fait par l'enfant;
   - remise d'un fait non accusé au parent;
   - accusé de remise par le parent.
3. Dans le wrapper enfant, convertir:
   - un diagnostic fournisseur normalisé en `warning`;
   - un terminal `Failed` corrélé en `failed`.
4. Dans le wrapper parent, convertir le fait typé en notification système à
   file normale. L'accusé est envoyé seulement après `PromptDispatched` pour
   une session gérée, ou après l'injection synchrone réussie dans un pane tmux.
5. Au registre du parent et à l'arrivée d'un fait, le daemon tente la remise
   des faits non accusés. Un refus ou une déconnexion laisse le fait rejouable.
6. Ne toucher ni au guichet Maicie ni aux adaptateurs qui ne savent pas
   distinguer un diagnostic normalisé.

## Fichiers prévus

- `crates/bridget-daemon/src/idempotency.rs`: fait durable, migration,
  sélection et accusé.
- `crates/bridget-daemon/src/fleet.rs`: façade de liens et réveil.
- `crates/bridget-transport/src/protocol.rs`: trames versionnées.
- `crates/bridget-transport/src/managed_session.rs`: diagnostic typé.
- `crates/bridget-transport/src/codex_app_server.rs`: source normalisée du
  refus d'outil connu.
- `crates/bridget-daemon/src/wrapper.rs`: publication enfant et réception
  parent.
- `crates/bridget-daemon/src/daemon.rs`: autorisation par instance, remise,
  reprise et accusé.
- `specs/068-remontee-incidents-delegues/`: preuves et journal.

## Ordre d'implémentation

1. Écrire les témoins rouges du stockage et du contrat filaire.
2. Implémenter le fait durable, la migration et la reprise non accusée.
3. Raccorder le wrapper enfant et le diagnostic Codex déjà normalisé.
4. Raccorder le daemon et le wrapper parent, puis les accusés.
5. Ajouter le terminal `failed` distinct.
6. Rejouer les tests ciblés et les scénarios de déconnexion.
7. Vérifier le diff contre les exigences Maicie et l'absence de détail brut.

## Risques et gardes

- **Double remise**: l'accusé est durable et l'événement a un identifiant
  stable.
- **Accusé frauduleux**: le daemon déduit le parent depuis la connexion et le
  lien, jamais depuis une valeur du client.
- **Confusion warning/failed**: types fermés et témoins séparés.
- **Perte lors d'une déconnexion**: aucun accusé avant `PromptDispatched` ou
  l'injection tmux réussie.
- **Fuite de contenu**: contrat sans champ de détail; les tests mutent un
  détail fournisseur pour vérifier son absence.
- **Changement métier**: tests négatifs sur les voies Maicie et guichet.

## Non-objectifs techniques

- Aucun nouveau crate, processus ou service.
- Aucune diffusion via les journaux attach ou l'UI avant que la remise au
  coordinateur ne soit attestée.
- Aucun changement des contrôles d'interruption de SPEC-063/064.
