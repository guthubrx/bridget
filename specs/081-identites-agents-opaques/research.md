# Recherche - SPEC-081

## Décisions

### agent_id opaque, pas secret

Décision : UUID stable pour joindre les données, sans en faire une autorisation.

L'OWASP Authorization Cheat Sheet recommande le moindre privilège et le refus par défaut :
https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html

Conséquence : les contrôles d'accès aux sockets, UI et Maicie restent obligatoires.

### Pas d'alias de compatibilité permanent

Décision : migration unique puis refus de name.

Conserver deux clés maintiendrait la confusion que le produit doit supprimer. Le projet est en pré-production et accepte redémarrage et recréation d'agents.

### Maicie marque sans deviner

Décision : une référence non résolue devient requires_retarget.

L'autorisation MCP sépare identité, transport et autorisation :
https://modelcontextprotocol.io/specification/2025-03-26/basic/authorization

Une cible introuvable ne justifie pas une redirection vers un nouvel agent homonyme.

### Réemploi du profil

Décision : étendre AgentProfileStore, sans nouveau registre.

Il contient déjà UUID, display_name, labels, avatar, instructions et préférences de notification. La migration réduit le modèle plutôt qu'elle n'ajoute une abstraction.
