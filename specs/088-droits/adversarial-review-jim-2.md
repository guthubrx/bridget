# Contre-revue adverse — SPEC-088 (implémentation, passe 2)

Date : demande envoyée le 2026-09-03 à 14:45 CEST par la ligne de commande du serveur ; verdict écrit par Jim (Codex, `437b3175-…`) et lu à 21:03 CEST.
Verdict reçu : **BLOCKED**, revue statique du diff non commité.

## Instruction des objections

| Objection | Vérifiée comment | Retenue | Changement |
|---|---|---|---|
| P0 la contrainte `CHECK` de `control_events` d'une base existante refuse `rights_set` | lecture de `ensure_schema` (CREATE IF NOT EXISTS) et de la clause 087 | oui | migration transactionnelle : colonnes + reconstruction de `control_events` avec copie des lignes ; test depuis le schéma exact 087 suivi d'un `set` de droits accepté |
| P0 le résolveur de test s'arrête à la fin de commande, écrite AVANT l'acte refusal | lecture de `record_command_completion` (fin puis refus) et du `break` | oui | parcours complet puis conclusion ; tests dans les deux ordres et avec deux items |
| `Unread` admet la réassignation (repli permissif) | `admit_autonomous_effect` | oui | `Reassignment` différée motif `droits` sous `Unread` ; test |
| `outputDelta` non filtré par le tour actif | comparaison avec `record_command_completion` | oui | garde `source_matches_active_turn` avant mémorisation |
| Tester « Modifier Bridget » peut rendre un faux réussi | lecture de la route | oui | jamais d'envoi : `refused_bridget` si désactivé, `enabled_not_measured` si activé |
| Admission « occupé » absente ; lecture du journal non bornée | route et `agent_journal_events` | partiellement | l'interface ne choisit jamais un agent `busy` ; lecture bornée (fichiers modifiés depuis la tentative, 3 fichiers, 20 000 événements) ; l'admission serveur reste asynchrone (tentative `pending` puis `unknown_expired`), documentée |
| Migration non transactionnelle, repli de lancement ouvert si l'état est illisible | `migrate_rights_columns`, `resolve_spawn_agent_type_for_posture` | oui | transaction unique, vérification des deux colonnes ; repli fermé : découverte ou refus |
| Signalement de sandbox falsifiable par `printf … ; exit 1` | `refusals.rs`, ADR-011 | non (désaccord documenté) | aucune information structurée authentifiée n'existe chez le fournisseur ; le signalement reste explicitement « non attesté », attribué à Bridget comme SIGNALEMENT, avec le geste Tester ; un agent qui ment n'est pas dans le modèle de menace de l'ADR-011. Arbitrage laissé au référent. |
| Docs : `server-rights.json` encore cité dans `quickstart.md` et `reuse-audit.md` | grep | oui | corrigés |

## Verdict brut de Jim

# Contre-revue adverse — SPEC-088, deuxième passe

**Verdict : BLOCKED.** La révision applique une part substantielle des cinq corrections demandées, mais elle ne rend pas l'ensemble sûr à mettre en production. Deux défauts sont bloquants : la migration d'une base réellement existante ne migre pas la contrainte de `control_events`, et la résolution d'un refus de sandbox par **Tester** ne voit pas le refus que son propre producteur écrit après la fin de commande. La reconnaissance de sandbox reste en outre un signalement falsifiable par une commande qui imprime la ligne puis échoue ; le nom « non attesté » est juste, mais il ne répond pas à une exigence de fiabilité.

Revue statique du diff non commité sur la base `a931a8ac`. Aucune compilation ni test n'a été lancé, conformément à la consigne.

## Réponses aux cinq corrections

