# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/plan.md](../../../../../../specs/166-rattachement-bridget-reprise/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Projet modifié : T3 Code.
- Langage : TypeScript.
- Périmètre principal : `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`.
- Test de régression : `apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`.
- Dépendance nouvelle : aucune.
- Persistance nouvelle : aucune.
- Interface publique nouvelle : aucune.
- Modification Bridget : documentation de livraison seulement.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

- Aucun renouvellement périodique n’est ajouté. Le contrôle au début du tour couvre le besoin avec une portée plus petite.
- Aucun redémarrage automatique de T3 ou de Bridget n’est ajouté.
- Aucune identité de secours fondée sur le processus, le dossier ou le fil voisin n’est acceptée.

<a id="extrait-4"></a>

## Extrait 4

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
