# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/adversarial-review-plan.md](../../../../../../specs/145-bridget-thread-viewer/adversarial-review-plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`.
T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`.
Référentiel : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/spec.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/contracts/human-thread-view.md` et preuves145 distinctes des anciennes sessions.

<a id="extrait-2"></a>

## Extrait 2

1. Autorisation : forger conversation, agent, racine et fil ; essayer autre projet/fournisseur ; supprimer ou dupliquer une liaison ; faire varier source T3 et Git ; déconnecter pendant lecture. Vérifier refus fermé, root canonisée et aucune utilisation last_known.
2. Non-mutation : ouvrir/fermer, list/show/history, recherche, refresh et refus. Vérifier handlers avant maintenance et CLI avant init/autostart. Comparer données métier et compteurs d'ACK, dispatch, réveil, reçu, mission et appel modèle.
3. Protocole : ancien daemon, capability absente, version inconnue, action agent injectée, JSON/UTF-8 invalide, sortie trop grande, exit2, timeout et stderr sensible. Vérifier enum fermé, argv sans shell et caps.
4. Pagination : trois pages au moins, from/to/limit invalides, ordre ASC, snapshot stable, ajout entre pages et refresh. Vérifier corps source exact, membres/noms groupés bornés et absence de trous/doublons sur données stables.
5. Contexte UI : A → B → A, fermeture, refresh répété, changement de fil, révocation et réponses désordonnées. Vérifier générations, vidage des données et aucune publication tardive.
6. Natif et accessibilité : icône b neutre, store/migration, menus/tabs, clavier/focus, recherche portée chargée, erreurs/retry, Sheet/panneau étroit et route Pull requests. Examiner l'interaction observée ; un snapshot seul ne suffit pas.
