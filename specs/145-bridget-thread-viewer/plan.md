# Plan technique — SPEC145

Date : 2026-10-07. Statut : Implemented, 20/20 tâches vérifiées. Audit validé et Converge2 CONVERGED à 16:24:21 UTC. Clôture documentaire hors Converge. Non installé, non activé.

## Contexte et contraintes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-1).

Le préflight et les skills Specify, Plan, AuditExisting, Tasks, Analyze et Implement ont été lus par le principal. Les templates et helpers locaux attendus sont absents. Appliquer leur protocole documentaire sans prétendre une exécution de helper ni installer un outillage.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-2).

## Architecture retenue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-3).

Ce canal local repose sur la confiance existante du compte système. Le fait de liaison ne revendique pas une authentification cryptographique entre deux processus du même compte. Le navigateur ne peut pas choisir l'agent. L'index de liaison est en mémoire et nettoyé à la déconnexion. Aucune table ni migration n'est ajoutée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-4).

La CLI intercepte inspect dans `run()` avant `initialize_process` et `resolve_current_identity`. Utiliser `Namespace::from_environment` sans prepare et `DaemonConnection::connect` direct. Un daemon absent produit un refus, sans autostart ni création de namespace.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-5).

## Identité et refus

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-7).

Les liaisons absentes, ambiguës, révoquées ou d'un autre projet produisent un refus explicite. Aucun inventaire global des agents ne sert de repli. La présence de deux fournisseurs dans un même projet ne donne pas accès aux fils de l'autre agent. Un ancien daemon sans capacité négociée produit un état incompatible, jamais un essai de `read` ou d'identité agent.

## Pagination et projection

Liste : ordre UUID croissant, `after_thread_id` exclusif, limite20 par défaut et100 maximum. Historique : séquence croissante, `from_seq` positif, limite50 par défaut et200 maximum. Réutiliser la borne `snapshot_seq` sous `to_seq` pour les pages suivantes. Un rafraîchissement démarre une nouvelle lecture cohérente. Ces positions de lecture humaines sont distinctes des curseurs agents.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-8).

Le panneau montre l'ordre attesté. Un type legacy absent apparaît sous « Message ». Les types attestés sont History, Action, Blocker et Decision. Le corps source reste exact ; sa copie utilise la chaîne source. Les membres et noms sont disponibles, jamais déduits des corps. La liste les enrichit par SQL groupé, avec au maximum16 membres par fil.

## Intégration native

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-10).

À fermeture, changement de contexte ou refus, annuler les travaux en cours quand possible et invalider leur génération. Toute réponse tardive est ignorée. Vider les données d'un contexte quitté ou devenu non autorisé. Le bouton Rafraîchir relit la liste et le fil courant sans émission, ACK ni tour modèle.

## Validation et gates

Avant les tâches : spec et checklist lisibles ; contrat fermé ; inventaire des sources exactes ; audit de réutilisation avec verdict de conception PASS. Le principal doit accorder le gate avant création de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/tasks.md`.

Avant implémentation : Analyze compare spec, plan, contrat et tâches. Les tests RED couvrent autorisation, non-mutation, anciens daemons, bornes, texte exact et contexte tardif. Ils n'utilisent pas le daemon actif.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-11).

Exécuter ensuite format, lint, typecheck et build selon les scripts existants vérifiés. Conserver commandes, compteurs et échecs réels dans la recette. L'aperçu isolé est désormais autorisé par l'utilisateur ; son ouverture about:blank ne prouve pas la recette. Aucune recette sur application active. Revendiquer une recette native seulement après observation de la surface isolée.

## Réutilisation et risques

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/plan.md#extrait-12).

Risque principal : le rôle Client pourrait recevoir trop de capacités si `ThreadRequest` est réouvert globalement. L'enum humain séparé et la matrice exhaustive du daemon empêchent cette dérive. Deuxième risque : le changement de contexte UI pourrait publier une réponse ancienne. Les clés et générations explicites doivent le prévenir et les tests le reproduire. Troisième risque : une page enrichie pourrait dépasser sa borne. La sérialisation et le décodage doivent appliquer leurs plafonds avant publication.
