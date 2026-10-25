# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/contracts/session.md](../../../../../../../specs/148-identite-delegation/contracts/session.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Outil T3 bridget_session : arguments fermés vides ; caller MCP authentifié de fil
requis. Réponse version1, environmentId, threadId, providerSessionId et
providerInstanceId. Session active du même fournisseur, fil non supprimé/archivé.
Pas de secrets ni de chemins de credential dans la réponse.

<a id="extrait-2"></a>

## Extrait 2

Environnement du montage stdio Bridget : BRIDGET_T3_MCP_ENDPOINT et
BRIDGET_T3_MCP_AUTHORIZATION. Une paire partielle ou invalide échoue. Endpoint
doit correspondre au runtime T3 local réel. Vérification à chaque opération.
Une preuve révoquée n'autorise aucun fallback PID ou identité globale.

<a id="extrait-3"></a>

## Extrait 3

Les identifiants de fils T3 sont opaques : UUID ordinaire ou identifiant d'enfant
créé par son orchestrateur. Seule la réponse authentifiée en fournit la valeur.
Les réponses réseau sont bornées à 64 Kio, sans redirection. Chaque échange
HTTP dispose de trois secondes. Le credential n'est pas mis en cache côté Bridget.
Le client capture `Mcp-Session-Id` à l'initialisation et le renvoie pour la
notification et l'introspection. Il termine la session de transport par DELETE
après chaque attestation, y compris après une réponse métier invalide. Un refus
de nettoyage après une attestation réussie ferme aussi l'appel. La révocation
du credential est donc vérifiée à chaque opération, sans cache de preuve.
