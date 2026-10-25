# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le brief indiquait 10 erreurs web et 16 erreurs serveur préexistantes, avec zéro erreur nouvelle. Les 10 erreurs web de ce tour correspondent à cette baseline. L'hypothèse « cause probable : packages/contracts » ci-dessous n'est pas établie et ne constitue pas un nouveau finding.
- La recette native réelle est déjà décrite dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-native-recipe149-sonnet-r1.md`. Elle a observé `journal_unavailable` après l'arrêt réel d'un enfant. Ce test prouve le nouveau rendu ; il ne remplace pas cette recette.
- Le principal a changé le libellé avant ce tour. L'empreinte « avant » ci-dessous contient déjà ce changement. Haiku a ensuite modifié uniquement sa mise en forme. La nouvelle revue Sonnet porte sur ce petit changement de libellé ; les approbations des autres sources T3 restent distinctes.

<a id="extrait-2"></a>

## Extrait 2

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal149.test.tsx` : test ajouté (9 tests devenus 10).
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal.tsx` : mise en forme seule (voir ci-dessous).

<a id="extrait-3"></a>

## Extrait 3

Environnement : Node v24.13.1 (`/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin`). `node_modules` est un lien symbolique vers le dépôt principal, déjà en place.

<a id="extrait-4"></a>

## Extrait 4

- Les tests existants sur la bannière « Lineage Bridget indisponible » et sur la reconnexion (« root global ») passent sans changement.
- Pas de nouvelle fixture ajoutée pour un cas « root ordinaire » : la fixture existante du test de reconnexion le couvre déjà.
- Les preuves UI antérieures restent valables sur le même périmètre : seul le libellé d'un cas a changé.
- Aucun test global de la suite T3 n'a été lancé (demande du brief).

<a id="extrait-5"></a>

## Extrait 5

1. **Mutation non exécutée.** Je n'ai pas relancé le test contre l'ancien libellé, pour ne pas modifier le code de production. Par construction, l'ancien texte commence par « Bridget indisponible (journal_unavailable) ». Il ferait donc échouer les deux assertions de message. Ce point n'est pas prouvé par exécution.
2. **Fixtures mockées.** Le test vérifie le rendu du composant avec une erreur simulée. Il ne prouve pas que le serveur renvoie `journal_unavailable` dans un cas réel. Ce point relève de la recette native.
3. **Aucun modèle ni fournisseur.** Ces tests n'impliquent aucun fournisseur ni modèle. Aucun succès de modèle n'est déduit.
4. **Empreinte de production changée.** Une revue Sonnet finale est requise sur `4abac314...`.
5. **Typecheck web : 10 erreurs hors périmètre.** Elles sont dans `apps/web/src/components/chat/MessagesTimeline.logic.test.ts` (7) et `MessagesTimeline.test.tsx` (3). Ces deux fichiers ne sont pas modifiés dans l'arbre de travail. Le brief indiquait 0 erreur web en baseline, mais je n'ai pas relancé le typecheck avant modification. Cause probable : les types modifiés dans `packages/contracts`. Cette cause n'est pas vérifiée. Mes fichiers ont 0 erreur. Journal complet : `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/tsc-web.log`.
6. **Lint global et suite T3 non lancés.** Le lint a été lancé seulement sur les 2 fichiers ciblés.
7. **Aucune capture d'écran** n'a été produite ni utilisée.
