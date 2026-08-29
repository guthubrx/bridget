# Plan technique — Session 063

## Contexte technique

La session est fondée sur `d5a362d613191747550460e79475328f3d72f418`.
Les transports possèdent leur file, le tour actif et leurs mécanismes natifs ;
le daemon conserve les gardes de routage et d’idempotence mais ne choisit pas
le tour à interrompre.

## Architecture retenue

1. L’UI et Attach continuent à persister, router et dédupliquer le message
   humain avant toute remise.
2. Dans chaque transport, `deliver` détecte un message humain arrivé pendant
   un tour actif : Codex le pilote ; ACP et Claude interrompent le tour courant
   puis gardent le message dans leur FIFO.
3. Codex ne solde pas une livraison sur l’accusé RPC. La confirmation de
   consommation du message est corrélée au tour actif ; une absence de preuve
   réinsère le message à l’avant dans l’ordre initial.
4. Le wrapper n’émet `DeliverAcked` qu’après cet événement de remise attestée.

## Couloirs exclusifs

| Couloir | Propriétaire | Fichiers |
|---|---|---|
| Déclenchement interruption | agent transport-interrupt | `crates/bridget-transport/src/acp.rs`, `crates/bridget-transport/src/claude_stream_json.rs` |
| Pilotage et remise Codex | agent codex-remise | `crates/bridget-transport/src/codex_app_server.rs` |
| Intégration | bridget | `crates/bridget-daemon/src/wrapper.rs`, documentation et merges |

`crates/bridget-daemon/src/daemon.rs` est explicitement hors modification :
il ne détient pas l’identifiant du tour actif.

## Validation

- Vérifier le formatage et la compilation des crates touchées après intégration.
- Ne pas relancer les campagnes archivées ni constituer de jury de confort.
- Après mise en service, produire seulement l’horodatage de route réelle exigé
  par le chantier : interruption ou pilotage humain pendant un tour actif.

## Constitution

- Isolation : chaque auteur code dans son worktree et son couloir exclusif.
- Minimalisme : réutiliser les files, corrélations et annulations existantes.
- Responsabilité : ne pas conclure à partir d’une intégration Git ; nommer ce
  que l’horodatage réel atteste et ce qu’il n’atteste pas.
