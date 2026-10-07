# Vérifications prévues — SPEC143

Frontend : `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/`.
Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/`.

## Tests ciblés

Depuis le dossier frontend ci-dessus, utiliser le binaire installé, sans réinstallation :

```bash
./node_modules/.bin/vp test run --project unit src/components/chat/MessagesTimeline.logic.test.ts src/components/chat/MessagesTimeline.test.tsx
```

La présence du binaire et la baseline143 doivent être vérifiées avant les tests RED. Le succès141 n'est pas une preuve143. Consigner les commandes réelles et résultats.

## Recette isolée

Fixtures pour chaque famille d'entrée reconnue, sorties MCP confirmées, nom/ID/absence de destinataire, erreur, `completed`, payload inconnu et outils/messages ordinaires. Vérifier fond/bordure/padding, dépliage au clavier, contenu exact, copie, actions et absence de nouveau panneau Sources. Préserver l'ancrage et les interactions de groupe141. Contrôler affichage étroit et thèmes disponibles sans toucher à l'application active.

Aperçu isolé autorisé par l'utilisateur, à coordonner avec le principal. Aucun restart de l'application active, commit, déploiement ou mission réelle. Exploration reçue, gate PASS lu par le principal et tâches autorisées. Vérifier les valeurs JSON sorties et leur rendu natif, pas leurs octets de sérialisation.

## Cache de l'aperçu et état final

Ajouter toutes les fixtures dans le clone avant la première connexion navigateur. Modifier ensuite directement la projection SQLite sans journal d'événement ne la rejoue pas dans le cache IndexedDB déjà chargé. Si un cache doit être corrigé, identifier uniquement la clé de l'environnement/thread du clone et coordonner cette action avec le principal ; aucun effacement global ou environnement actif. Pendant la recette143, seule la clé du clone `34f5965e-48b1-408c-bcf8-2ae9c29c04d8` a été touchée.

Recette PASS,346 tests PASS, format/lint/types/build PASS ; audit diff grade A sans résidu. Clone arrêté proprement, captures et données conservées. Fonctionnalité non installée/non activée ; aucun commit143. Seconde comparaison Converge finale consignée dans le journal.

## Complément US4 — Preuves finales distinctes

Révision gelée392 PASS (283 logique/109 UI), soit46 cas ajoutés au socle346. Format/lint/types/build/diff PASS,22 warnings lint existants. Recette native isolée : gauche/repli, Enter/Space et focus après deux RAF, note/lien conservés, citation DOM sur texte répété, conteneur320px sans débordement. Ce dernier contrôle n'est pas une recette de viewport mobile.

Contrat conservateur : terminé/préfixe strict seulement, noms antérieurs même fil, notes canonique simple/gras après frontière explicite. Tout `[` hors fence dans une réponse mixte, HTML hors code ou frontière ambiguë garde le natif complet. Le corps agent mixte reste monté sous `display:none` pour les citations ; copie originale et métadonnées inchangées. Aucun nouveau panneau Sources ni bénéfice de calcul revendiqué.

Recette détaillée : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/ui-recipe-US4.json`. Clone arrêté proprement à11:35:22, fixtures et onglet masqué conservés. Convergence principale CONVERGED à11:39:54 sur tâches12/13 byte-identiques ; clôture T013 ensuite,13/13. Aucune installation, activation, commit, fusion, push ou restart T3.
