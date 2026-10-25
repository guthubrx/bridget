# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La preuve T3 est valide pour un fil `mcp:…`, mais l'ancienne liaison exigeait un UUID.
Cette contradiction causait `permission_attestation_unavailable` avant les droits métier.
La liaison accepte maintenant les identifiants exacts UUID, `mcp:…` et `thread:mcp:…` :
1 à 2048 octets, filtre commun des contrôles Cc/Cf, connexion primaire, identité exacte,
projet attesté et absence de contradiction. Les droits et la propriété restent inchangés.

<a id="extrait-2"></a>

## Extrait 2

Référence lue avant correction :
/Users/moi/11.Repositories/t3code-local/apps/server/src/mcp/OrchestratorMcpService.ts
(`stableThreadId`, `sessionIdentity`) et
/Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts.
T3 utilise des identifiants opaques et revalide l'identité du tour. Bridget conserve
ses contrôles de provenance, requis pour son pont et le travail distant.

<a id="extrait-3"></a>

## Extrait 3

- RED initial : 1 PASS / 1 FAIL sur le véritable préfixe `mcp:`.
- RED Unicode Cf : 0 PASS / 1 FAIL, puis correction par le filtre commun de lineage.
- GREEN : 7 tests distincts PASS, source SHA-256 vérifiée sur le commit livré.
- Contre-revue r2 `65b3fdfb-431c-40af-a5e1-e6b00d473bc4` : APPROVE ; résultat intégral lu puis ACK.
- Une compilation finale macOS, signature adhoc vérifiée ; SHA-256 `6f53d2601ff8aac635cffd93612a4bf39c4f0c0e7bdc43e593dda6c4679ae486`.
- Source fusionnée et poussée sur `main` ; maître 3688 remplacé par 82087.
- T3, son serveur, le pont et le worker Cartae 589872 sont préservés. Worker : zéro redémarrage.
- Les deux missions Cartae conservent tâche, mission, placement, instance et compteur de reprise 0.
- Aucun changement de protocole worker : aucun paquet Linux ni redémarrage Cartae nécessaires.

<a id="extrait-4"></a>

## Extrait 4

Le statut MCP du maître confirme `172-31f42307` actif. La lecture officielle unique
`bridget_capabilities({})` réussit depuis le véritable fil opaque
`mcp:f5a66bd1-7cef-4d2c-888b-06dd7d8263d3` le 22 octobre à 09:38:18 CEST.
Preuve T3 : item 3469, confirmation 3475 ; rapport Bridget `mcp-33631-6ad9bd74-6`.
Le catalogue rend la portée projet et le bon worktree, sans
`permission_attestation_unavailable`. Le défaut d'accès de ce contexte est corrigé.
La recette ne lance aucune mission ; elle ne qualifie pas l'authentification fournisseur
ni une nouvelle exécution distante. L'ancienne tâche d'un autre propriétaire n'est pas relue.
Aucune suite complète et aucun test fournisseur ne sont déduits des tests de bibliothèque.
