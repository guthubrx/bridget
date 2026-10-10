# Contrat d'identité148

Outil T3 bridget_session : arguments fermés vides ; caller MCP authentifié de fil
requis. Réponse version1, environmentId, threadId, providerSessionId et
providerInstanceId. Session active du même fournisseur, fil non supprimé/archivé.
Pas de secrets ni de chemins de credential dans la réponse.

Environnement du montage stdio Bridget : BRIDGET_T3_MCP_ENDPOINT et
BRIDGET_T3_MCP_AUTHORIZATION. Une paire partielle ou invalide échoue. Endpoint
doit correspondre au runtime T3 local réel. Vérification à chaque opération.
Une preuve révoquée n'autorise aucun fallback PID ou identité globale.

Après preuve, le fil doit avoir un rattachement Bridget vivant et un credential
auxiliaire correspondant. Le daemon valide ce credential avant que le résolveur
ne rende l'identité, y compris pour un outil de diagnostic. Identifiant opaque
de l'agent et instance sont résolus ensemble ; un renommage ne change pas l'adresse.
Les identifiants bruts fournis par l'agent ne remplacent jamais cette preuve.

Les identifiants de fils T3 sont opaques : UUID ordinaire ou identifiant d'enfant
créé par son orchestrateur. Seule la réponse authentifiée en fournit la valeur.
Les réponses réseau sont bornées à 64 Kio, sans redirection. Chaque échange
HTTP dispose de trois secondes. Le credential n'est pas mis en cache côté Bridget.
Le client capture `Mcp-Session-Id` à l'initialisation et le renvoie pour la
notification et l'introspection. Il termine la session de transport par DELETE
après chaque attestation, y compris après une réponse métier invalide. Un refus
de nettoyage après une attestation réussie ferme aussi l'appel. La révocation
du credential est donc vérifiée à chaque opération, sans cache de preuve.