| Correction attendue | Verdict | Éléments vérifiés |
|---|---|---|
| Signalement seulement si le même item finit en échec | **OUI, condition appliquée ; NON, provenance fiable.** | La ligne est mémorisée par `item_id` dans `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:1459`, retirée pour cet identifiant puis émise seulement si `failed` à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:1540`. En revanche, la ligne vient toujours d'`outputDelta`, texte brut contrôlable par la commande, à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:2356`. `printf 'bwrap: … Permission denied' >&2; exit 1` satisfait exactement le prédicat fermé de `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/refusals.rs:93`. Un agent peut donc fabriquer le signalement. |
| Tester corrélé par `message_id` et commande exacte | **OUI, corrélateurs présents ; NON, résultat refusé correct.** | Le `message_id` rendu est enregistré à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:943`. Le résolveur filtre ce message à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:1003` et compare la commande à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:1041`. Il exige aussi le même `item_id` pour le refus à `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:1066`. Mais il s'arrête dès la fin de commande (`break`, `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:1054`) alors que le producteur écrit cette fin **avant** l'acte `refusal` (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:1538` puis `:1565`). Un refus réel devient donc `unknown_expired`, jamais `refused_provider_sandbox`. |
| Droits dans `control_state`, sous une génération | **OUI dans le modèle neuf ; NON pour migration production.** | Les deux champs sont bien additifs dans `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/protocol.rs:1131` et passent par une unique transaction de `ControlStateSet` (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:276`, `:323`, `:389`). Le fichier `server-rights.json` a disparu du chemin exécutable ; seul `server-rights-tests.json` conserve les résultats de mesure (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:845`). Mais la migration historique est incomplète, détaillée ci-dessous. |
| `Option<bool>` sans valeur permissive par défaut | **NON au niveau du comportement.** | Sur le fil, `None` est correctement conservé (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/protocol.rs:1135`) et un état lu avec `None` est différé (`/home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/control.rs:104`). Toutefois, le rejet de négociation avec un daemon ancien retourne `Unread` (`/home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/control.rs:147`) et `Unread` admet tous les effets, y compris la réassignation (`/home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/control.rs:98`). C'est un défaut permissif de compatibilité, même s'il n'est pas écrit littéralement `Some(true)`. |
| `local_toggle` réservé au rendu local | **OUI pour ce geste précis ; NON pour l'isolement complet des lignes locales.** | Le normaliseur élimine un `local_toggle` hors origine locale (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/assets/ui/app.js:7373`) et seul le rendu des références le construit avec `allowLocalToggle` (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/assets/ui/app.js:7785`). En revanche, la matrice de profil est lue du payload serveur (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/assets/ui/app.js:7556`) puis écrite dans `localStorage` (`:7630`) après un clic de profil (`:9050`). Un serveur compromis ne desserre donc pas une ligne automatiquement, mais peut fournir les valeurs locales qu'un clic humain connu appliquera. |

## Défauts bloquants et changements minimaux prioritaires

### P0 — La migration de production refuse tout changement de droits

Une base `a931a8ac` possède déjà `control_events.kind CHECK (pause_on,pause_off,budget_set)` (version de référence `a931a8ac:crates/bridget-daemon/src/referent_control.rs:50` et `:136`). La nouvelle création ne modifie pas une table existante (`CREATE TABLE IF NOT EXISTS`, `/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:156`), et `migrate_rights_columns` ne touche que `control_state` (`:187`). Un `ControlStateSet` de droits tente ensuite d'insérer `rights_set` (`:400`) : la contrainte historique le refuse et la transaction annule la modification. Le test de migration ne reproduit pas ce cas : il crée seulement `control_state`, pas `control_events` (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:499`).

Changement minimal : migrer/reconstruire `control_events` dans la même transaction SQLite que les deux colonnes, avec la nouvelle contrainte et la copie des lignes existantes ; ajouter un test depuis le schéma exact de `a931a8ac`, suivi d'un `ControlStateSet` `rights_set` réellement accepté.

### P0 — Le refus généré ne peut pas être résolu par Tester

Voir la seconde ligne du tableau. Le test unitaire construit le refus **avant** la fin (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:1191`), ordre contraire du journal réel. Il valide donc une séquence impossible et masque le défaut.

Changement minimal : parcourir l'intégralité des événements du `message_id`, collecter les fins canoniques et les refus par `item_id`, puis résoudre après le parcours. Ajouter les deux ordres, en particulier `command/completed` puis `refusal`, et le cas de deux items.

## Risques importants et trous de couverture

### Signalement sandbox falsifiable et pollution inter-tour

ADR-028 dit explicitement qu'une ligne imprimée par l'agent n'établit rien (`/home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/028-refus-explicites-et-page-droits.md:13`). Le code en fait néanmoins un acte `refusal` attribué à Bridget (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-transport/src/codex_app_server.rs:1550`). La condition sortie + échec réduit le bruit mais ne distingue pas une sandbox d'un `printf` suivi d'un code 1. De plus `outputDelta` est retenu sans vérifier son `threadId`/`turnId` avant mise en cache (`:2356`), tandis que la fin, elle, vérifie le tour actif (`:1484`). Une sortie étrangère qui réutilise un `item_id` peut donc contaminer l'item actif.

