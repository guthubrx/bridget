# Plan SPEC169 — Priorité Bridget depuis adaptateur de bureau

Date :2026-10-15. Implémentation14/14, P2 CONVERGED avec0 ajout. Pipeline :Livré avec conservations justifiées. Installation, activation unique, reprise, consigne ON, HTTP et recette native sont prouvés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md. Les checkpoints antérieurs ci-dessous restent historiques.

Spécification : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/spec.md

## Résultat attendu et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-2).

## Contexte et lieux de travail

## Architecture retenue

### 1. Un réglage et un fait de lecture fiable

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-6).

### 2. Une préférence qui ne porte aucun droit

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-10).

Les attestations et vérifications existantes restent seules autorités. Une vraie rotation d'identité conserve son cycle166. Un changement de préférence ne simule pas cette rotation.

### 3. Un seul endroit pour composer les consignes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-12).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-13).

Le montage ou un catalogue antérieur ne prouve pas l'authentification future du daemon. La consigne ON demande `bridget_auto_delegate`. Si le catalogue est paresseux, une seule recherche d'outil bornée par le mécanisme existant peut le rendre accessible avant de conclure à son absence. Sans outil ou découverte disponible, annoncer le blocage. Aucune nouvelle sonde HTTP de santé n'est ajoutée par tour.

La consigne ne promet pas l'obéissance absolue du modèle. Les refus et la non-duplication restent contrôlés par Bridget. L'absence de réponse après admission demande un suivi ou un rejeu avec les références existantes, jamais une autre délégation d'hôte.

### 4. Appliquer les changements aux tours et aux raccords existants

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-14).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-15).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-16).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-17).

Les références détaillées figurent dans l'audit de réutilisation : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/reuse-audit.md. Les variantes qui partagent ces compositeurs sont couvertes par leurs chemins effectifs, sans annonce de montage supplémentaire.

### 5. Garder le moteur et son suivi existants

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-18).

### 6. Corriger le prérequis causal de reprise du gros fil

ROOT a constaté sur le fil long l'erreur `champ response limit` : la réponse complète fait33 868 206 octets, au-delà du plafond16Mio. `visibleTurnItems` en représente environ27,5Mo redondants et n'est pas consommé par Bridget. La simulation ROOT qui conserve tous les champs utiles et rend cette seule liste vide donne6 372 704 octets. Cette simulation ne prouve pas encore le correctif installé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-19).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-20).

T012 peut être codée en parallèle par un second worker propriétaire des seuls fichiers HTTP et de leur banc. T001–T010 restent séquentielles chez le principal. Le second ne lance aucun contrôle ; le principal garde l'unique permis de tests/types/lint lourds. T011 attend toujours T010 et T012. Couverture et dépendances inchangées.

## Ordre de réalisation et groupes de preuves

Après contre-revue et audit de réutilisation PASS, générer les tâches d'implémentation. Ce plan ne les génère pas.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-21).

Bancs ciblés existants : réglages serveur, instructions partagées, instructions développeur Codex, driver MCP Codex, adaptateur Claude, gestionnaire de sessions et Lineage149. Ajouter seulement les cas manquants dans ces groupes fonctionnels. Aucun test de simple câblage ni fichier de test par fichier de code n'est demandé.

Le binaire Pi est hors PATH dans l'environnement constaté. Le contrat privé Pi est donc vérifié par tests/fixtures ; aucun parcours Pi réel n'est attesté et aucun fournisseur supplémentaire ne sera installé pour ce lot.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-22).

## Installation, activation et retour arrière

ROOT exécute la livraison. Avant remplacement, relever les PID, dates de naissance, chemins chargés, versions et empreintes réels. Vérifier l'état durable des conversations et tâches. Préparer la sauvegarde de l'application installée et du fichier de réglages réellement résolu. Ne pas utiliser un ancien checkpoint comme mesure courante. Ne pas restaurer ni effacer une base de conversations pour faire une recette.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-23).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-24).

Après activation, vérifier le code effectivement chargé, la lecture ON et les nouvelles consignes. Reprendre une conversation existante par son cycle normal. Vérifier ses références, son identité courante et le suivi des tâches des deux moteurs. La recette de sous-tâche ne lance qu'un raccord et une identité existants, déclarés et vérifiés. Une capacité manquante se rapporte comme telle.

Si l'activation échoue, appliquer le retour arrière préparé de l'application et du réglage, puis vérifier la reprise. Conserver les bases, journaux et travaux vivants. Une correction ne supprime aucune preuve antérieure. L'installation seule ne clôt pas le jalon d'activation.

## Jalons du pipeline et statuts

Les14 tâches d'implémentation portent le code et ses preuves, dont T013/T014 ajoutées après R1. T011 dépend de T010 et T012 ; T013/T014 partent du gel T011 et précèdent le passage2. Les identifiants restent stables. Elles ne contiennent pas une condition circulaire exigeant leur propre futur commit. Les jalons distincts suivent la revue, l'intégration, le paquet, l'installation, l'activation, les reçus et le nettoyage. `Implemented` exige100% des tâches réellement terminées. Le pipeline n'est `Livré` qu'après les preuves de livraison et d'activation, dont la reprise du gros fil.

L'estimation initiale de 75–145 minutes publiée à 19:15 CEST précédait l'autorisation d'activation. ROOT publiera une estimation révisée après le découpage des tâches. Aucun délai futur n'est annoncé comme réalisé.

## Constitution, minimalisme et responsabilité future

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-25).

Potentiel minimalisme : 0 ligne de fonctionnalité nouvelle identifiée comme supprimable dans le plan. Aucun volume de code n'est encore mesuré. L'implémentation devra relire son diff pour retirer les branches, textes et états redondants.

La frontière durable est documentée une seule fois dans le contrat : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/contracts/delegation-preference.md. L'ADR169 y renvoie. Le contexte par tour ferme le transport sans MCP. Aucune inconnue de conception bloquante ne reste identifiée ; l'exécution des gates reste à prouver.

## Arbitrages documentaires après contre-revue R2

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-26).

C2 est conservé : absence/OFF garde la politique habituelle avec enveloppe de reset sans MCP, même au premier tour inconnu. Codex reçoit t3_code_orchestration et t3_code_runtime ; ordinal1 n'exclut pas import/reprise. Seuls6 986 octets OFF avec MCP sont byte-identiques. Contexte identique dédupliqué, coût restant non mesuré ;87 transcriptions adaptées, pas87 bugs produit. Pas de branche/cache supplémentaire ni de passage3.

Cache Claude blocked jusqu'à réouverture/changement de clé ou processus, message UI et reprise existante positive non prouvés restent explicites. Pi absent et75 erreurs TS non GREEN sont conservés. P2 et J1 restent clos ; J2 est partiel et FR017/SC003 réel/SC009/J3–J6 restent ouverts.

## Checkpoint historique après clôture J1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/plan.md#extrait-27).

## Bilan opérationnel

Une compilation du paquet et une activation seulement. Les tests source ne sont pas relancés après les seules mises à jour documentaires. Retraits et conservations du lot169 sont consignés dans /Users/moi/.cache/bridget-delivery169/final169.json ; aucun nettoyage étranger ni suppression de base.
