# Plan 172 — parcours distant complet
Statut : en cours. Périmètre élargi explicitement par l'utilisateur le 18 octobre 2026.

## Référence T3 vérifiée
- /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/subagentProjection.ts :
  l'enfant hérite du projet et de l'espace du parent.
- /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ThreadLaunchService.ts :
  préparation Git, enregistrement du dossier, puis libération du lancement.
- /Users/moi/11.Repositories/t3code/apps/server/src/sourceControl/SourceControlRepositoryService.ts :
  clone standard, refus d'écraser une destination occupée.

T3 ne transfère pas lui-même un projet local vers Cartae. Bridget doit ajouter
ce transport, en conservant l'héritage, les contrôles Git et l'ordre de lancement.

## Décision
Réutiliser la fédération SSH installée et vérifiée. Le maître prépare le projet
depuis la racine attestée du parent. Un export Git transportable (bundle) conserve
le commit exact, y compris sans dépôt publié. La destination importe les objets,
crée le dépôt absent, puis publie la correspondance dans son registre. Le maître
publie ensuite la même correspondance. Le worker relit déjà ce registre à chaud.
Aucun profil fournisseur, clé SSH ou droit du parent n'est copié.
Les répertoires existants ne sont jamais remplacés. Un rejeu reprend le même
projet. La sonde authentifiée habituelle et l'admission restent obligatoires.

## Ordre d'exécution
1. Contrôler l'identité et les droits du parent, le dossier et le commit source.
2. Résoudre la correspondance existante ou préparer une destination gérée.
3. Réutiliser le dépôt déjà prêt ; transférer les objets Git manquants.
4. Publier les correspondances après préparation réussie, avec écritures atomiques.
5. Exécuter la sonde et le routage existants ; créer l'espace demandé et lancer.
6. Conserver la tâche et son reçu au rejeu ; remettre le résultat au même parent.

## Validation avant livraison
- Projet déclaré, commit déjà présent ; aucune copie inutile.
- Projet Git neuf sans correspondance ni dossier distant.
- Commit local non publié et reprise après publication distante seule.
- Refus d'écraser un dossier étranger ; source hors parent ; lien symbolique.
- Identifiant explicite incohérent ; registre modifié concurremment ; panne SSH.
- Régressions du routage, du worktree et des droits.
- Recette réelle Cartae, statut terminal et résultat dans le fil parent.
La compilation du correctif partiel est conservée hors activation. Aucune
nouvelle livraison avant les contrôles du parcours complet.
