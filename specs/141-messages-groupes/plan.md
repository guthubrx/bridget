# Plan141 — Présentation des lots directs

Date : 2026-10-07. Statut : Implemented, non activé. Gate de réutilisation, Converge passe 2 et audit final validés.
Début de préparation : 07:30 CEST. Estimation totale : 20–35 minutes, hors livraison de production.
Branche Bridget : session-141-messages-groupes.
Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes
Code T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
Maquette : /Users/moi/.cache/bridget-grouped-mockup-20261007/index.html

## Contexte technique

Le frontend T3 utilise React, TypeScript et Vite Plus/Vitest. SPEC140 fournit une projection pure puis BridgetMessageBody dans la timeline existante. Le format Rust est déjà produit par batch_envelope. Aucun changement Rust n'est prévu.

La maquette sert de référence pour les noms visibles, les sections et le groupe compact. Les sujets illustratifs et les blocs source sont supprimés du comportement cible. Le logo et les thèmes existants sont réutilisés.

## Réutilisation de l'existant

| Élément | Source vérifiée | Changement prévu |
|---|---|---|
| Type de projection | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:86 | Ajouter une collection facultative de messages directs projetés. |
| Projection pure | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 | Étendre la branche des lots directs avec un découpage conservateur. |
| Branche des lots | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:163 | Garder le comportement des notifications. |
| Carte existante | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4034 | Ajouter les sections dans BridgetMessageBody, sans seconde timeline. |
| Corps Markdown | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4119 | Réutiliser UserMessageBody pour chaque corps complet et le repli sûr. |
| Clé fil/message | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2239 | Conserver l'isolation de l'état local. |
| Ancrage | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4049 | Appeler ctx.onToggleWorkEntry lors des replis imbriqués. |
| Copie et actions | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2264 | Réutiliser MessageCopyButton avec row.message.text original pour les seuls lots directs ; préserver les autres copies et les pièces jointes. |
| Tests existants | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:208 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:287 | Étendre ces suites avec les cas SPEC141. |

## Conception

