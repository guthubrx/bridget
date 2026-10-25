# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/plan.md](../../../../../../specs/143-echanges-discrets/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Frontend : `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/`, base `5724eb7f12e4556327efa14f51f43b9caf404e25`.
Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/`.
Préflight sync appliqué par le principal. Scripts et templates SpecKit locaux vérifiés absents ; appliquer les protocoles des skills utilisateur, sans installation ni réinitialisation du projet.

<a id="extrait-2"></a>

## Extrait 2

- `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` : projection existante des enveloppes Bridget, reconnaissance bornée et repli sûr.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` : rendu Bridget, logo, toggle, Markdown, copie et état d'ouverture existants.
- `PlainWorkEntryRow` dans ce rendu, ligne4915 : intégration des sorties outils ; appel et résultat sont déjà projetés dans la même work entry.
- Tests existants `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx`.

<a id="extrait-3"></a>

## Extrait 3

Recette native sur aperçu isolé de la révision finale à11:33–11:35 CEST : ouverture click, repli Enter et ouverture Space avec focus stable après deux RAF ; note visible et lien préservé ; citation DOM réelle dans note sur texte répété résolue sans ambiguïté. Conteneur320px testé sans débordement, pas viewport mobile. Pas de clipboard hôte, navigation citation E2E ou livraison fournisseur E2E revendiquée. Services isolés arrêtés à11:35:22, ports libres, T3 actif inchangé. Convergence principale finale CONVERGED à11:39:54, tâches12/13 byte-identiques pendant la comparaison ; T013 cochée seulement ensuite en phase documentaire distincte.
