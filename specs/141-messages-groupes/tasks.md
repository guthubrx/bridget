# Tâches141 — Messages Bridget groupés

Date : 2026-10-07. Statut : Implemented, non activé.
Gate de réutilisation PASS lu et validé par le principal avant génération.
Suites ciblées : passage final 294 PASS (215 logique, 79 rendu), 2,84 secondes, sur le code gelé. Baseline existante : 269 PASS dans deux suites. Dix tâches sur dix validées. T008 cochée après Converge passe 2 et le verdict d'audit, dans une phase documentaire distincte de la lecture seule. Aucune installation de production.

## Préparation et fondations

- [x] T001 Vérifier les worktrees, la synchronisation, le format réel et la baseline ; lire le gate PASS et fixer le contrat dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/plan.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/reuse-audit.md. Preuves : baseline 269 PASS transmise par le principal et gate validé avant ce fichier.

## US4 — Repli sûr et projection (P1)

Cette histoire correspond à US14104. Elle précède le rendu car le découpage doit être certain avant de présenter les auteurs. Test indépendant : chaque entrée ambiguë conserve le corps entier et la copie source.

- [x] T002 [US4] Ajouter les tests RED SPEC141 dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts : format réel de trois messages, LF/CRLF, indices et compteurs incohérents, nom long, corps vide/long, séparateur dans le corps, suffixe final exact/inconnu et en-tête trop long. Vérifier ordre, corps intacts et aperçu littéral ≤120 ; garder les cinq familles hors lot direct compatibles. Exécuter uniquement la suite concernée et consigner les échecs attendus.
- [x] T003 [US4] Étendre la projection existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts pour les lots directs seulement. Valider en O(n), conserver les tranches de corps, rejeter les ambiguïtés et garder le repli intégral. Réutiliser au besoin le nettoyage de libellé direct/lot pour sa règle de compatibilité. Ne pas créer de stockage, API ou dépendance. Rendre les tests T002 verts.

## US1 — Groupe compact et sections nommées (P1)

Cette histoire correspond à US14101. Test indépendant : trois messages reçus donnent trois sections dans l'ordre, avec les noms et extraits littéraux.

- [x] T004 [US1] Ajouter les tests RED de rendu SPEC141 dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx : groupe fermé, premier message ouvert à la première ouverture, sections indépendantes, mêmes auteurs distincts, UUID seul visible sans nom « Bridget » inventé, aperçus ≤120, aucun sujet inféré, aucune source pour lot valide ou repli sûr, copie exacte et régressions SPEC140. Ajouter explicitement un lot contenant [x](t3-context://v1/skill/ctx_1) sans contexte structuré : la copie doit être row.message.text intégral, y compris en repli sûr. Les autres familles gardent leur copie actuelle. Consigner les échecs attendus avant modification du rendu.
- [x] T005 [US1] Étendre BridgetMessageBody dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx. Réutiliser UserMessageBody, logo et thèmes. Afficher noms et extraits, conserver un UUID seul visible, premier message ouvert puis états indépendants conservés. Supprimer la branche technique pour les lots directs seulement. Fournir row.message.text original à MessageCopyButton pour les seuls lots directs, y compris le repli sûr, afin de conserver les références t3-context sans contexte structuré. Ne pas modifier la copie des autres familles ni le helper partagé. Un composant local de section est permis pour porter ses interactions et son accessibilité répétées. Rendre les tests T004 verts.

## US2 — Lecture et interactions (P1)

Cette histoire correspond à US14102. Test indépendant : les sections s'ouvrent au clic et au clavier, restent indépendantes et ne déplacent pas l'état vers un autre fil/message.

- [x] T006 [US2] Vérifier le vrai composant /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx et ses tests existants. Couvrir aria-expanded/controls, focus, Tab/Entrée/Espace, conservation des choix, isolation fil/message et appel d'ancrage existant. Faire la recette isolée en largeur 320, dans les deux thèmes, avec textes et noms longs ; consigner preuves et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/implementation.md.

## US3 — Conservation et contrôles ciblés (P1)

Cette histoire correspond à US14103. Test indépendant : copie égale au message reçu et pièces jointes/actions conservées, sans source pour les lots directs.

- [x] T007 [US3] Valider copie exacte, pièces jointes/actions et enveloppes SPEC140 hors lot direct dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx. Exécuter les deux suites, format/lint des fichiers modifiés, types du package web et build web isolé avec les binaires déjà installés selon /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/quickstart.md. Ne pas réinstaller les dépendances ni lancer de contrôle global ; consigner les résultats réels.

## Convergence et audit

- [x] T008 Relier FR14101–FR14115 et SC14101–SC14107 aux preuves dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/converge.md. Relire le diff ciblé, complexité O(n), minimalisme et responsabilité future ; consigner l'audit et les limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/2026-10-07/session-2026-10-07-spec141-01/scoring.md. Mettre à jour tests/statuts uniquement sur preuve. Aucune installation, aucun restart de production et aucun commit automatique.

## Dépendances et exécution

Ordre : T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008. Les histoires ont toutes priorité P1. Le repli sûr est fondamental ; il est traité avant le rendu. Aucun marqueur [P] : les modifications des deux fichiers d'implémentation et de leurs tests se suivent. Les observations de recette peuvent être préparées séparément, mais ne doivent pas éditer les mêmes fichiers.

L'incrément de projection reste testable avant le rendu. Le rendu complète ensuite le besoin validé ; aucune version réduite n'est livrée à la place du plan complet. Chaque tâche possède un résultat observable. T003 et T005 réutilisent les règles locales. T008 vérifie les lignes et abstractions conservées pour limiter la charge future.

## Corrections issues de Converge — passe 1

Ces deux tâches complètent le périmètre existant. Elles ont suivi RED puis GREEN. Leurs cases ont été mises à jour après les preuves, dans une phase de journalisation distincte de Converge. Aucun statut ni aucune case antérieure n'a changé pendant la passe 1. La nouvelle passe Converge et le verdict final d'audit restent nécessaires pour T008.

- [x] T009 [US4] Préserver exactement les fins de ligne du corps aux frontières des sections dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts. Écrire le test RED réel body "A\r" dans une enveloppe LF : la projection doit garder "A\r" au lieu de "A". Distinguer les retours LF/CRLF structurels du producteur d'un CR seul appartenant au corps, aux frontières intermédiaires et finales. Appliquer le correctif minimal puis obtenir GREEN sur les tests de conservation et les deux suites ciblées. Ne pas normaliser les corps.
- [x] T010 [US3] Préserver la copie row.message.text originale quand l'en-tête canonique d'un lot direct dépasse 1024 caractères dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx et les tests /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx, avec le prédicat de reconnaissance nécessaire dans /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts. Écrire le test RED : en-tête canonique long, projection null et référence [x](t3-context://v1/skill/ctx_1) sans contexte structuré ; la copie ne doit pas devenir x. Utiliser un prédicat canonique ancré O(n) pour la seule décision de copie. Garder la reconnaissance d'affichage bornée et le rendu historique intégral inchangés. Obtenir GREEN sur ce cas et les copies des autres familles ; ne pas modifier le helper partagé.

Clôture après Converge : l'empreinte des tâches est restée identique pendant la passe 2. T008 a été cochée ensuite, après réception de l'audit A/100, sans critique/majeur ouvert et avec validateur sortie 0. La session est implémentée et non installée. La livraison reste distincte et non engagée.