Changement minimal : ne jamais convertir ce texte en refus causal ; conserver un diagnostic brut non attribué, ou n'élever le niveau qu'avec une information structurée authentifiée du fournisseur. À défaut, filtrer aussi `outputDelta` par `source_matches_active_turn` et inclure `(thread_id, turn_id, item_id)` dans la corrélation.

### Tester peut afficher un faux succès pour « Modifier Bridget »

Quand dogfooding est activé, le geste `bridget` devient la même lecture de zéro octet qu'un test Fichiers (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/control_settings.rs:906`), sur le chemin de politique (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:919`). Il ne vérifie ni le checkout système ni la capacité de modifier Bridget. La route admet en outre n'importe quel identifiant syntaxiquement valide (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:909`), donc un agent hors projet système peut réussir ce test.

Changement minimal : restreindre cette ligne à l'agent et au projet système attestés, puis utiliser un dry-run non mutateur de la même autorisation. Si ce dry-run n'existe pas, rendre l'issue « configuration activée, non mesurée » au lieu de `passed`.

### Tester ne respecte pas son contrat d'admission ni une lecture bornée

Le contrat promet `409 busy` (`/home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/contracts/rights-v1.md:13`), mais la route ne vérifie pas la présence et l'interface sélectionne explicitement un agent `busy` (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/assets/ui/app.js:9133`). Le résultat peut alors expirer « inconnu » alors que le droit est valide mais la commande est restée en file. Enfin chaque lecture parcourt tous les JSONL de l'agent, sans borne de 30 s ni curseur (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/ui.rs:853`).

Changement minimal : vérifier l'état libre au POST, persister une borne de lecture/curseur à l'admission, et limiter temps, volume et nombre de journaux lus. Ajouter un test d'agent occupé et un journal volumineux.

### Migration partielle et repli d'ouverture

Les deux `ALTER TABLE` ne sont pas protégés par une transaction explicite et le contrôle ne vérifie que l'existence de `agent_posture` avant de conclure que la migration est complète (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:187`). Une interruption entre les deux colonnes laisse une base que les démarrages suivants ne répareront pas. Lorsqu'une lecture de contrôle échoue, le lancement retombe alors sur le type normal, donc potentiellement la posture complète (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/daemon.rs:7424`).

Changement minimal : migration transactionnelle, vérification indépendante des deux colonnes et repli de lancement fermé (refus ou découverte) quand l'état est illisible.

## Chemins d'autorisation et ADR-003 / ADR-027

La nouvelle propriété est au bon endroit : `ControlStateSet` est contrôlé par le principal humain (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/daemon.rs:8850`) et Maicie ne fait qu'une lecture de l'état (`/home/moi/bridget-referent/.worktrees/088-droits/plugins/maicie/src/control.rs:3`). Cela respecte la séparation daemon/Maicie de l'ADR-003 et la vérité unique de l'ADR-027 (`/home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md:13`). Un agent utilisant ses outils déclarés ne peut pas muter les droits : le test dédié l'atteste (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/daemon.rs:24201`), et un message service est rejeté (`:9481`).

La borne ADR-011 subsiste : un processus local hostile capable de parler directement à la socket peut déclarer le périmètre humain, car la reconnaissance compare seulement une chaîne (`/home/moi/bridget-referent/.worktrees/088-droits/crates/bridget-daemon/src/referent_control.rs:31`). ADR-027 l'assume expressément (`/home/moi/bridget-referent/.worktrees/088-droits/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md:22`) ; ce n'est pas un nouveau chemin introduit par SPEC-088, mais ce n'est pas une garantie contre un agent ayant un shell arbitraire sur le même compte.

Enfin, les documents ne sont pas entièrement révisés : le quickstart ordonne encore de lire `server-rights.json` (`/home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/quickstart.md:13`) et le reuse audit garde la décision inverse (`/home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/reuse-audit.md:89`). Les gates restent décochées (`/home/moi/bridget-referent/.worktrees/088-droits/specs/088-droits/tasks.md:24`, `:35`, `:42`) ; ils devront être exécutés après les corrections, hors de cette revue.

