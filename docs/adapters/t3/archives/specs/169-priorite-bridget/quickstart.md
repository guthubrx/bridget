# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/quickstart.md](../../../../../../specs/169-priorite-bridget/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les nouvelles délégations vont à Bridget lorsque `preferBridgetDelegation=true`. Bridget choisit la machine, le moteur, la connexion, le modèle et l’effort que vous ne fixez pas. Les tâches T3 déjà lancées restent suivies sur leur moteur d'origine. Un refus Bridget ne déclenche pas de fallback silencieux vers T3.

<a id="extrait-2"></a>

## Extrait 2

Le paquet T3 signé `0.0.45-local.169` a été construit une seule fois, installé puis activé une seule fois. ASAR SHA-256 : `a23d41088ed6aa3671a129778efc42736089f68b433d7f4dd946bc25888fb52c`. Les preuves opératoires privées sont dans /home/moi/.local/state/bridget-delivery169/operational169/operational169-proofs.json. Les nouveaux processus T3 sont main4717/serveur4792. Bridget71927/72468 et le tunnel60206 sont conservés.

<a id="extrait-3"></a>

## Extrait 3

Depuis le worktree T3 confirmé par ROOT, utiliser les bancs existants et compléter leurs cas utiles. Exemple pour la cible de développement prévue :

<a id="extrait-4"></a>

## Extrait 4

```sh
cd /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget
vp test run /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/serverSettings.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts
vp test run /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/CodexDeveloperInstructions.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/CodexMcp.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.test.ts /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/bridget/BridgetLineage149.test.ts
```

<a id="extrait-5"></a>

## Extrait 5

Faire le lint des seuls fichiers changés et les types des paquets affectés selon /Users/moi/11.Repositories/t3code-local/AGENTS.md. Ne pas lancer de contrôle global du dépôt. Conserver les sorties exactes et les limites, sans annoncer de succès global.

<a id="extrait-6"></a>

## Extrait 6

Avec ON, Codex et Claude demandent l'outil automatique Bridget s'il est réellement exposé. ACP, OpenCode, Cursor et Pi annoncent son absence de montage. Les droits existants restent applicables et un refus ne lance pas T3 en secours. OFF restitue les consignes habituelles. Les tâches déjà lancées gardent leur moteur.

<a id="extrait-7"></a>

## Extrait 7

Pour désactiver, mettre `preferBridgetDelegation=false` dans /Users/moi/.t3/userdata/settings.json en conservant toutes les autres valeurs. Il n'y a pas de fallback silencieux en cas de refus Bridget.

<a id="extrait-8"></a>

## Extrait 8

J1 est clos : P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est documenté : source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` et documents Bridget `f4cb623aed57ce251bbd0b1f293f9b0c2b6f2c11` sont intégrés/poussés ; le C2 Git reste à ROOT. J3–J5 sont clos sur les preuves privées et publiables ; J6 reste ouvert. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md.
