# Recherche initiale — SPEC143

Date : 2026-10-07. Exploration et sources reçues. Seuls les formats ci-dessous sont confirmés pour la compaction.

## Décisions acquises

- Entrées : reprendre les familles déjà reconnues dans SPEC139/140/141. Retirer leur fond/bordure, diminuer le padding, garder leur contenu et interactions.
- Sorties : reconnaître l'identité MCP et une forme réelle confirmée, pas un texte contenant « Bridget ».
- Destinataire : nom fourni par une donnée attestée ; sinon ID bref réel ; sinon libellé neutre. Aucun appel réseau pour enrichir un nom.
- Sémantique : la fin d'un appel outil ne prouve pas la réception. Ne pas déduire « livré » de `completed`.
- Repli : rendu natif inchangé pour une sortie inconnue, ambiguë ou hors périmètre.

## Exploration confirmée avant gate

| Question | Source à lire | Décision après preuve |
|---|---|---|
| Quels noms MCP et payloads atteignent la timeline ? | Discovery du principal, `PlainWorkEntryRow:4915` | Envoi Codex et Claude seulement, formes ci-dessous |
| Où se trouvent destinataire et résultat brut ? | Paramètres `to`/`body` et résultat de la même work entry | Nom attesté/ID bref ; pas d'annuaire réseau ajouté |
| Quels styles sont hérités ? | `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:2125` et `:2130` | Retirer bordure et `backgroundColor` seulement sur la branche Bridget |
| Quels tests couvrent déjà copie/toggle ? | Deux suites `MessagesTimeline` existantes | Étendre les tests, pas recréer un harness |

Codex : `toolData={type:mcpToolCall,server:'bridget',tool:'bridget_send',arguments:{to,body,...},result:{content:[text JSON],structuredContent:{status,...}},status,error...}`. Claude : `toolData={toolName:'mcp__bridget__bridget_send',input:{to,body,...},result:{type:'tool_result',tool_use_id,content:JSON string ou array,is_error?}}`. Appel et résultat occupent la même entrée WorkLog. Completed ne signifie pas accepted : le MCP peut retourner `in_flight`. Le libellé neutre « Envoi » ne promet pas de livraison. L'erreur reste visible.

Refus attestés par discovery : `unknown_recipient`, `cross_project_reason_required`, `envelope_mismatch`. Même si l'appel technique est completed, ces résultats ne sont ni acceptation ni livraison. MCP `isError`, Claude `is_error`, refus et formes/statuts inconnus gardent le rendu natif. Ne pas inventer de taxonomie de succès.

La revue indépendante signale que le helper natif de détection d'échec ne lit pas toolData : ne pas compter sur lui pour détecter les drapeaux MCP ou refus. Lecture confirmée de `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx:4601` : `buildToolCallExpandedBody` applique `trim`, puis sérialise toolData par `JSON.stringify`. Les valeurs JSON restent accessibles au dépliage ; leur ordre/espacement sérialisé n'est pas un contrat d'octets. Corps entrants et copie restent exacts.

## Références et limites

Sources projet : SPEC141 et ses tests existants, projection/render proposés par le principal. Références utilisateur déjà lues : constitution, standards frontend/tests, charge cognitive et qualité du code. Les primitives locales `.specify/scripts/bash/check-prerequisites.sh` et `.specify/templates/spec-template.md` sont absentes dans le worktree143.

Sources primaires consultées par le principal : [WAI disclosure](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/) pour bouton/`aria-expanded`/Enter/Space ; [web.dev focus](https://web.dev/learn/accessibility/focus) pour focus visible ; [Testing Library queries](https://testing-library.com/docs/queries/about/) pour assertions proches des actions utilisateur ; [Playwright bonnes pratiques](https://playwright.dev/docs/best-practices) pour recette isolée. Ces références ne prouvent pas une conformité complète de l'application.

Annuaire Bridget `same_project` vérifié par le principal : seulement `bdget` Codex, aucun autre fournisseur joignable dans le périmètre. Revue inter-fournisseur indisponible ; revue interne indépendante réalisée, corrections ci-dessus retenues. Aucun envoi externe non sollicité, mission active, modèle payant ou redémarrage pour cette recherche. Gate PASS lu par le principal, huit tâches autorisées. Aperçu isolé autorisé, pas de déploiement ni relance production.

## Recherche et arbitrages US4 — Révision finale

Préfixe de relais strict déjà présent dans les consignes entrantes ; rendu `AssistantTimelineRow`, texte original, citations, Markdown, métadonnées, fichiers et contrôle de focus réutilisés. Noms préparés uniquement depuis les en-têtes directs antérieurs du même fil, O(n), sans I/O.

La contre-revue a reproduit deux pertes de compatibilité : citation ambiguë sur texte répété au passage natif et lien de note défini dans le corps agent. Le corps mixte reste donc monté sous CSS `display:none`, sans `hidden` ni `aria-hidden`, pour garder stable le flux canonique de citation. Ce coût égale le rendu complet antérieur143. Tout `[` hors fence, HTML hors code et note/frontière non canonique déclenche le natif complet ; aucune résolution Markdown nouvelle.

Un test avec le vrai `useComposerFocusState` a reproduit le focus restauré vers le composeur après repli. La primitive existante `ctx.onToggleWorkEntry(row.id,false)` corrige ce comportement. Réédition du texte A→B→A repliée, état lié séparément au fil/message et absence de remount par fragment conservés. Tests finaux392 et contre-revue APPROVE reçus ; aucune livraison runtime déduite.

Recette native finale réellement observée sur clone isolé : les captures et limites sont enregistrées séparément dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/ui-recipe-US4.json`. L'aperçu natif a eu une déconnexion fugace, résolue par `preview_open`, sans redémarrage de l'application. Le socle avait utilisé un navigateur de remplacement ; ne pas confondre ces deux recettes.
