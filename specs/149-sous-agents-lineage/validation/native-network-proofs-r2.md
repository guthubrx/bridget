# Preuves natives r2 - T036 et T039 (récapitulatif)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde r2 après le correctif F1 (reçu natif r5). Aucun commit, aucune case cochée, aucune écriture de production, aucun redémarrage, aucun modèle.
Archives r1 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r1/`.

## Verdict global : PARTIAL

| Tâche | Contrôles | Verdict | Rapport |
|---|---|---|---|
| T036 interop réseau | 43 + 15 + 7 = 65/65 PASS | **APPROVE** sur le périmètre exécuté | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| T039 scénarios dégradés | 35/37 PASS, 2 FAIL (diagnostic de F2) | **PARTIAL** : F1 corrigé, nouvel écart F2 à trancher | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` |

## Binaire testé et sources

- Binaire IMMUABLE r5 : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-ec6b18b5d468`, SHA-256 `ec6b18b5d468dd0031eb6282da10bdc27cb38b7193f07b4a8b9107cbe92d8b0f` recalculé, égal au reçu `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt-r5.json`. Chaque recette refuse de démarrer si l'empreinte diffère.
- Fichiers de production r5 (`daemon.rs`, `native_delegation.rs`, `wrapper.rs` et les autres fichiers du reçu) : empreintes identiques à celles d'aujourd'hui.
- Dérive FUTURE (pas un défaut du binaire) : `Cargo.toml` (`sha2` + `asm`), `Cargo.lock`, et un fichier de test ajouté. L'empreinte de production d'aujourd'hui est celle du reçu canonique r6 (binaire `620e729fca53`, non rejoué ici). Autrement dit, r6 = r5 + manifestes.
- Sources T3 : empreinte des 32 fichiers de production `26fba25527ad0247` et `git diff HEAD` `59a54267ce7c1d2a`, tous deux inchangés depuis le rapport T3 r5. Détail : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/fingerprints.json`.

## Ce qui a changé depuis r1

1. **F1 corrigé** (R9.2.a à e : 5/5 PASS) : 1 tour, 1 PID, lancements 12 → 12, `failed: unreachable` dès le démarrage, 1 notice d'échec, rejeu sans relance. En r1 : 2 tours, 2 PID, 12 → 13.
2. **Réserve « deux instances T3 » levée** : `unified149.ts` (7/7 PASS) prouve qu'UN seul serveur `bin.ts` émet le credential MCP (vraie session Codex de T3, vrai adaptateur, vrai fait de permissions) ET sert Lineage ; le cancel arrête les PID ; l'interruption du run par T3 ferme le credential.
3. **Nouvel écart F2 (moyen)** : après SIGTERM + relance, une racine qui attendait un descendant reste `waiting_for_children` sans fin, sans notice, avec un résultat retenu jamais remis. Cause prouvée : l'exécution du descendant reste `running` alors que sa tâche est `failed`. Seul `cancel` sort de l'état. Décision requise (options A/B dans `recovery149.md`).
4. **Observations** : O4 (le fournisseur en vol survit à l'arrêt du daemon, mesuré : `ppid 1`, 6,9 s, groupe de processus propre), O1 (sens exact de G-P-07(b), proposition de clarification), O6 (`queued` non exposable en réel), O7 (lancement pendant l'arrêt).
5. **Incident de recette déclaré** : un premier essai a lancé le VRAI `codex` de l'utilisateur dans un HOME isolé ; sa requête vers `api.openai.com` a reçu 401 (aucune authentification, aucun modèle, texte de recette sans secret). Corrigé par un chemin absolu dans `settings.json` de la fixture. Détail dans `interop149.md`.

## Ce qui est réellement exécuté

- Daemon Bridget 149 réel (r5), `bridget mcp` réel, CLI `bridget lineage` réelle, SIGTERM réels du daemon (PID individuels vérifiés, jamais `-9`).
- Hôte MCP de T3 réel (`interop149`, `recovery149`) et serveur T3 complet `bin.ts` (`server149`, `unified149`).
- Enfants : faux serveur app-server Codex fermé. Pas de `bridget_fixture.mjs`, pas de faux Orchestrator HTTP, aucun modèle.
- ACK : `delivery_generation` (u64, 19 chiffres) lu et renvoyé en entier exact, jamais en `Number`.

## Simulé, nommé

Hôte MCP de `interop149`/`recovery149` : projection de conversation en mémoire et fait publié par la recette ; faux pair Codex de T3 dans `unified149` (réponses capturées, tour tenu ouvert) ; wrapper du fil T3 côté daemon (trames réelles) ; enfants fermés ; R9.4.c et R9.4.d mutent la base PRIVÉE de la fixture pour un diagnostic.

## Lacunes concrètes avant de cocher

1. Décision du principal sur F2 (option A recommandée), puis rejeu de `recovery149.ts` sur le binaire qui la contient. Aucun code de production n'a été touché par moi.
2. Revue finale du texte G-P-07(b) (clarification proposée, O1).
3. `queued` non engagé après SIGTERM : seulement couvert par le test unitaire (O6).
4. Orphelin après arrêt du daemon (O4) : à évaluer sous launchd (non vérifié ici).
5. Rejeu éventuel sur le binaire r6 (non fait : consigne r5 immuable).
6. Hors périmètre ici : modèle réel (T037), voie standalone Claude/GLM et observer PTY (T038), `can_use_tool` (G-P-01), rendu navigateur et coque desktop.

## Fichiers

- Recettes et résultats : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (voir `README.md`) ; résultats bruts `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/*.json`.
- Rapports : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md`.
