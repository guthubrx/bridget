# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/tasks.md](../../../../../../specs/141-messages-groupes/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Statut : Implemented, installé, activation différée.
Gate de réutilisation PASS lu et validé par le principal avant génération.
Suites ciblées : passage final 294 PASS (215 logique, 79 rendu), 2,84 secondes, sur le code gelé, puis 294/294 fraîchement rejoués avant livraison. Baseline existante : 269 PASS dans deux suites. Dix tâches sur dix validées. T008 cochée après Converge passe 2 et le verdict d'audit, dans une phase documentaire distincte de la lecture seule. Application adjacente installée et vérifiée ; aucune activation ni relance de T3.

<a id="extrait-2"></a>

## Extrait 2

- [x] T002 [US4] Ajouter les tests RED SPEC141 dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts : format réel de trois messages, LF/CRLF, indices et compteurs incohérents, nom long, corps vide/long, séparateur dans le corps, suffixe final exact/inconnu et en-tête trop long. Vérifier ordre, corps intacts et aperçu littéral ≤120 ; garder les cinq familles hors lot direct compatibles. Exécuter uniquement la suite concernée et consigner les échecs attendus.
- [x] T003 [US4] Étendre la projection existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts pour les lots directs seulement. Valider en O(n), conserver les tranches de corps, rejeter les ambiguïtés et garder le repli intégral. Réutiliser au besoin le nettoyage de libellé direct/lot pour sa règle de compatibilité. Ne pas créer de stockage, API ou dépendance. Rendre les tests T002 verts.

<a id="extrait-3"></a>

## Extrait 3

- [x] T006 [US2] Vérifier le vrai composant /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx et ses tests existants. Couvrir aria-expanded/controls, focus, Tab/Entrée/Espace, conservation des choix, isolation fil/message et appel d'ancrage existant. Faire la recette isolée en largeur 320, dans les deux thèmes, avec textes et noms longs ; consigner preuves et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/implementation.md.

<a id="extrait-4"></a>

## Extrait 4

- [x] T007 [US3] Valider copie exacte, pièces jointes/actions et enveloppes SPEC140 hors lot direct dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx. Exécuter les deux suites, format/lint des fichiers modifiés, types du package web et build web isolé avec les binaires déjà installés selon /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/quickstart.md. Ne pas réinstaller les dépendances ni lancer de contrôle global ; consigner les résultats réels.

<a id="extrait-5"></a>

## Extrait 5

- [x] T009 [US4] Préserver exactement les fins de ligne du corps aux frontières des sections dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Écrire le test RED réel body "A\r" dans une enveloppe LF : la projection doit garder "A\r" au lieu de "A". Distinguer les retours LF/CRLF structurels du producteur d'un CR seul appartenant au corps, aux frontières intermédiaires et finales. Appliquer le correctif minimal puis obtenir GREEN sur les tests de conservation et les deux suites ciblées. Ne pas normaliser les corps.
- [x] T010 [US3] Préserver la copie row.message.text originale quand l'en-tête canonique d'un lot direct dépasse 1024 caractères dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx et les tests /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx, avec le prédicat de reconnaissance nécessaire dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts. Écrire le test RED : en-tête canonique long, projection null et référence [x](t3-context://v1/skill/ctx_1) sans contexte structuré ; la copie ne doit pas devenir x. Utiliser un prédicat canonique ancré O(n) pour la seule décision de copie. Garder la reconnaissance d'affichage bornée et le rendu historique intégral inchangés. Obtenir GREEN sur ce cas et les copies des autres familles ; ne pas modifier le helper partagé.

<a id="extrait-6"></a>

## Extrait 6

Clôture après Converge : l'empreinte des tâches est restée identique pendant la passe 2. T008 a été cochée ensuite, après réception de l'audit A/100, sans critique/majeur ouvert et avec validateur sortie 0. La session est implémentée et non installée. La livraison reste distincte et non engagée.
Cette clôture décrit l'état avant autorisation de livraison. État courant après livraison : /Applications/T3 Code (Local SPEC141).app installée à côté et vérifiée ; /Applications/T3 Code (Local).app reste active et intacte. Activation différée, aucune relance effectuée. Le journal post-installation reste à committer par le principal ; aucun identifiant final n'est inventé.
