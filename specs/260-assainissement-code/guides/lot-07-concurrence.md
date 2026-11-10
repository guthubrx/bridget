# Lot 7 - Concurrence observable (T081-T091)

But : voir ce que le démon cache aujourd'hui, sans changer sa conduite. Trois mesures : verrous empoisonnés récupérés, connexions actives et leur maximum, durée du tour de délégation. Mesures de départ : `releves.md` section 6.
Branche : `session-260-lot07-concurrence`. Worktree : `.worktrees/260-lot07-concurrence`. Code exécutable modifié : « activation en attente » (R8).
Règles de fond (FR14 à FR16, research C9) :

- Un verrou empoisonné reste récupéré **comme aujourd'hui** : le service continue. On ajoute seulement une trace et un compteur.
- Aucun plafond de connexions (D6 : non). Aucun compteur dans `get_status` (ce serait un changement de protocole, R6.1).
- Aucun regroupement de requêtes (D7). On mesure d'abord.

## T081 - Trait `RecoverPoison` et compteur

Créer `crates/bridget-transport/src/poison.rs`, déclarer `pub mod poison;` dans `crates/bridget-transport/src/lib.rs` (à côté de `pub mod fsutil;`). Contenu :

```rust
//! Récupération d'un verrou empoisonné avec trace et compteur.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::PoisonError;

static POISON_RECOVERIES: AtomicU64 = AtomicU64::new(0);

/// Nombre de verrous empoisonnés récupérés depuis le démarrage du processus.
pub fn poison_recoveries() -> u64 {
    POISON_RECOVERIES.load(Ordering::Relaxed)
}

pub trait RecoverPoison<G> {
    /// Retourne la garde même si le verrou est empoisonné (comportement historique), et consigne le fait.
    fn recover_poison(self) -> G;
}

impl<G> RecoverPoison<G> for Result<G, PoisonError<G>> {
    fn recover_poison(self) -> G {
        self.unwrap_or_else(|poisoned| {
            let n = POISON_RECOVERIES.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_power_of_two() {
                log::warn!("verrou empoisonné récupéré ({n} fois depuis le démarrage)");
            }
            poisoned.into_inner()
        })
    }
}
```

La trace est émise aux puissances de deux (1, 2, 4, 8...) pour ne pas inonder le journal. Ajouter deux tests dans le même fichier : (1) un verrou sain retourne sa valeur et ne change pas `poison_recoveries()` ; (2) un verrou empoisonné (thread qui panique en le tenant, `std::panic::catch_unwind` ou `thread::spawn(...).join().unwrap_err()`) retourne la valeur et fait passer le compteur de `n` à `n+1`. Les tests s'exécutent en parallèle : comparer `poison_recoveries()` avant/après avec `>=` (le compteur est global).
Vérifier : `C-CHECK`, `C-CRATE` (`bridget-transport`). Ce lot ajoute 2 tests : la ligne de base des noms change (voir T091). Commit : `feat(transport): Ajouter la recuperation tracee des verrous empoisonnes`.

## T082 - Mesure avant le remplacement

```sh
rg -c 'unwrap_or_else\(\|\w+\| *\w+\.into_inner\(\)\)' crates --glob '*.rs' | sort -t: -k2 -nr > "$V/lot07-sites-avant.txt"
awk -F: '{s+=$2} END{print s, "sites"}' "$V/lot07-sites-avant.txt"
```

Attendu : plus de 600 sites. Écrire le total dans le journal. Commit : `docs(specs): Releve des verrous empoisonnes avant remplacement`.

## T083 à T087 - Remplacement fichier par fichier

Motif remplacé : `.unwrap_or_else(|e| e.into_inner())` (le nom de la variable peut varier) devient `.recover_poison()`.

```sh
perl -pi -e 's/\.unwrap_or_else\(\|(\w+)\|\s*\1\.into_inner\(\)\)/.recover_poison()/g' <fichier>
```

Ensuite, importer le trait dans chaque fichier modifié qui ne l'a pas :

- crate démon : `use bridget_transport::poison::RecoverPoison;`
- crate transport : `use crate::poison::RecoverPoison;`

Placer la ligne après le dernier `use` de tête de fichier (colonne 0, hors `#[cfg(test)]`). Les sous-modules qui écrivent `use super::*;` reçoivent l'import de leur parent. Si le compilateur dit « no method named `recover_poison` » : ajouter l'import dans ce fichier.
Vérifier après chaque tâche : `C-CHECK` ; `git diff -U0 | rg '^[+-]' | rg -v 'recover_poison|into_inner|RecoverPoison|^(\+\+\+|---)'` ne montre aucune ligne. Un cas où le `into_inner()` ne vient pas d'un verrou (pas de `PoisonError`) : le compilateur refuse ; annuler la ligne et la noter.

