# Plan initial — SPEC143

Date : 2026-10-07. Statut : Implemented — non installé, non activé.13/13 tâches sur preuves, convergence principale US4 CONVERGED à11:39:54 CEST. Le socle conserve ses preuves historiques ; US4 a ses propres392 tests, contrôles, contre-revue et recette. T013 clôturée après verdict, dans une phase documentaire distincte.

## Contexte

Frontend : `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/`, base `5724eb7f12e4556327efa14f51f43b9caf404e25`.
Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/`.
Préflight sync appliqué par le principal. Scripts et templates SpecKit locaux vérifiés absents ; appliquer les protocoles des skills utilisateur, sans installation ni réinitialisation du projet.

## Réutilisations confirmées par exploration

- `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` : projection existante des enveloppes Bridget, reconnaissance bornée et repli sûr.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` : rendu Bridget, logo, toggle, Markdown, copie et état d'ouverture existants.
- `PlainWorkEntryRow` dans ce rendu, ligne4915 : intégration des sorties outils ; appel et résultat sont déjà projetés dans la même work entry.
- Tests existants `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx`.

Aucun nouveau composant partagé, store, parser générique ou framework proposé. Deux petits helpers typés peuvent étendre la logique locale pour les formes attestées Codex/Claude et l'extraction sûre du destinataire ; leur règle de compatibilité est justifiée dans l'audit. Ne pas altérer la projection141 pour changer la copie brute.

Sorties retenues : Codex `mcpToolCall`/serveur `bridget`/outil `bridget_send` et Claude `mcp__bridget__bridget_send`, avec paramètres et résultat de la même work entry. Pas de reconnaissance par un titre libre ni par un simple mot dans le texte. Statut neutre « Envoi » ; conserver les erreurs. Le retour MCP peut être `in_flight` malgré l'état technique completed.

MCP `isError`, Claude `is_error`, refus confirmés (`unknown_recipient`, `cross_project_reason_required`, `envelope_mismatch`) et statut/résultat inconnu gardent le natif. Le JSON technique reste au dépliage existant, sans nouveau panneau Sources. Le helper natif de détection d'échec ne lit pas ces champs toolData : les garde-fous de projection doivent les contrôler explicitement.

## Séquence prévue

1. Explorer les vrais noms/outils MCP, paramètres, résultats et états reçus par `PlainWorkEntryRow`. Recenser les familles d'entrées reconnues et les styles hérités. Exploration reçue, preuves dans recherche/audit.
2. Mettre à jour recherche et contrat avec les seules formes confirmées. Produire le gate réutilisation, puis les tâches et Analyze avant implémentation.
3. Tests RED : retrait des cadres d'entrée, sortie confirmée compacte, dépliage/copie/clavier, nom absent, erreur, `completed` non livré, fallback natif et ordinary inchangé.
4. Modifier les styles d'entrées et l'intégration locale des seules sorties confirmées. Réutiliser les interactions et le rendu complet.
5. Tests GREEN ciblés et recette isolée. Converge lecture seule et audit proportionné. Pas de restart, commit ou déploiement.

## Contraintes et vérification

React/Vitest existants : les standards génériques Next.js/Pytest Cartae ne s'appliquent pas. Utiliser les binaires installés sans réinstallation. Parents/wrappers lus lignes2125/2130. Tester le contenu et l'interaction, pas seulement un snapshot. Réutiliser le contrôle accessible existant, bouton ou `role=button`, son `aria-expanded`, clavier et focus visible ; conserver contraste et retour à l'ancrage.

`buildToolCallExpandedBody:4601` sérialise toolData et applique déjà `trim` aux blocs. Préserver les valeurs JSON et le rendu natif complet, pas promettre des octets de sérialisation bruts. Les corps entrants et leur copie exacte restent régis par141. Aperçu isolé autorisé par l'utilisateur ; aucune relance de l'application active.

Charge cognitive réduite par réutilisation : une ligne au repos, données exactes à l'ouverture, pas de panneau supplémentaire. Les formats inconnus gardent le rendu natif plutôt qu'une interprétation hasardeuse.

## Complément US4 — Réponses textuelles interagents

Début : 2026-10-07 11:05:59 CEST. L'utilisateur autorise l'extension de143, pas une nouvelle session ni une livraison.

1. Vérifier dans les quatre fichiers existants le préfixe de relais, les frontières explicites des notes utilisateur, les associations ID/nom disponibles et les protections du rendu assistant. Consigner les preuves avant le GO.
2. Aligner contrat, modèle, audit de réutilisation et Analyze sur les formats réellement confirmés. Aucun parser générique ni enrichissement réseau.
3. Produire des tests RED indépendants du socle346 : préfixe strict et replis, état streaming, nom/ID, note utilisateur visible, ouverture souris/clavier, copie et protections du rendu assistant.
4. Étendre localement la projection et le rendu assistant : ligne à gauche « Entre agents », sans cadre et repliée par défaut. Réutiliser logo, Markdown et contrôles accessibles. Les messages ordinaires gardent leur rendu.
5. Rejouer les deux suites ciblées, format/lint/type/build et diff. Réaliser une recette sur le seul aperçu isolé autorisé. Faire une convergence et une revue proportionnée portant sur la révision US4, sans réutiliser fictivement l'audit du socle.

