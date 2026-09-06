# Données 090

Pas de migration de base.

- Session interactive : instance Bridget existante, nom relisible, fil fournisseur
  attesté, socket Unix possédée, PID/naissance serveur, PID TUI. Durée = invocation.
- Tour : fil/turnId source, origine humaine externe ou message Bridget existant,
  terminal fournisseur ; jamais deux marqueurs de remise pour le même message.
- Autorité de permission : native/user en interactif, réglage historique pour les
  sessions gérées. La réception d'une requête n'est pas une approbation.
- Lien daemon : états existants connecté/reconnexion/terminé ; sa perte ne change
  pas le fil fournisseur. Échec fournisseur n'autorise pas une reconstruction cachée.
  `TerminalSessionReady` atteste le propriétaire terminal du cycle de vie sur
  cette connexion seulement ; purge à la fermeture, réannonce à la reconnexion.
  Le mode cli et le journal ne permettent pas de déduire ce fait.
- Octets : frames WS texte → ligne JSONL sans ré-sérialisation. Les payloads du
  journal continuent d'être produits par le pilote existant avec raw/provenance.
