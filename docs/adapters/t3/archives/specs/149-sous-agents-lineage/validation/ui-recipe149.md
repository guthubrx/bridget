# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/ui-recipe149.md](../../../../../../../specs/149-sous-agents-lineage/validation/ui-recipe149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Sources lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149.md` (r2), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r3.md` (r3), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` (r4, la plus récente).

<a id="extrait-2"></a>

## Extrait 2

- Verdict de la ronde r4 : APPROVE, zéro finding actif. Ce verdict vaut pour la recette UI web avec daemon fixture.
- T040 reste non coché. Le cochage revient au principal, après revue.
- Revue T041 (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md`) : le critère de T040 est rempli à son niveau cible. Ce critère est une recette UI web sur serveur isolé avec daemon fixture, journal, nested, statuts, modèle, arrêt, reconnexion et preuves visuelles. SC001 et T037 sont des preuves séparées. Elles ne rendent pas T040 incomplète.
- Preuve valide : l'interface web Lineage, le journal, l'arrêt et la reconnexion, avec un serveur T3 réel et un daemon simulé.
- Preuve non valide : la coque desktop, le mobile, iOS, le moteur natif de bout en bout, SC001, T037 et le MCP privé authentifié.

<a id="extrait-3"></a>

## Extrait 3

| Couche | État |
|---|---|
| Serveur T3 du worktree (`apps/server`, Node 24) | RÉEL |
| Auth par jeton scopé et ticket WebSocket | RÉEL |
| Projection SQL privée, `BridgetLineage`, garde lecture seule des fils virtuels | RÉEL |
| UI web du worktree (Vite+) ouverte dans la preview T3 | RÉEL |
| Daemon Bridget : `bridget_fixture.mjs` sur un magasin JSON privé | SIMULÉ |
| Tâches, journaux, résultats : jeu `[recette149]` | SYNTHÉTIQUE |
| Modèle `glm-5.3-flash` affiché | Donnée de fixture. Aucun modèle n'a tourné. |

<a id="extrait-4"></a>

## Extrait 4

Aucun modèle n'a tourné. Aucune app T3 Desktop n'a été lancée. Aucun service de production n'a été touché. Les tests de clic utilisent la preview T3, sans Playwright ni Chrome.

<a id="extrait-5"></a>

## Extrait 5

| Suite | Résultat r4 | Source |
|---|---|---|
| Tests serveur 149 (5 fichiers) | 120 PASS, 0 FAIL | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` |
| Tests UI et client ciblés (journal, ThreadRelationshipsControl, client-runtime) | 22 PASS, 0 FAIL (3 fichiers, sans le mobile). Rejoués par la revue T041 avec le journal mobile : **24 PASS, 0 FAIL, 4 fichiers**. | idem et `review-final149.md` |
| `BridgetLineage149.test.ts` seul | 23 PASS, 0 FAIL | idem |
| Matrice réseau `matrix149.mjs all` | 38 PASS, 0 FAIL | idem |
| Rejeux `replay_r3.mjs all` | 23 PASS | idem |
| Rejeux `replay_r4.mjs all` (nouveau) | 22 PASS | idem |
| Mesures navigateur F2 | 9 PASS, 9 captures | idem |
| `tsc --noEmit` serveur | 16 erreurs, toutes de base, 0 dans les fichiers 149 | idem |
| `tsc --noEmit` web | 10 erreurs, toutes de base, 0 dans les fichiers Bridget | idem |

<a id="extrait-6"></a>

## Extrait 6

| Constat | Statut | Par qui |
|---|---|---|
| F1 (r3, sûreté) : un contexte forgé rendait Lineage indisponible pour toute la racine | Corrigé. Vérifié en r4 (5 nouveaux tests, mutation testée, captures 06 et 07). | Correction de Sol dans `BridgetLineage.ts` |
| F2 (r3, interface) : la barre basse masquait la dernière ligne du journal | Corrigé. Vérifié en r4 (mesures du tableau ci-dessus). | Correction de Sol dans `BridgetTaskJournal.tsx` |
| Constat r2 : « Arrêter » masqué à 1280 px, carte ouverte | Clos en r3, confirmé en r4 | Vérification sur l'interface |
| `snapshot_changed` épuisé dégrade les 135 fils d'un coup | Observation. Aucune correction demandée. | À trancher par l'owner |
| Après une panne globale, la bannière watch exige un clic « Reconnecter Bridget » | Observation. Comportement de design existant. | À trancher par l'owner |
| Le message « Le contenu déjà lu est conservé » s'affiche même sans lecture (cas T4) | Observation. Non corrigé. | À trancher par l'owner |
| Un compte de fils a varié (137 puis 138) lors d'une lecture SQL | Non reproduit. Aucun événement de suppression. | Non qualifié |
| Fixture : le CLI ignore `--t3-thread`. S149-19 n'est pas prouvé. | Limite de fixture | Hors périmètre de cette recette |
| Fixture : un second fil hôte du même projet ne peut pas être lié (ZD-1) | Limite de fixture. Le cas est une observation. | Hors périmètre de cette recette |

<a id="extrait-7"></a>

## Extrait 7

1. Le daemon Bridget est simulé (`bridget_fixture.mjs`). Cette preuve ne valide pas le moteur natif.
2. Les données sont synthétiques (`[recette149]`). Le modèle affiché est une donnée de fixture. Aucun modèle n'a tourné.
3. SC001 n'est pas validé. Une délégation réelle de bout en bout n'a pas eu lieu.
4. Le MCP privé authentifié n'est pas prouvé. La matrice valide les scopes côté RPC, pas le MCP privé.
5. T036, T037, T038 et T039 ne sont pas prouvés par cette recette. Leurs preuves natives relèvent d'autres owners, encore en cours.
6. Le mobile n'est testé qu'en jsdom. Le viewport de 375 px est une fenêtre de navigateur, pas une vraie coque mobile.
7. iOS et la coque desktop ne sont pas testés.
8. La recette ne remplace aucune preuve de service. Elle vaut pour l'interface et le serveur T3 uniquement.
