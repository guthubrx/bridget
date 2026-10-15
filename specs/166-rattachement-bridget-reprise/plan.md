# Plan d’implémentation — SPEC-166 Rattachement Bridget après reprise T3

## Résumé

Un fil Codex natif peut rester chargé avec une preuve Bridget A après la rotation de la session privée vers une preuve B. Le début du tour détecte cette différence, mais il retire seulement les droits publiés. Il ne recharge pas B dans le fil natif. Le premier appel Bridget du tour échoue alors avec `t3_session_unavailable`.

La correction réutilise la reprise native `thread/resume`. Juste avant `turn/start`, l’adaptateur compare la preuve montée avec la preuve courante. Si elles diffèrent, il reprend le même fil avec sa configuration courante. Il publie ensuite les droits du tour et le démarre. Si elles sont identiques, le chemin actuel reste inchangé.

## Contexte technique

- Projet modifié : T3 Code.
- Langage : TypeScript.
- Périmètre principal : `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`.
- Test de régression : `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`.
- Dépendance nouvelle : aucune.
- Persistance nouvelle : aucune.
- Interface publique nouvelle : aucune.
- Modification Bridget : documentation de livraison seulement.

## Cause établie

`threadRuntimeParams()` monte la preuve courante lors d’un démarrage, d’une reprise ou d’une bifurcation. Il mémorise cette preuve dans `bridgetCredentialsByThread`.

`startNativeTurn()` relit ensuite la preuve courante. Quand la preuve mémorisée diffère, il invalide les droits. Il lance toutefois le tour sans reprendre le fil natif. Le processus fournisseur conserve donc l’ancienne configuration MCP.

Le défaut apparaît après une rotation sur une session fournisseur partagée. La réinstallation du pont recrée une preuve, mais elle ne garantit pas son montage dans un fil natif déjà chargé.

## Constitution Check

| Règle | Décision |
|---|---|
| Processus SpecKit | Session 166 isolée avec spec, plan, tâches, analyse et reçu. |
| Worktrees dédiés | Artefacts Bridget et source T3 isolés sur des branches de session. |
| Diagnostic avant modification | Échec réel, état SQLite et flux de reprise observés avant le code. |
| Réutiliser avant de créer | Réutiliser `threadRuntimeParams`, `thread/resume` et la carte de preuves. |
| Minimalisme | Deux fichiers T3 prévus. Aucun service ni stockage. |
| Refus sûr | Aucun repli vers une autre identité. Un échec de reprise bloque le tour. |
| Tests | Oracle RED de rotation, puis régressions démarrage, reprise, bifurcation et tours. |
| Livraison | Build signé, sauvegarde, installation et recette réelle avant clôture. |

Aucune violation constitutionnelle n’est nécessaire.

## Architecture retenue

### 1. Vérifier la preuve au dernier moment

Le début d’un tour relit `McpProviderSession`. Il compare l’objet courant avec celui qui a produit la configuration du fil natif.

### 2. Remonter uniquement en cas de différence

Si la preuve est absente ou identique, aucun appel natif supplémentaire n’est ajouté. Si une preuve courante existe et diffère, l’adaptateur envoie un seul `thread/resume` sur le même identifiant natif. Il utilise `excludeTurns: true` et `threadRuntimeParams()`.

Cette reprise conserve l’historique. Elle remplace uniquement la configuration du fil natif.

### 3. Publier les droits après le remontage

La publication `provider_turn` reste liée à la preuve effectivement montée. Une preuve ancienne ne reçoit aucun fait de droits. Un échec de reprise empêche `turn/start`.

### 4. Garder la reprise publique inchangée

Le chemin public `resumeThread()` et le nouveau contrôle utilisent un petit helper interne commun. Cela évite deux implémentations de `thread/resume`.

## Réutilisation de l’existant

