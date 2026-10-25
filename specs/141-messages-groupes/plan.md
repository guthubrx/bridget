# Plan141 — Présentation des lots directs

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-1).

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-2).

La maquette sert de référence pour les noms visibles, les sections et le groupe compact. Les sujets illustratifs et les blocs source sont supprimés du comportement cible. Le logo et les thèmes existants sont réutilisés.

## Réutilisation de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-6).

## Clôture documentaire

Dix tâches validées sur preuves. Converge passe 2 : CONVERGED, 22 exigences/critères couverts, fichier des tâches inchangé pendant la lecture seule. Audit final : A, score 100, diff ciblé de quatre fichiers, aucun point critique/majeur ouvert. Validateur du principal : zéro erreur, zéro avertissement, sortie 0.

Audit : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/2026-10-07/session-2026-10-07-spec141-01
Résultats : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/validation/results.json

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/plan.md#extrait-7).
