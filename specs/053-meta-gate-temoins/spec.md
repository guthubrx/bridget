# Spécification 053 — Méta-gate des témoins de présence

**Statut** : Prêt pour revue

**Tests** : 3/3 (100 %)

**Base gelée du lot** : `52173dc7d886e30d980332734e99e815bd092309`

**Objectif** : `f264237f-535b-477e-9769-6f8b817b3114`

## Problème mesuré

Le lot 050 a ajouté un commentaire au-dessus de
`daemon::presence_tests` : cette famille est couramment retirée des sélections
de revue parce que plusieurs de ses tests sont instables sous charge. Le
commentaire décrit correctement le risque, mais il n'a aucun comportement
exécutable et ne peut donc pas refuser une sélection incomplète.

Sur la base gelée, le binaire de tests de la bibliothèque `bridget-daemon`
liste 576 tests, dont 109 sous le préfixe exact
`daemon::presence_tests::`. Une sélection peut en omettre un, plusieurs ou la
famille entière tout en restant une commande Cargo valide.

## Propriété

Le méta-gate reliste la famille attendue depuis le binaire de tests complet,
puis reliste la sélection proposée et compare les noms exacts. Il refuse si un
seul témoin attendu manque. Une source complète vide ou une sélection vide est
inobservable et ne peut jamais produire un succès.

Le méta-gate ne décide pas si les témoins sont verts : il garantit seulement
qu'ils appartiennent à l'univers déclaré par le mesureur. Le résultat de leur
exécution reste une mesure distincte.

## Scénarios

### US1 — Sélection complète

Une sélection sans filtre contient les 109 témoins attendus. Le méta-gate
annonce les cardinaux de la famille et de l'univers sélectionné, puis réussit.

### US2 — Famille amputée

Une sélection qui retire exactement un témoin reste non vide. Le méta-gate la
refuse, annonce le cardinal manquant et nomme le témoin absent.

### US3 — Instrument muet

Une sélection qui ne liste aucun test est refusée comme inobservable. Elle
n'est jamais assimilée à une famille complète de zéro témoin.

## Exigences fonctionnelles

- **FR-5301** : la source attendue est obtenue par `cargo test -p
  bridget-daemon --lib -- --list`, sans manifeste recopié à la main.
- **FR-5302** : la famille est définie par le préfixe exact
  `daemon::presence_tests::` sur les noms de tests listés.
- **FR-5303** : la sélection est relistée avec les arguments libtest fournis au
  méta-gate.
- **FR-5304** : chaque nom attendu absent de la sélection produit un refus non
  nul et est écrit sur la sortie d'erreur.
- **FR-5305** : zéro témoin attendu ou zéro test sélectionné produit un refus
  nommé, jamais un succès vide.
- **FR-5306** : le diagnostic de succès annonce le nombre de témoins présents,
  attendus et le cardinal total de la sélection.
- **FR-5307** : le lot ne modifie ni le comportement des tests de présence, ni
  leurs skips/ignores, ni le transport Bridget.

## Critères de succès

- **SC-5301** : une sélection complète rend 109/109 sur la base gelée.
- **SC-5302** : retirer exactement un nom attendu rend un refus qui nomme ce
  test, alors que les 108 autres restent sélectionnés.
- **SC-5303** : une sélection de cardinal zéro rend un refus
  `sélection inobservable`.
- **SC-5304** : le banc exécutable annonce trois scénarios, leurs états et un
  compte final passé/échoué/ignoré.

## Hors périmètre

- Rendre déterministes les tests de `daemon::presence_tests`.
- Transformer leurs rouges connus en ignores ou les déplacer.
- Intercepter toutes les commandes `cargo test` exécutées hors du méta-gate.
- Modifier les règles de sélection d'un service CI absent du dépôt.
