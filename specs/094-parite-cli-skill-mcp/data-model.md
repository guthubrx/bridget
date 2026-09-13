# Modèle 094

Aucun nouveau schéma persistant.

- Identité : UUID et instance résolus à l'appel MCP, prouvés par inscription
  auxiliaire. Le nom n'est pas une clé d'autorisation.
- Renommage : réutiliser DisplayNameOutcome et sa révision/no-op.
- DND : échéance Unix validée, appliquée par le daemon à la seule présence
  autorisée ; durée finie, volatile au redémarrage du daemon.
- Domaine : intention optionnelle dans le fichier existant agent-domains/UUID,
  réinitialisation = suppression ; ACK mémoire et sauvegarde sont deux étapes,
  un échec de la seconde ne doit pas annoncer de persistance réussie.
- Runtime : modèle/effort déclarés, source fermée Declared. Aucune sélection.
- Observations : projections existantes ; compte agent inconnu = null avec
  disponibilité false, jamais zéro inventé ; pas de chemin ou instance exposé.
- Inventaire : table documentaire de décisions par commande, contrôlée par
  test contre répartiteur/catalogue ; ce n'est pas une autorité d'exécution.
