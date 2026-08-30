# Analyse transversale du programme 065-067

**Date**: 2026-08-29
**Verdict**: PASS documentaire après corrections RC8, aucune implémentation
autorisée à ce stade

**Contre-revue**: RC8 du 2026-08-30, écarts de cible et de contrats fermés dans
les artefacts; revalidation obligatoire sur la future tête `main` propre

## Progressivité

| Incrément | Autorité ajoutée | Dépend de | Utilisable seul | Rollback |
|---|---|---|---|---|
| SPEC-065 | identité métier Maicie et liaison runtime Bridget | SPEC-063 prouvée, SPEC-064 stabilisée | oui, backend host | désactiver la liaison ou ignorer `project_id` |
| SPEC-066 | cycle de vie d'un environnement Docker partagé | SPEC-065 implémentée et prouvée | oui, sans secret projet | bascule explicite vers host |
| SPEC-067 | approbation de profils et résolution d'extensions/secrets | SPEC-066 implémentée et prouvée | oui, par projet inscrit | désactiver le profil puis recréer |

Chaque incrément produit une valeur autonome. Aucun incrément ne rend le suivant
obligatoire et aucune activation globale n'est prévue.

## Unicité des autorités

| Concept | Autorité unique | Consommateurs |
|---|---|---|
| `ProjectIdentity` | Maicie | utilisateur, délégations, Bridget par contrat |
| `ProjectBinding` | Bridget | admission des chemins, flotte, diagnostics |
| `ProjectRootPolicy` | configuration hôte Bridget | admission des racines 065 |
| `ProjectEnvironment` | Bridget | lifecycle, wrapper, supervision |
| `ProjectRuntimePolicyDefinition` | configuration hôte Bridget | image, UID/GID, limites et ABI 066 |
| `ProjectProfile` et approbation | Maicie | utilisateur, Bridget par proposition approuvée |
| profil résolu et attestations runtime | Bridget | lifecycle, environnement, diagnostics |
| `ProjectResourceCatalog` | configuration hôte Bridget | résolution des ExtensionRef et SecretRef 067 |
| vérité de mission et relations parent-enfant | contrats stabilisés par SPEC-064 | Maicie et Bridget selon leurs frontières |
| incidents runtime délégués et leur acquittement | contrats stabilisés par SPEC-068 | Bridget, wrappers et projections Maicie |

Il n'existe ni second registre de projet, ni seconde flotte, ni autorité agentique
sur une décision durable.

## Absence de chevauchement

- SPEC-065 ne crée ni conteneur, ni secret, ni gestionnaire d'extensions.
- SPEC-066 ne définit ni valeur secrète, ni profil métier, ni téléchargement de
  plugin.
- SPEC-067 ne redéfinit ni identité projet, ni transport inter-agent, ni cycle de
  vie générique du conteneur.
- SPEC-063 reste l'autorité de l'interruption et du pilotage humain.
- SPEC-064 reste l'autorité du plan de contrôle, des soumissions, des capacités
  fournisseurs et de la relation parent-enfant.
- SPEC-068 reste l'autorité de la remontée, de la persistance, du rejeu et de
  l'acquittement des incidents runtime délégués; 065 à 067 ne créent aucun
  canal concurrent.

## Invariants communs

- Maicie et Bridget restent sur l'hôte et déterministes.
- Un seul conteneur est partagé par les agents d'un projet.
- Les worktrees sont utilisés pour les travaux Git concurrents, pas pour isoler
  systématiquement chaque agent.
- Le code, les worktrees et l'état durable restent sur l'hôte.
- Les projets existants restent sur host sans inscription et activation
  explicites.
- Une panne Docker ne déclenche jamais un fallback host silencieux.
- Le runtime ingress est privé et attesté par projet et génération.
- Le socket du wrapper est explicite et ne dépend jamais du `HOME`; le
  conteneur utilise un HOME/XDG privé sous son unique state root.
- L'UID/GID conteneur est numérique, non-root, configuré et prouvé compatible
  avec les montages et secrets privés avant activation.
- Spawn et lifecycle Docker sont sérialisés par réservation et epoch.
- Un rebind invalide tout profil approuvé avant lecture ou montage.
- Tout changement de backend, version ou digest de politique runtime invalide
  également le profil avant lecture ou montage.
- Une référence de ressource est résolue uniquement par le catalogue hôte
  fermé; une SecretSourceStamp détecte toute mutation hors rotation dans le
  modèle de menace local coopératif.
- Les agents actifs terminent sur leur ancienne génération après rebind; aucune
  nouvelle admission ne traverse une génération divergente.
- Le projet est la frontière de confiance v1; ses agents peuvent lire les
  secrets projet.
- Les liaisons parent-enfant, événements de liaison et incidents runtime
  délégués portent le même `ProjectReference` que l'exécution source, y compris
  après persistance, redémarrage, rejeu et acquittement.

## Éléments volontairement différés

- conteneur par agent;
- stack Maicie/Bridget par projet;
- Kubernetes, gVisor ou ordonnanceur multi-hôte;
- Vault ou broker dynamique de secrets;
- secret visible par un seul agent d'un projet;
- filtrage egress par destination;
- mémoire globale writable;
- téléchargement automatique de plugins.

Ces éléments exigent une nouvelle preuve de besoin et une spec séparée. Ils ne
doivent pas être introduits comme sous-tâches implicites des SPEC-065 à 067.

## Gate documentaire

- Les trois specs restent `Draft`.
- Les 104 tâches restent décochées et les trois journaux d'implémentation
  indiquent explicitement `non commencée`.
- Les contrats sont des contrats de conception, pas du code exécutable.
- Le passage à l'implémentation exige d'abord la preuve de SPEC-063, la
  stabilisation de SPEC-064, la présence des contrats SPEC-068, une validation
  humaine explicite de SPEC-065 et un nouveau worktree créé depuis une tête
  `main` propre au minimum égale à `d589b24`.
- L'absence de `.specify` ou de l'outil `specify` ne bloque pas l'implémentation
  et ne doit déclencher ni installation ni mise à jour de SpecKit. Si `main`
  avance après `d589b24`, les trois audits de réutilisation doivent être rejoués.
