# Lot 5 - Aiguillage découpé (T054-T068)

But : réduire `handle_wrapper_message_with_peer` (5011 lignes avant le lot 4) à moins de 1500 lignes. Les 37 branches de 30 lignes ou plus deviennent des fonctions `on_xxx`. Les blocs `service_refusal` et `client_refusal` deviennent des fonctions. Mesures et listes complètes : `releves.md` section 3 (lire avec `sed -n '36,60p'`).
Branche : `session-260-lot05-aiguillage`. Worktree : `.worktrees/260-lot05-aiguillage`. Prérequis : lot 4 intégré dans `main` (il allège `daemon.rs`). Code exécutable modifié : « activation en attente » (R8).
Fichier : `crates/bridget-daemon/src/daemon.rs`. Les numéros de ligne de `releves.md` datent d'avant le lot 4. **Chercher par symbole**, jamais par numéro.
Interdits propres au lot : ne pas modifier `WrapperToDaemon` ni `DaemonToWrapper` (R6.1). Ne pas changer l'ordre des branches. Ne pas changer une ligne du corps déplacé.

## Méthode de déplacement (valable pour chaque branche)

La fonction contient `match msg {` (expression finale). Chaque branche a la forme :

```rust
WrapperToDaemon::Send { champ_a, champ_b, .. } => {
    CORPS
}
```

Étapes :

1. Dans `crates/bridget-daemon/src/daemon/wrapper_dispatch.rs`, ajouter la fonction :
   ```rust
   pub(super) fn on_send(champ_a: TypeA, champ_b: TypeB, /* contexte */) -> Option<DaemonToWrapper> {
       CORPS
   }
   ```
   CORPS est copié sans changement (couper-coller, pas de retouche). Les `return` du corps retournent désormais de `on_send` : même type `Option<DaemonToWrapper>`. Le `?` sur `Option` reste valable.
