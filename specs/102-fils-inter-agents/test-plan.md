# Recette détaillée 102 — tests à implémenter, non exécutés

Oracle principal : compter les entrées persistées, destinataires d'injection,
pages transmises, appels de faux fournisseur et curseurs. Ne pas confondre un
mock « send appelé » avec une injection réelle, ni absence de log avec silence.

## Organisation prévue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/test-plan.md#extrait-1).

Une même propriété centrale peut être testée au niveau unitaire puis une fois
aux frontières ; ne pas copier toute la matrice sur chaque couche. Les tests
autorisés manipulent seulement données synthétiques, répertoires temporaires,
horloge injectable et processus qu'ils ont créés.

## Matrice obligatoire

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/test-plan.md#extrait-2).

## Scénarios Gherkin de référence

À transcrire dans le fichier .feature avant les tests Rust ; ils ne sont pas
exécutables dans l'état documentaire actuel.

```gherkin
# language: fr
Fonctionnalité: Discuter entre agents sans réveiller tous les membres
  Scénario: Deux agents échangent dans un fil de quatre membres
    Étant donné un fil ouvert dont A, B, C et D sont membres
    Quand A publie vingt contributions en sollicitant seulement B
    Et B répond en sollicitant seulement A
    Alors C et D ne reçoivent aucune sollicitation
    Et aucun appel fournisseur n'est déclenché chez C ou D par ces contributions
    Et les contributions restent consultables par C et D

  Scénario: Une lecture interrompue ne fait pas perdre une page
    Étant donné que B a confirmé les entrées jusqu'à 10
    Quand Bridget prépare pour B la page 11 à 20
    Et la réponse est perdue avant confirmation de B
    Alors le repère confirmé de B reste 10
    Et sa prochaine lecture restitue la page non confirmée

  Scénario: Une nouvelle mention survit à une ancienne confirmation
    Étant donné que B lit les entrées jusqu'à 20
    Quand A publie l'entrée 21 en sollicitant B
    Et B confirme la page finissant à 20
    Alors l'entrée 21 reste nouvelle pour B
    Et sa sollicitation n'est pas effacée par cette confirmation
```

## Mesures, traces et non-régression

SC-005 : 10000 entrées synthétiques de256octets,200 opérations séquentielles
lecture/publication de1Kio, participants16 max, daemon local sans fournisseur.
Mesurer durée avec horloge monotone ; publier machine/modebuild et p50/p95/max,
nombre de requêtes/entrées renvoyées. Le seuil est un critère de recette sur le
poste déclaré, pas une promesse universelle de performance. Préserver un flux
de20 DM témoins et constater leurs reçus pendant le test, pas seulement après.

Crash tests : points de coupure contrôlés, fermeture/reprise de composants de
test ou erreur injectée, pas SIGKILL d'un processus partagé. Cas de concurrence
avec barrières/canaux, délais d'attente bornés ; pas sleep arbitraire comme preuve.

Les tests de synthèse vérifient l'absence de génération automatique et la recette
de provenance, pas la qualité universelle d'un modèle. Aucun « coûttokens réduit
deX% » déclaré sans mesure fournisseur spécifique. Zéro appel induit chez les
non-ciblés est en revanche vérifiable exactement avec le faux fournisseur.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/test-plan.md#extrait-3).

## Liaison scénario → test Rust (T003)

Fichier Gherkin : tests/features/102-fils-inter-agents.feature ; chaque scénario
porte les tags `@Vnn` et `@spec102_vnn_<slug>`. Sauf mention contraire, le test
vit dans crates/bridget-daemon/tests/spec102_threads_test.rs.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/102-fils-inter-agents/test-plan.md#extrait-4).
