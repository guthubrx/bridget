# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/quickstart.md](../../../../../../specs/145-bridget-thread-viewer/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget isolé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`.
T3 isolé : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`.
Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/`.

<a id="extrait-2"></a>

## Extrait 2

Employer les scripts de test et binaires existants vérifiés. Créer des fixtures temporaires pour daemon, SQLite et connexions T3. Ne pas connecter cette recette au daemon actif. L'aperçu isolé est autorisé par l'utilisateur le2026-10-07. Aucun redémarrage, installation ou publication n'est autorisé.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

Vérifier aussi exit3 pour daemon absent ou namespace indisponible : T3 affiche unavailable sans parser stderr. Exit2 continue à porter le JSON métier inchangé. Ce statut opérationnel ne crée aucun nouveau code d'erreur métier.
