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