1. Reconnaître uniquement l'en-tête 📥 direct dans la borne actuelle de 1024 caractères. Conserver les autres familles. Ne pas utiliser le texte comme preuve d'origine.
2. Vérifier la grammaire réelle documentée dans le contrat. Découper en un passage linéaire. Valider total, indices contigus, expéditeurs non vides et limites. Une ligne susceptible d'être un séparateur dans un corps rend le lot ambigu. Revenir alors au corps entier.
3. Conserver les corps par tranches du texte. Retirer seulement les retours de ligne ajoutés par le format autour des sections. Le suffixe final connu peut être masqué uniquement par correspondance exacte. Un suffixe inconnu reste lisible.
4. Afficher un groupe fermé sur une ligne. À la première ouverture, ouvrir la première section et laisser les autres fermées. Conserver ensuite les choix indépendants lors des replis du groupe. Afficher le nom reçu et au plus les 120 premiers caractères du corps. Un expéditeur composé du seul UUID conserve cet UUID visible ; aucun libellé « Bridget » ne remplace cette donnée. Aucun sujet dérivé ni type de mission n'est ajouté.
5. Réutiliser boutons natifs, aria-expanded, aria-controls et le focus existant. Un nom long peut se répartir sur plusieurs lignes dans la section. Conserver les deux thèmes et les couleurs de SPEC140.
6. Garder l'état local attaché au fil/message. L'ancrage existant reçoit chaque changement de hauteur. Aucun état dérivé persistant ni store global n'est ajouté.
7. Pour tous les lots directs reconnus, ne pas rendre les commandes Sources/Détails techniques, même si le découpage a échoué. Un en-tête trop long garde le rendu intégral historique, sans panneau supplémentaire. La copie reste celle du message complet.
8. Pour les seuls lots directs, fournir row.message.text original au MessageCopyButton existant, y compris en repli sûr. Le chemin actuel remplace une référence comme [x](t3-context://v1/skill/ctx_1) par x en l'absence de contexte structuré ; il ne satisfait donc pas la copie intégrale du lot. Préserver cette référence dans la copie du lot. Ne pas modifier la transformation des autres familles ni le helper partagé. Ajouter un test explicite sans contexte structuré et les cas UUID seuls.

## Phases et validation

Phase 0 : recherche ciblée, spécification, plan et audit de réutilisation. Lire et valider le gate avant de générer les tâches.

Phase 1 : écrire les tests de projection en échec (TDD). Couvrir le format réel, LF/CRLF, les indices, noms longs, corps vides ou longs, séparateurs dans le corps et suffixes inconnus. Implémenter ensuite la projection minimale.

Phase 2 : écrire les tests de rendu puis modifier BridgetMessageBody et le choix du texte transmis à MessageCopyButton pour les seuls lots directs. Vérifier sections indépendantes, noms et UUID seuls, aperçus littéraux, absence de sources, copie originale avec référence t3-context sans contexte structuré, pièces jointes/actions et isolation fil/message.

Phase 3 : contrôles ciblés de tests, format, lint, types du package web et build web isolé. Recette du vrai composant : clavier, largeur 320, thèmes, textes longs, ancrage et repli sûr. Aucun service actif n'est redémarré.

Phase 4 : Converge relie chaque FR/SC à une preuve code/test/recette. Analyse de cohérence des artefacts. Audit final du diff, de la complexité, du minimalisme et de la charge future. Contre-revue externe seulement si un autre fournisseur est joignable. Aucun fournisseur alternatif n'est actuellement identifié par Bridget.

## Gates constitutionnels

- [x] Autorisation de session et worktrees dédiés établis.
- [x] Articles XIX/XX : étendre projection et rendu existants ; aucune dépendance, API ou nouvelle abstraction générale.
- [x] Paresse vertueuse : une règle locale explicite évite une présentation dupliquée.
- [x] Impatience maîtrisée : suites ciblées et recette isolée donnent une boucle courte.
- [x] Exigence professionnelle : copie intacte, limites documentées, tests comportementaux et relecture du diff avant livraison.
- [x] Responsabilité future : contrat de format et commandes de validation lisibles sans contexte de conversation.
- [x] Article XVIII : reconnaissance bornée O(1), découpage et validation O(n), rendu O(n), n étant la longueur du texte. Aucun tri ni recherche répétée dans une boucle.
- [x] Données actives protégées : fixtures synthétiques, aucune migration ni écriture de conversation.
- [x] Audit de réutilisation PASS documenté ; lecture du principal requise avant tasks.

## Exceptions ciblées et limites

Les scripts et modèles locaux SpecKit sont absents. Le protocole est appliqué manuellement ; aucune phase n'est supprimée. La synchronisation utilisateur a été effectuée avant cette préparation.

Les prescriptions génériques Next.js, frontend-v2, next-intl et Pytest ne correspondent pas au frontend T3 React/Vite/Vitest vérifié. Les tests sont ajoutés aux suites existantes avec un titre SPEC141. Aucun framework, dossier de traduction ou outil de test n'est ajouté pour satisfaire une règle d'un autre projet. Les contrôles restent sur les fichiers et le package web concernés. Aucun lint, test ou build global du monorepo n'est imposé. Cette exception suit le périmètre explicite de la session et évite une correction de travaux étrangers.

Aucune décision architecturale nouvelle ni reconstruction de composant. L'implémentation et la recette ne redémarrent ni n'installent la production. La livraison autorisée ensuite forme une phase distincte, sans relance de T3. Aucun nettoyage d'autres branches ou worktrees. Les artefacts SpecKit restent dans le dépôt Bridget ; le dépôt T3 reçoit uniquement les changements d'implémentation nécessaires.

## Clôture documentaire

Dix tâches validées sur preuves. Converge passe 2 : CONVERGED, 22 exigences/critères couverts, fichier des tâches inchangé pendant la lecture seule. Audit final : A, score 100, diff ciblé de quatre fichiers, aucun point critique/majeur ouvert. Validateur du principal : zéro erreur, zéro avertissement, sortie 0.

Audit : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/2026-10-07/session-2026-10-07-spec141-01
Résultats : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/validation/results.json

Le statut Implemented ne signifie pas activé dans l'application de production. Le 2026-10-07, l'utilisateur a autorisé commit, fusion, push et installation de la session141, avec interdiction explicite de relancer T3. Le principal réalise la livraison avec sauvegarde et vérification. Ce document ne revendique pas encore d'installation ni d'activation ; les preuves seront consignées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/livraison.md.