| Tâche | Fichiers | Commit |
|-------|----------|--------|
| T083 | `crates/bridget-daemon/src/daemon.rs` | `refactor(daemon): Tracer les verrous empoisonnes de daemon.rs` |
| T084 | `crates/bridget-daemon/src/daemon/native_delegation.rs`, `native_lineage.rs` | idem pour ces fichiers |
| T085 | `crates/bridget-transport/src/codex_app_server.rs`, `acp.rs`, `claude_stream_json.rs` | idem |
| T086 | `crates/bridget-daemon/src/wrapper.rs`, `fleet.rs` | idem |
| T087 | tous les fichiers restants listés par `rg -l 'unwrap_or_else\(\|\w+\| *\w+\.into_inner\(\)\)' crates --glob '*.rs'` (commit par crate) | idem |

Après T087 : `C-CRATE` pour les deux crates. Relancer la commande de T082 avec `<nom>` = `apres` : le total ne doit plus compter que les sites non remplacés, notés dans le journal.

## T088 - Compteur de connexions et maximum

Existant : `pub struct Metrics` (`daemon.rs`, chercher `pub active_connections: AtomicUsize`) avec `increment_connections` et `decrement_connections`, jamais appelés. Chercher la boucle d'acceptation : `rg -n 'match listener.accept\(\)' crates/bridget-daemon/src/daemon.rs` (première occurrence hors tests, environ ligne 7356 avant les lots précédents).
1. Dans `Metrics`, ajouter `pub max_connections: AtomicUsize` (valeur initiale 0, à côté de `active_connections`) et dans `increment_connections` : `let n = self.active_connections.fetch_add(1, Ordering::Relaxed) + 1; if self.max_connections.fetch_max(n, Ordering::Relaxed) < n && n.is_power_of_two() { log::info!("connexions actives : {n} (nouveau maximum)"); }`.
2. Appeler `increment_connections()` à l'acceptation, avant le `thread::spawn`. Dans le thread, première ligne : une garde locale dont le `Drop` appelle `decrement_connections()`. Elle couvre toutes les sorties, dont les `return` anticipés. Ne pas ajouter de plafond ni de refus.
3. Test : dans le module de tests existant de `Metrics` (chercher `Metrics::` dans les tests), ajouter un test à deux incréments et un décrément : `active_connections == 1`, `max_connections == 2`.
Justification de la garde (1 usage) à écrire dans le journal : sans elle, une sortie anticipée laisserait le compteur faux. Vérifier : `C-CHECK`, `C-CRATE` (`bridget-daemon`). Commit : `feat(daemon): Compter les connexions actives et leur maximum`.

## T089 - Durée du tour de délégation

Chercher `native_delegation::tick(&st_reminder);` (boucle du thread `st_reminder`, environ ligne 7077). Remplacer cette ligne par :

```rust
let tick_start = std::time::Instant::now();
native_delegation::tick(&st_reminder);
let tick_elapsed = tick_start.elapsed();
if tick_elapsed > NATIVE_TICK_SLOW {
    log::warn!("tour de délégation lent : {} ms", tick_elapsed.as_millis());
}
```

Ajouter, près de la boucle, la constante `const NATIVE_TICK_SLOW: Duration = Duration::from_millis(250);` avec le commentaire : `// Seuil provisoire : un quart de la période du tour (1 s). À ajuster après mesure réelle (programme 260, D7).` La mesure ne change ni l'ordre ni la durée des appels. Vérifier : `C-CHECK`. Commit : `feat(daemon): Consigner les tours de delegation lents`.

## T090 - Regroupement des requêtes (fermée par D7)

Lire D7 dans `decisions.md` (`origin/main`). Statut `en attente` : **ne rien modifier au code.** Écrire dans le journal : « différé (porte D7) », et décrire en 5 lignes ce qu'il faut mesurer après activation (nombre de lignes de `log::warn` « tour de délégation lent » par heure, durée maximale lue). Les endroits à étudier plus tard : `native_delegation.rs`, fonction `tick`, requêtes par élément (`open_requests()` puis `for_mission`, `failed_for_owner`). Cocher la case de la tâche seulement avec cette note écrite. Statut `tranchée` : écrire la réponse ; si elle demande un regroupement, **ne pas l'implémenter ici** (hors périmètre) : écrire « à spécifier » et le dire à François.

## T091 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot07`), `C-ECHECS` : aucune différence.
2. `C-NOMS` : la différence avec la base est exactement les tests ajoutés (T081 : 2, T088 : 1). Vérifier que ce sont les seules lignes ajoutées, puis mettre à jour `base-noms.txt` et le dire.
3. Contrôle FR14 : `rg 'into_inner' crates --glob '*.rs' | rg -c 'unwrap_or_else'` doit baisser du nombre de sites remplacés ; aucun `.lock().unwrap()` n'a été ajouté (`git diff origin/main -U0 | rg '^\+.*\.lock\(\)\.unwrap\(\)'` : aucune ligne).
4. Bilan : sites remplacés, sites restants, seuil du tour, « activation en attente ; mesures réelles à lire après activation ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Un test existant échoue parce qu'un compteur global change : ne pas modifier le test, noter, ajuster seulement si le test lit le compteur ajouté.
- Remplacement qui ne compile pas dans plus de 5 fichiers : arrêter à T084, noter. Le trait reste utile (T081) même si le remplacement est partiel.
- Interdit : plafond de connexions, nouveau champ de protocole, arrêt du service sur verrou empoisonné.
