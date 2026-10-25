# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/validation.md](../../../../../../specs/145-bridget-thread-viewer/validation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/node_modules/`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/node_modules/`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/node_modules/`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/contracts/node_modules/`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime/node_modules/`.

<a id="extrait-2"></a>

## Extrait 2

Les liens @t3tools/contracts et @t3tools/client-runtime observés dans le root/web, ainsi que contracts/shared/ssh/tailscale/web côté serveur, ciblent tous ce worktree145. Exemples de cibles exactes : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/contracts`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/shared`, `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/ssh` et `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/tailscale`.

<a id="extrait-3"></a>

## Extrait 3

Accord opérationnel transmis par le principal : CLI exit3 pour daemon absent ou namespace indisponible ; T3 mappe unavailable sans parser stderr. Exit2 JSON métier reste inchangé. Aucun nouveau code métier. Cette décision contractuelle devra être vérifiée par les tests CLI/service ; elle ne constitue pas leur résultat.

<a id="extrait-4"></a>

## Extrait 4

Reçu du travailleur Rust transmis par le principal :10 tests SPEC145 PASS,4 tests CLI PASS,9 tests threads PASS,2 tests SPEC138 T3 PASS et295 tests transport PASS. Un test transport de baseline reste ignoré ; il n'est pas déclaré exécuté. Format, clippy avec -D warnings, build et diff passent avec exit0.

<a id="extrait-5"></a>

## Extrait 5

Revue finale31 sources/tests, dont8 Rust et23 T3. Hash agrégé avant/après identique selon le principal : `7c765f38d24163def6fe80a1b4c792aca1b954e3f6612ee63ac1215a40456bc6`. Verdicts Rust et T3 : APPROVE. Les relecteurs appartiennent au même fournisseur ; aucun fournisseur différent n'est disponible.

<a id="extrait-6"></a>

## Extrait 6

Les outils de capture native échouent sur une incohérence de redimensionnement de la plateforme. Aucune capture finale n'est revendiquée et le harness n'est pas modifié pour l'obtenir. Les22 cas clavier des onglets sont de vraies interactions JSDOM. Aucune preuve clavier complète de T3 sur matériel natif ou application installée n'est ajoutée.

<a id="extrait-7"></a>

## Extrait 7

Le principal confirme les deux branches session-145-bridget-thread-viewer et la préservation des travaux extérieurs. Main Bridget conserve ses six chemins sales142 connus. Main T3 conserve seulement `/Users/moi/11.Repositories/t3code-local/native/resource-monitor/._target`. Aucun source ou harness n'est modifié par T020.