2. Les types des champs : lire la définition de la variante dans `crates/bridget-transport/src/protocol.rs` (`rg -n 'Send \{' crates/bridget-transport/src/protocol.rs`). La branche prend ses champs par valeur : les paramètres aussi.
3. Le contexte (état partagé, pair, canal d'émission...) : ce sont les variables que le corps utilise et que la fonction englobante a définies. Premier essai : paramètres de la signature de `handle_wrapper_message_with_peer` + les `let` qui précèdent `match msg {`. Lancer `C-CHECK-LEGER`. Chaque erreur « cannot find value `v` in this scope » : ajouter `v` en paramètre, avec le type donné par le compilateur (« expected ... found ... »). Répéter jusqu'à zéro erreur. Au plus 6 allers-retours par branche, sinon arrêter la branche (R4).
4. Dans `daemon.rs`, remplacer le corps de la branche par l'appel :
   ```rust
   WrapperToDaemon::Send { champ_a, champ_b, .. } => wrapper_dispatch::on_send(champ_a, champ_b, /* contexte */),
   ```
   Garder le motif de la branche tel quel.
5. `C-CHECK-LEGER` : code 0. Commit : `refactor(daemon): Extraire la branche <Nom> de l'aiguillage`.

`C-CHECK-LEGER` = `python3 scripts/build.py cargo check --offline --locked -p bridget-daemon --features bridget-daemon/test-support` (sans `--all-targets`, plus rapide). `C-CHECK` complet (avec `--all-targets`) à la fin de chaque tâche de tranche.
Un motif de branche avec une garde (`if ...`) ou plusieurs variantes (`A | B =>`) : ne pas la déplacer ; la noter, passer à la suivante.

## T054 - Préparer le module

1. Mesure de départ : `rg -n 'fn handle_wrapper_message_with_peer|pub fn get_status' crates/bridget-daemon/src/daemon.rs`. Longueur = ligne de `get_status` moins ligne de la fonction. Écrire le résultat dans `validation/lot05-depart.txt`.
2. Créer `crates/bridget-daemon/src/daemon/wrapper_dispatch.rs` avec pour seul contenu `use super::*;`. Imiter un module existant : `sed -n '1,15p' crates/bridget-daemon/src/daemon/user_input.rs`.
3. Dans `daemon.rs`, déclarer `mod wrapper_dispatch;` au bon endroit : chercher `mod user_input;` et ajouter la ligne juste en dessous.
4. `C-CHECK-LEGER` : code 0 (avertissement d'import inutile toléré). Commit : `refactor(daemon): Preparer le module de l'aiguillage`.

## T055 - `service_refusal`

Chercher `let service_refusal = {` dans la fonction (bloc d'environ 265 lignes). Appliquer la méthode : le bloc devient `let service_refusal = wrapper_dispatch::service_refusal(<contexte>);`. La fonction retourne la valeur finale du bloc : le type est donné par l'erreur du compilateur ou par l'usage de `service_refusal` plus bas. Commit : `refactor(daemon): Extraire le controle de refus de service`.

## T056 - `client_refusal`

Idem pour `let client_refusal = {` (environ 321 lignes). Idem pour le commit : `refactor(daemon): Extraire le controle de refus client`. Le bloc `attach_refusal` (28 lignes) reste en place.

## T057 à T066 - Les 37 branches, par tranches

Une branche = un commit. Une tâche = quelques branches. Après les branches d'une tâche : `C-CHECK`. Après T059, T062, T065 : `C-CRATE` (`bridget-daemon`).
Chercher chaque branche par `WrapperToDaemon::<Nom> {` dans la fonction. Taille en lignes indiquée.

| Tâche | Branches (nom : lignes) |
|-------|-------------------------|
| T057 | Send 429 ; ServiceRequest 264 ; RelaunchOrder 203 ; CancelRequest 203 |
| T058 | SpawnOrder 189 ; ServiceHello 124 ; ControlExecutionReported 112 ; DecommissionOrder 109 |
| T059 | ArtifactPublish 94 ; Register 89 ; GuichetReply 89 ; CoordinationSubscribe 88 |
| T060 | ClientHello 88 ; QueueEditRequest 81 ; ObservationRequest 78 ; Subscribe 75 |
| T061 | DisplayNameSet 72 ; DeliveryRejected 72 ; ThreadRequest 71 |
| T062 | DelegatedRuntimeEvent 67 ; NativePermissionFact 65 ; WaitAgentLinks 52 |
| T063 | T3ThreadBindingFact 50 ; DaemonIdentityRequest 49 ; Subscribed 48 |
| T064 | ObservedActivity 47 ; TurnState 42 ; Unsubscribe 41 |
| T065 | DeliveryIndeterminate 38 ; ListRequests 37 ; AdoptStoppedOrder 36 ; DisplayNameResolve 35 ; DirectoryScoped 34 |
| T066 | AttachRejected 34 ; JournalReady 34 ; LiveJournalFragment 32 ; Gap 32 |

Total : 4+4+4+4+3+3+3+3+5+4 = 37 branches. Nom de fonction : `on_` + nom de la variante en minuscules avec `_` (ex. `on_service_request`, `on_relaunch_order`, `on_gap`).
Une variante absente (supprimée par une autre session) : la noter et passer.

## T067 - Contrôle de complétude

```sh
rg -c 'wrapper_dispatch::on_' crates/bridget-daemon/src/daemon.rs     # attendu : 37 (moins les branches notées arrêtées)
rg -n 'fn handle_wrapper_message_with_peer|pub fn get_status' crates/bridget-daemon/src/daemon.rs
```

Longueur = écart des deux lignes. Écrire dans `validation/lot05-fin.txt`. Critère SC06 : moins de 1500. Si la valeur est entre 1500 et 1800 : lister les 5 plus grandes branches restantes (lecture de la fonction, par tailles à vue) et les noter dans le journal ; ne pas en extraire d'autres (hors périmètre).

## T068 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot05`), `C-ECHECS` : aucune différence.
2. `C-NOMS` : aucune différence (aucun test ajouté ni retiré).
3. Bilan dans `implementation.md` : liste des branches arrêtées ou notées, longueur finale, « activation en attente ».
4. Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Une branche refuse de compiler après 6 allers-retours : annuler cette branche (`git checkout -- crates/bridget-daemon/src/daemon.rs crates/bridget-daemon/src/daemon/wrapper_dispatch.rs`), noter, passer à la suivante. Les commits précédents restent.
- Une branche a besoin d'une mutation d'une variable locale de la fonction englobante (`let mut` modifié par la branche) : ne pas la déplacer, la noter.
- `C-ECHECS` montre une différence : `git revert` du dernier commit de branche, rejouer (R3).
- Conflit Git sur `daemon.rs` à l'intégration : arrêt du lot (R4). Le fichier est le plus modifié du dépôt ; réaliser le lot vite et l'intégrer dès la porte franchie.
