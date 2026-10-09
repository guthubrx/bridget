# Recette prévue — SPEC145

Date : 2026-10-07. Procédure à exécuter après implémentation ; aucun résultat présumé.

## Environnement

Bridget isolé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`.
T3 isolé : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`.
Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/`.

Employer les scripts de test et binaires existants vérifiés. Créer des fixtures temporaires pour daemon, SQLite et connexions T3. Ne pas connecter cette recette au daemon actif. L'aperçu isolé est autorisé par l'utilisateur le2026-10-07. Aucun redémarrage, installation ou publication n'est autorisé.

Rust : utiliser `/Users/moi/.cargo/bin/cargo` directement. Fixer CARGO_TARGET_DIR à un dossier privé145. Le rejeu frais attesté emploie `/tmp/bridget145-cache-check.9Uc6FY`. Pour un futur contrôle frais, créer un nouveau dossier privé avec `mktemp -d /tmp/bridget145-target.XXXXXX`, puis employer son chemin absolu comme CARGO_TARGET_DIR. Ne pas modifier PATH pour sélectionner Cargo. Ne pas utiliser des artefacts du target partagé comme preuve145 après un essai de baseline. Un résultat zéro test ne valide aucune sélection.

## Scénarios

1. Préparer deux agents liés à deux conversations T3 du même projet, trois fils aux appartenances différentes et un fil hors projet. Ajouter au moins trois pages d'historique avec Unicode, espaces, lignes blanches, code et types variés.
2. Vérifier list/show/history par la CLI inspect Client. Vérifier le rôle, la capacité négociée, l'identité serveur et les seuls champs humains projetés. Tester aussi agent dormant avec lien primaire présent.
3. Forger une identité, une racine, un fil et une liaison ambiguë. Vérifier chaque refus. Simuler une déconnexion et l'absence de capacité sur un ancien daemon. Ne pas accepter une identité `last_known`.
4. Comparer l'état des membres, reçus, ACK, réveils, files d'émission et missions avant/après chaque lecture et refus. Compter zéro dispatch et zéro appel modèle.
5. Tester les RPC T3 avec connexion authentifiée et faux ProcessRunner. Vérifier argv, racine issue du projet, délai6s, sortie256 Kio et projection128 Kio. Tester sorties invalides, tronquées et versions inconnues.
6. Tester le panneau natif : icône neutre, ouverture, fermeture, sélection de plusieurs fils, chargement des pages ASC, corps exact et copie source. Vérifier menus et panneaux existants.
7. Chercher localement, vérifier zéro appel distant et libellé de portée. Rafraîchir manuellement, vérifier seules lectures autorisées et absence de boucle.
8. Produire une réponse A tardive après passage B, fermeture et révocation. Vérifier qu'aucune donnée A ne revient. Tester chargement, vide, refus, daemon absent, délai et incompatibilité.
9. Utiliser les noms accessibles dans les tests d'interaction. Vérifier clavier, focus visible et annonces de chargement. Ne pas déduire conformité complète d'un simple test DOM.
10. Exécuter tests ciblés, format, lint, typecheck et build selon les scripts réels. Conserver commandes, exit codes, compteurs et limites de preuve. La recette native utilise seulement la surface isolée désormais autorisée et doit être observée.

Risques à reproduire : daemon absent sans autostart ni création de namespace ; inspect intercepté avant initialize_process et resolve_current_identity ; lecture et refus avant collect_closed_attach_views ; racine Git différente du workspace traitée par resolve_communication_project ; sortie métier exit2 décodée sans exposer stderr. Les comparaisons portent sur les données métier, pas sur les octets SQLite/WAL.

Vérifier aussi exit3 pour daemon absent ou namespace indisponible : T3 affiche unavailable sans parser stderr. Exit2 continue à porter le JSON métier inchangé. Ce statut opérationnel ne crée aucun nouveau code d'erreur métier.

## Clôture

La réussite exige des preuves SC145 distinctes pour backend et frontend. Une réussite documentaire ne prouve aucune installation. Commit, fusion, push, déploiement et redémarrage restent soumis à une autorisation distincte.