- `threadRuntimeParams()` prépare déjà la configuration privée par fil.
- `bridgetCredentialsByThread` indique déjà la preuve montée dans chaque fil natif.
- `McpProviderSession.readMcpProviderSession()` fournit la preuve courante.
- `resumeThread()` définit déjà la requête native, la compatibilité avec les fils archivés et le décodage.
- `publishMcpProviderPermissions()` conserve l’attestation des droits par tour.
- Le banc de rejeu Codex contrôle déjà l’ordre exact des requêtes natives.

## Divergences volontaires

- Aucun renouvellement périodique n’est ajouté. Le contrôle au début du tour couvre le besoin avec une portée plus petite.
- Aucun redémarrage automatique de T3 ou de Bridget n’est ajouté.
- Aucune identité de secours fondée sur le processus, le dossier ou le fil voisin n’est acceptée.

## Fichiers prévus

| Fichier | Changement |
|---|---|
| `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts` | Reprendre le fil natif si la preuve montée diffère de la preuve courante. |
| `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts` | Reproduire une rotation entre le chargement du fil et le premier tour. |
| `docs/decisions/166-rattachement-bridget-reprise.md` | Historiser l’invariant et le refus sûr. |
| `specs/166-rattachement-bridget-reprise/*` | Conserver le plan, les preuves et le reçu. |

## Phases d’implémentation

1. Figer le diagnostic et l’audit de réutilisation.
2. Ajouter un rejeu RED : preuve A montée, rotation vers B, puis début de tour.
3. Extraire le helper interne de reprise native.
4. Remonter B avant `turn/start` seulement si A et B diffèrent.
5. Prouver que le chemin stable n’ajoute aucune requête.
6. Exécuter les tests ciblés, le typecheck et les contrôles proportionnés.
7. Intégrer et pousser les sources validées.
8. Construire, signer, sauvegarder, installer et activer T3.
9. Reprendre un fil réel et vérifier Bridget sans réinstaller le pont.
10. Finaliser les reçus et nettoyer les worktrees.

## Stratégie de test

- RED déterministe : la transcription attend `thread/resume` avec B avant `turn/start`.
- Identité : la configuration envoyée contient B et jamais A après la rotation.
- Droits : B reçoit le fait `provider_turn` du tour actif.
- Stabilité : une preuve inchangée ne produit pas de reprise supplémentaire.
- Reprise : le chemin public conserve son résultat et la gestion des fils archivés.
- Partage : deux fils d’une session fournisseur gardent leurs preuves distinctes.
- Régression : démarrage, reprise, bifurcation, interruption et continuation restent valides.
- Qualité : format, tests ciblés, typecheck et suite adaptée au risque.
- Runtime : le premier appel Bridget d’un fil repris réussit sans réinstallation du pont.

## Observabilité et preuve

La recette conserve les éléments suivants :

- versions source et application ;
- PID et date de démarrage avant et après activation ;
- résultat du test RED, puis du même test après correction ;
- résultats des contrôles ciblés ;
- statut Bridget avant et après la reprise réelle ;
- absence de `auth_failed`, de HTTP 401 et de `t3_session_unavailable` sur le scénario validé.

## Risques et protections

| Risque | Protection |
|---|---|
| Reprendre chaque tour | Comparaison par identité avant tout appel. |
| Monter la preuve d’un autre fil | Lecture et carte indexées par `threadId`. |
| Perdre l’historique | Même identifiant natif et `excludeTurns: true`. |
| Démarrer avec une preuve non montée | `turn/start` attend la fin de `thread/resume`. |
| Élargir les droits | Publication inchangée et liée au run courant. |
| Masquer l’échec | Une erreur de reprise remonte comme erreur de démarrage du tour. |
| Régression des fils archivés | Réutiliser le traitement `thread/unarchive` existant. |

## Charge cognitive future

Le changement ajoute une seule règle : la preuve publiée doit être la preuve montée. Il centralise la requête native déjà présente et n’ajoute aucun nouvel état durable.
