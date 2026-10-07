# Livraison SPEC140 — État vérifié et activation T3 à venir

Date :2026-10-07. Commit, fusion, push et installation autorisés explicitement. Le redémarrage T3 est également autorisé malgré les tours actifs. Cette autorisation ne prouve pas que le redémarrage a déjà eu lieu.

## Bridget — Livré et activé

Source54b815aa fusionnée dans main et poussée sur le remote github, avec la branche de session. Exécutable release Mach-O arm64 installé atomiquement dans /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.

Services relancés proprement par SIGTERM après identification des processus. Aucun agent managed/external actif lors de cette activation ;29 fils T3 étaient présents. État réellement observé après remise en service :daemon build54b815aad079,37 connexions ; pont T3 running avec29 fils. Ces contrôles prouvent le service en ligne, pas l'exécution de toutes les missions.

Les processus T3 PID62878/62954 et les identifiants de tours actifs sont restés inchangés pendant l'activation Bridget. T3 n'a pas été redémarré par cette phase.

Sauvegarde privée : /Users/moi/.cache/bridget-adoptions/spec140-20261007.lQLkje. VACUUM n'était pas utilisable avec le schéma legacy DQS. Le fallback SQLite .backup suivi de quick_check avec .dbconfig dqs_ddl on passe. La base source n'a pas été éditée. Les données et secrets de sauvegarde restent privés, hors Git.

## T3 — Sources livrées, paquet prêt

Source9706acbde6 fusionnée dans local/v0.0.45 et poussée sur le remote fork, avec la branche de session. La branche durable local-patch/bridget-headings-v0.0.45 est étendue par cherry-pickcc8fe08686 et poussée. Les cinq fichiers de correction sont identiques entre les deux branches :le correctif est conservé pour la reconstruction.

Paquet version0.0.45-local.140, commit source9706acbde648. Signature ad hoc validée ; il ne s'agit pas d'une notarisation Apple. SHA256 du paquet prêt :dab141939a4171c3b4e7169fc68346b498ce179c414f9e2e8ee6acdb42f89ba1.

À ce point de rédaction, le paquet T3 n'est pas installé. Aucune activation frontend ou santé après redémarrage n'est revendiquée.

## Activation T3 — Préparation autorisée

Le principal prépare un job externe pour terminer la livraison même si le redémarrage coupe son tour. Délai prévu45 secondes après lancement pour lui permettre de finir sa réponse. Arrêt propre de T3, sauvegarde du paquet et des données privées, remplacement contrôlé, relance et contrôle santé. Retour arrière prévu si le contrôle échoue. Les sauvegardes ne sont pas publiées.

Répertoire privé d'adoption : /Users/moi/.cache/t3-adoptions/spec140-20261007.jSfUyE. Sauvegarde de base :quick_check déjà PASS. Reçu final attendu : /Users/moi/.cache/t3-adoptions/spec140-20261007.jSfUyE/result.json.

Le lancement du job, l'installation et son résultat doivent être confirmés par ce reçu et les contrôles réels du principal. Ce document ne transforme pas une préparation en succès de déploiement. Aucun cleanup de sauvegarde, données ou autres travaux n'est effectué par cette rédaction.

## Preuves conservées

Les rapports d'audit versionnables sont conservés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/validation/audit/. Les résultats de développement n'ont pas été réécrits pour leur attribuer une installation inexistante.
