# Spécification 120 - Se protéger d'une seconde application T3

## Fiche synthèse
Spec: 120-garde-doublon-t3 | Statut: Implemented | Priorité: P1 | Date: 2026-09-25
Branche: session-120-garde-doublon-t3 | Suite de l'incident du 25/09 05:18Z, demandée par l'utilisateur.

## Problème observé
Le 25/09 à 05:18:48Z, un agent coordinateur a ouvert `/Applications/T3 Code (Alpha).app` par
automatisation d'interface (`getApp`) pendant un diagnostic. Cette application partage
`~/.t3/userdata` avec « T3 Code (Local) ». En 11 secondes, elle a marqué en échec les tours en
cours de deux workers (« Provider session did not survive a server restart ») et laissé leurs
nouveaux tours en attente, alors que les processus fournisseurs travaillaient encore ; le pont ne
pouvait plus relier leurs réponses. En se fermant, elle a effacé `server-runtime.json`, que l'agent
a dû réécrire à la main. Le pont l'avait détectée, mais seulement dans son journal.

## Exigences
- **FR-001** : à la détection d'une seconde application, une alerte visible (notification macOS)
  nomme le paquet `.app` à quitter ; les deux processus portent le même nom.
- **FR-002** : si `server-runtime.json` manque ou désigne un serveur mort alors que notre serveur
  répond, le pont rétablit notre déclaration telle que T3 l'avait écrite ; une déclaration valide,
  même étrangère, n'est jamais écrasée.
- **FR-003** : la skill Bridget interdit aux agents d'ouvrir une autre application T3 et donne la
  voie de diagnostic en lecture seule.

## Hors périmètre
- Fermer le doublon automatiquement : fermer un processus exige l'accord de l'humain.
- Empêcher le lancement : l'application officielle ne peut pas être modifiée (signature) ; la
  déplacer hors de `/Applications` relève d'une décision de l'utilisateur.

## Critères de succès
- **SC-001** : test de restauration (absent, serveur arrêté, étranger valide, serveur mort).
- **SC-002** : `app_bundle` rend `T3 Code (Local).app` pour le processus réel de l'application.
- **SC-003** : recette complète verte.