Le rendu complet doit préserver le texte original et les fonctionnalités de citation, copie, métadonnées, actions Markdown et fichiers modifiés. Une note pour l'utilisateur se sépare seulement par une frontière explicite documentée. Les données ambiguës et le streaming restent conservateurs. Une projection de texte ne prouve jamais une remise par Bridget.

### Formats confirmés et décisions du complément

L'exploration du codeworker, transmise par le principal, confirme la réutilisation de `AssistantTimelineRow`, du préfixe canonique déjà fourni aux agents et des `timelineEntries`. Le format retenu commence exactement par `↪ Réponse à UUID (relayée par Bridget) :` puis une ligne blanche. La projection s'applique seulement à une réponse terminée. Le streaming reste natif.

La frontière utilisateur fermée est une ligne `---` hors bloc de code, une ligne blanche, puis `Résumé pour toi :` ou `Pour toi :`, simple ou en gras. Les variantes incomplètes, plusieurs frontières et blocs de code ambigus conservent le rendu complet natif. Les marqueurs en bloc de code ne créent pas de frontière. Le nom du destinataire provient uniquement d'un en-tête direct Bridget antérieur, avec UUID correspondant, dans la même conversation. Un parcours mémoïsé O(n) prépare ces associations sans I/O ; UUID court sinon, UUID complet accessible.

La demande de citation ciblant le message force le rendu complet natif pour préserver les offsets. Copie, métadonnées et fichiers modifiés restent hors du repli. Les contrôles et actions Markdown sont réutilisés. Même périmètre de quatre fichiers : aucune dépendance, API, source daemon ou donnée live modifiée. Rejouer la baseline346 avant les tests RED US4.

### Révision après contre-revue fonctionnelle US4

Deux défauts reproduits motivent la révision : une citation dans la note repliée peut devenir ambiguë lorsqu'un texte identique existe aussi dans le corps agent ; un lien Markdown dans la note perd sa définition si celle-ci se trouve dans le corps séparé. Verdict intermédiaire APPROVE_WITH_CHANGES, pas une approbation finale.

Décision acceptée par le principal : maintenir le corps agent des réponses mixtes monté sous CSS `display:none`, sans `hidden` ni `aria-hidden`, pour stabiliser le flux de texte canonique des citations. Ce coût égale le rendu Markdown complet antérieur143 ; la compacité visuelle ne prétend pas réduire le calcul. Références Markdown, notes de bas de page et HTML hors code pouvant traverser la frontière gardent le message entier natif. Tester la réédition du texte A → B → A : le corps doit rester replié ; état fil/message protégé séparément.

Historique de la révision intermédiaire : six nouveaux échecs RED (quatre logique, deux UI),367 PASS sur373 cas. Le GREEN366 précédent n'était pas final. Ces étapes sont remplacées par la révision finale392 décrite ci-dessous, sans effacer leurs preuves.

### Révision finale gelée et vérifiée

La garde finale applique le natif à tout `[` hors fence dans une réponse mixte et aux notes non canoniques. Le focus est conservé par `ctx.onToggleWorkEntry(row.id,false)`, primitive existante qui évite la restauration du composeur au repli. Les tests RED suivants sont reçus puis corrigés :9 références→381 cas,6 notes malformées→387,4 espaces insécables/tabulations→391,1 focus réel `useComposerFocusState`→392.

Gel392 PASS,283 logique/109 UI, soit46 cas de plus que le socle346. Rejeux indépendants : principal5,40s à11:32:57 CEST ; relecteur6,08s à11:32:20 CEST, verdict APPROVE sans résidu. Format/lint/types/build/diff PASS ;22 warnings lint de baseline et avertissement chunk existant.

Recette native sur aperçu isolé de la révision finale à11:33–11:35 CEST : ouverture click, repli Enter et ouverture Space avec focus stable après deux RAF ; note visible et lien préservé ; citation DOM réelle dans note sur texte répété résolue sans ambiguïté. Conteneur320px testé sans débordement, pas viewport mobile. Pas de clipboard hôte, navigation citation E2E ou livraison fournisseur E2E revendiquée. Services isolés arrêtés à11:35:22, ports libres, T3 actif inchangé. Convergence principale finale CONVERGED à11:39:54, tâches12/13 byte-identiques pendant la comparaison ; T013 cochée seulement ensuite en phase documentaire distincte.
