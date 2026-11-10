# Lot 16 - Écrans Android séparés (T143-T150)

But : `MainActivity.kt` (1154 lignes) est découpé en un fichier par écran, chacun sous 300 lignes. `MobileViewModel.kt` (1074 lignes) perd ses fonctions pures (D15). Aucun écran ne change d'apparence ni de comportement. Mesures : `releves.md` section 13.
Dossier : `android/app/src/main/java/app/cartae/bridget/` (noté `K/` ci-dessous). Branche : `session-260-lot16-android`. Worktree : `.worktrees/260-lot16-android`. Aucun Rust. **Aucun APK publié, aucune installation sur appareil.** Code Android modifié : « activation en attente » (R8).
Un déplacement de code sans changement visible est une correction technique : pas de diaporama de preuves (règle des preuves de correction visible).

## Méthode de déplacement (valable pour chaque tâche)

1. Dans `K/MainActivity.kt`, repérer par **nom** (`rg -n 'fun <Nom>\(' K/MainActivity.kt`) les fonctions à déplacer. Un composant s'étend jusqu'à la fonction suivante de colonne 0.
2. Créer le nouveau fichier avec `package app.cartae.bridget`, puis les `import` nécessaires (voir 4), puis les fonctions **copiées sans modification**, hors un seul changement : `private fun` devient `internal fun` (les autres fichiers du même module doivent les voir). Une annotation `@Composable` qui précède la fonction reste attachée à elle.
3. Supprimer les fonctions de `MainActivity.kt`.
4. Imports : compiler (commande Gradle ci-dessous). Chaque erreur « Unresolved reference: X » demande l'import de X : le copier depuis la liste d'imports de `MainActivity.kt` (qui garde les siens). Un import devenu inutile dans `MainActivity.kt` produit seulement un avertissement : le supprimer à la fin du lot (T150).
5. Compiler : `cd android && ./gradlew --offline :app:compileDebugKotlin`. Les noms exacts des tâches se confirment par `./gradlew tasks --all --offline | rg -i 'compileDebugKotlin|testDebugUnitTest'`. Si Gradle exige le réseau et refuse `--offline` : noter l'erreur exacte, arrêter le lot (R4), ne pas installer de dépendance.
6. Commit par fichier : `refactor(android): Deplacer <écran> dans son fichier`.

## T143 - Constat et outil

```sh
cd android && ./gradlew --version --offline 2>&1 | tail -5; ./gradlew tasks --all --offline 2>&1 | rg -i 'compileDebugKotlin|testDebugUnitTest' | tee "../$V/lot16-taches-gradle.txt"; cd ..
wc -l android/app/src/main/java/app/cartae/bridget/MainActivity.kt android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt
```

Écrire les chiffres dans le journal. Passage de référence : `cd android && ./gradlew --offline :app:testDebugUnitTest 2>&1 | tail -30 | tee "../$V/lot16-base-tests.txt"` : la liste des tests qui échouent avant le lot (ligne de base Android). Si Gradle échoue pour une raison d'environnement (JDK, SDK absent) : écrire la cause exacte, **arrêter le lot** : sans compilation, aucun déplacement n'est vérifiable.
Ne pas commiter `android/build/` ni `.gradle/` (ignorés par Git).

## T144 - `Texts.kt` : fonctions de texte

Déplacer, dans `K/Texts.kt` : `sentinelReason`, `sentinelStage`, `eventText`, `choiceText`, `commandState`, `stateText`, `recoveryText`, `errorText`, `dateText`, `updateStage`, `hostUpdateStage`. Ce sont des fonctions de traduction de codes en texte. Ne pas toucher aux ressources `strings.xml`.

## T145 - `ConversationScreen.kt`

Déplacer `Conversation` (228 lignes), `activityKey`, `isToolActivity`, `ToolGroup`, `ToolActivity`, `hasMessageText`. Le fichier visé reste sous 300 lignes.

## T146 - `SettingsScreen.kt`

Déplacer `Settings` (135 lignes) et `Confirm`.

## T147 - `AgentDirectoryScreen.kt`

Déplacer `Welcome`, `AgentDirectory`, `ProjectLabel`, `ProjectAvatar`. Vérifier d'abord que `ProjectPresentation.kt` (15 lignes) ne définit pas déjà un doublon : `rg -n 'ProjectLabel|ProjectAvatar' K`.

## T148 - `Panels.kt`

Déplacer `FilePanel`, `TaskPanel`, `Questions`, `Outbox`. Vérifier qu'aucune n'existe déjà ailleurs : `rg -n 'fun (FilePanel|TaskPanel|Questions|Outbox)\(' K`.

## T149 - `MobileViewModel.kt` : fonctions pures seulement (D15)

Défaut de D15 : extraire **seulement** les fonctions pures ; documenter le reste.
1. Lister les fonctions de la classe : `rg -n '^\s*(private |internal )?fun ' K/MobileViewModel.kt | wc -l` (environ 95).
2. Une fonction est pure si elle (a) ne lit et n'écrit aucun champ de la classe (aucun `this`, aucun `_state`, aucun `viewModelScope`), (b) n'a aucun effet (réseau, fichier, journal), (c) donne le même résultat pour les mêmes arguments. Critère de repérage rapide : le corps n'utilise que ses paramètres.
3. Déplacer les fonctions pures, **au plus 15**, dans `K/MobileViewModelPure.kt` sous forme de fonctions de colonne 0 (`internal fun`), et remplacer leurs appels dans la classe par l'appel direct (même nom, même arguments). Si un nom entre en collision : arrêter cette fonction.
4. Écrire au sommet de `MobileViewModel.kt` un commentaire de 4 lignes : la classe garde l'état partagé ; les fonctions pures sont dans `MobileViewModelPure.kt` ; le reste (état et effets) n'est pas découpé (décision D15, défaut).
Statut `tranchée` pour D15 avec une autre portée : appliquer la réponse de François, écrire laquelle.
Commit : `refactor(android): Extraire les fonctions pures du modele de vue`.

## T150 - Porte du lot

1. Retirer les imports devenus inutiles de `MainActivity.kt` (compiler ; chaque avertissement « unused import » ou « Unused import » à supprimer).
2. `wc -l K/MainActivity.kt` : moins de 300 lignes. Chaque nouveau fichier : `wc -l` moins de 300 (sinon le noter ; `Conversation` seule fait 228 lignes).
3. Compilation : `cd android && ./gradlew --offline :app:compileDebugKotlin` : succès.
4. Tests : `./gradlew --offline :app:testDebugUnitTest` : mêmes échecs que `lot16-base-tests.txt` (comparer les listes d'échecs).
5. Contrôle d'intégralité : `git diff origin/main --stat -- android | tail -1` ; le nombre de lignes supprimées de `MainActivity.kt` égale à peu près le nombre de lignes ajoutées dans les nouveaux fichiers (écart attendu : en-têtes et imports).
6. Bilan : lignes avant/après par fichier, fonctions pures extraites, « activation en attente (aucun APK publié) ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Gradle ne compile pas avant la première modification : arrêt du lot (la ligne de base manque).
- Un composant déplacé change d'aspect : impossible sans modification du corps ; annuler le fichier (`git checkout -- android`) et supprimer le nouveau fichier avec `rm <chemin exact>`.
- Interdit : ajouter une dépendance Gradle, modifier `build.gradle.kts`, `AndroidManifest.xml`, ou un fichier de ressources.
