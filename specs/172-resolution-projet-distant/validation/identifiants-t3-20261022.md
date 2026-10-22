# Liaison des identifiants T3 — 22 octobre 2026

Le maître Bridget exécute le correctif `172-31f42307` depuis 2026-10-22T09:36:59.188552+02:00.
Source de production : `31f42307efd5c3d490a2fdb8e2c09c49f0b0a30b`. Le numéro de paquet reste 0.4.1 ;
le build chargé distingue cette correction.

## Défaut et correction

La preuve T3 est valide pour un fil `mcp:…`, mais l'ancienne liaison exigeait un UUID.
Cette contradiction causait `permission_attestation_unavailable` avant les droits métier.
La liaison accepte maintenant les identifiants exacts UUID, `mcp:…` et `thread:mcp:…` :
1 à 2048 octets, filtre commun des contrôles Cc/Cf, connexion primaire, identité exacte,
projet attesté et absence de contradiction. Les droits et la propriété restent inchangés.

Référence lue avant correction :
/Users/moi/11.Repositories/t3code-local/apps/server/src/mcp/OrchestratorMcpService.ts
(`stableThreadId`, `sessionIdentity`) et
/Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts.
T3 utilise des identifiants opaques et revalide l'identité du tour. Bridget conserve
ses contrôles de provenance, requis pour son pont et le travail distant.

## Validation et déploiement

- RED initial : 1 PASS / 1 FAIL sur le véritable préfixe `mcp:`.
- RED Unicode Cf : 0 PASS / 1 FAIL, puis correction par le filtre commun de lineage.
- GREEN : 7 tests distincts PASS, source SHA-256 vérifiée sur le commit livré.
- Contre-revue r2 `65b3fdfb-431c-40af-a5e1-e6b00d473bc4` : APPROVE ; résultat intégral lu puis ACK.
- Une compilation finale macOS, signature adhoc vérifiée ; SHA-256 `6f53d2601ff8aac635cffd93612a4bf39c4f0c0e7bdc43e593dda6c4679ae486`.
- Source fusionnée et poussée sur `main` ; maître 3688 remplacé par 82087.
- T3, son serveur, le pont et le worker Cartae 589872 sont préservés. Worker : zéro redémarrage.
- Les deux missions Cartae conservent tâche, mission, placement, instance et compteur de reprise 0.
- Aucun changement de protocole worker : aucun paquet Linux ni redémarrage Cartae nécessaires.

Le premier préflight a refusé une tâche locale encore active. Elle a ensuite terminé
avec `cleanup_done=true`, sans intervention. L'activation a repris après constat de
zéro descendant local. Les commandes distantes déjà `released/native_ready` restent
exécutées par le même runtime worker ; `prepare_restart` conserve leurs identités.
L'attente de leur fin n'était donc pas nécessaire.

## Parcours réel et limites

Le statut MCP du maître confirme `172-31f42307` actif. Une unique lecture officielle
du catalogue a été demandée au fil opaque concerné, pour vérifier son contexte exact.
Son résultat reste attendu à ce checkpoint. Le succès de notre propre fil UUID ne le remplace pas.
Aucune suite complète et aucun test fournisseur ne sont déduits des tests de bibliothèque.

L'ancienne tâche façades `837972ad-ad58-4512-9a71-fd2843ab5be5` reste `failed`,
`cleanup_done=true`, `guard_released=true`, avec son worktree propre conservé.
Elle appartient à chargement ; un autre propriétaire ne gagne pas son accès.
Aucune tâche n'a été recréée.

Preuves privées : /home/moi/.cache/bridget-identifiants172 et
/Users/moi/.cache/bridget-identifiants172. Sauvegarde précédente conservée :
/Users/moi/.cache/bridget-identifiants172/bridget-before.
