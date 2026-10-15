# Développement168 — clôture du lot décideurs D

Date : 2026-10-15.
État : sources D gelées. Code et tests source prêts pour la campagne ROOT.
Ce reçu ne déclare aucun test PASS et ne valide aucun runtime168.

## Périmètre et ownership

Le travail s’est fait exclusivement sur Cartae, dans le worktree autorisé :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte

Reprise explicite du lot D après arrêt de son précédent propriétaire. Base annoncée : 1f69e5d8.
Aucune nouvelle session ni aucun nouveau workflow n’a été ouvert.

Fichiers modifiés par cette reprise :
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/Cargo.toml
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/developpement168-d-final.md

Le manifeste a reçu uniquement features=["raw_value"] pour la dépendance serde_json déjà présente.
ROOT a autorisé cet ajout. Le manifeste racine garde serde_json="1", sans changement de version.
RawValue emprunte le JSON imbriqué pour vérifier ses 4096 octets avant de décoder ses champs.
Ce choix évite un second parseur JSON et n’ajoute aucune dépendance.

Le helper stop_owned_child est réutilisé dans :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs

Ce helper et les exports existants dans le fichier ci-dessous n’ont pas été modifiés par cette reprise :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lib.rs

## Code terminé

La signature publique run_isolated_claude_decision et les champs de ClaudeIsolatedDecisionOptions restent inchangés.
Une seule extension publique, autorisée par ROOT et publiée à C : CallerDeadline::until(Instant).
CallerDeadline::new conserve ses deux secondes historiques. Une échéance expirée refuse avant réseau.

La surveillance Claude démarre son échéance avant le lancement et la création du contexte enfant.
Le répertoire enfant est créé privé en mode0700. Seul l’enfant change de répertoire.
L’environnement enfant est vidé, puis reçoit la liste autorisée fournie par C.
Aucun changement global de cwd ou d’environnement n’est effectué par ce chemin.

stdin et stdout sont non bloquants. Une seule boucle lit et écrit les deux tubes.
Le tampon stdout est limité à 64 Kio plus un octet de détection.
StructuredOutput.input, result.structured_output et le JSON du champ result sont limités à 4 Kio avant leur décodage détaillé.
stderr va directement à /dev/null. Aucun tampon stderr n’est créé.

Tous les retours après spawn, y compris une erreur try_wait, passent par l’arrêt confirmé du groupe détenu.
La sortie du leader ne suffit pas si un descendant tient stdout. Aucun SIGKILL ni signal à un autre groupe.
Si TERM et INT ne suffisent pas, le thread conserve child et le groupe jusqu’à confirmation physique.
C doit posséder RoutingDecisionPermit dans ce thread. Le receiver du demandeur peut expirer sans abandonner le permis.

La commande conserve les flags qualifiés : print, stream-json en entrée/sortie, verbose, safe-mode,
restricted, outils vides, MCP strict vide, sources de paramètres vides, slash commands désactivées,
sans persistance de session et avec schéma fermé. Aucun bare ou max-turns1 n’est ajouté.
Le seul outil admis est StructuredOutput. Les outils, MCP, skills, plugins et slash commands étrangers sont refusés.
Les trois capacités de cycle de vie observées lors de la qualification restent des métadonnées autorisées.
Toute trame de contrôle inattendue, initialisation répétée, outil répété, résultat multiple ou contradiction refuse.
Le JSON final est décodé directement dans le type fermé, afin de conserver le refus des champs dupliqués.
C valide encore ID, effort et contraintes du snapshot avant CAS. Le modèle ne gagne aucune autorité.

Laya168 transmet l’ensemble entier de RoutingDecisionRequest dans le champ state réellement lu.
Plus de huit candidats, state>1536 ou JSON>4096 produit CapacityUnavailable avant réseau.
Aucun top8, tournoi, découpage ni suppression de faits.
Les IDs168 c001..c128 restent distincts des IDs163 c01..c08.
Laya ne déclare pas un effort choisi : le défaut attesté est conservé.
default_effort=None avec liste d’efforts vide reste le défaut natif autorisé.
Un défaut absent avec des efforts déclarés est invalide.
Les gardes de réponse Laya168 reprennent les contrôles réels de modèle, routage et usage de163.
Le constructeur et le parseur historiques163 restent inchangés.

## Oracles source ajoutés

Dans le banc transport existant, sept tests :
- spec168_faux_executable_verifie_flags_env_cwd_et_stderr_neutralise
- spec168_stdin_non_lu_expire_depuis_le_lancement_et_arrete_enfant
- spec168_stdout_excessif_est_borne_et_arrete_sans_attendre_la_fin_naturelle
- spec168_sortie_leader_ne_libere_pas_un_descendant_qui_tient_stdout
- spec168_exit_nonzero_refuse_meme_une_sortie_structuree_valide
- spec168_formateur_refuse_outils_capacites_et_reponses_contradictoires
- spec168_objets_json_sont_bornes_avant_leur_decodage_detaille

Ils utilisent le patron de faux fournisseur Python déjà présent dans les bancs de processus.
Le faux exécutable inspecte les vrais arguments, l’environnement, cwd0700 et le descripteur stderr.
Les refus couvrent aussi outils interdits, capacités étrangères, contradiction, répétition,
champ inconnu ou dupliqué, stdout excessif et objet JSON excessif, y compris ses espaces bruts.

Dans le banc décision HTTP existant, six tests :
- spec168_laya_post_choisit_sur_l_ensemble_entier_et_garde_ses_faits
- spec168_laya_refuse_capacite_et_deadline_avant_tout_reseau
- spec168_laya_default_natif_null_et_transport_legacy163_restent_distincts
- spec168_laya_http_refuse_choix_effort_et_metadonnees_invalides
- spec168_effort_valide_ou_defaut_atteste_sans_niveau_invente
- spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel

Le POST contrôlé choisit réellement c004 parmi quatre candidats et vérifie tous les faits reçus.
Un socket en écoute observe l’absence totale d’appel pour plus de huit candidats,
state excessif et échéance expirée.
Le banc réemploie ControlledServer et inspecte les octets HTTP réels.
Le test de permis utilise un leader et un descendant qui ignorent TERM/INT jusqu’à une levée contrôlée.
Il vérifie l’expiration du demandeur, le refus d’une seconde chaîne et la libération après drain.
Il lève son verrou avant toute assertion, pour ne pas abandonner de processus de fixture.

## Vérifications effectuées

Lecture ciblée des contrats168, du reuse-audit et du reçu D précédent.
Lecture du diff complet du périmètre décideurs, puis self-review ciblée après simplification.
rustfmt ciblé : exécuté, RC0.
git diff --check ciblé : exécuté, RC0.

Commande de formatage exécutée sur Cartae :
/home/moi/.cargo/bin/rustfmt --edition 2024 --config skip_children=true /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs

Le premier appel rustfmt par nom court n’était pas dans le PATH SSH. L’appel absolu ci-dessus a réussi.
Aucun Cargo, rustc, build ou test n’a été exécuté par cette reprise.
Aucun appel modèle supplémentaire, aucune mutation Git, aucun restart, aucune modification auth ou configuration de production.

## Self-review Articles XVIII, XIX et XX

Complexité : validation O(n log n+b), n<=128 ; transport et parsing O(b), b borné.
Aucune requête par candidat ni collecte supplémentaire. Un POST représente la charge entière.

Minimalisme & Frugalité : mêmes client HTTP, permis, parseurs serde et helper de groupe.
Aucun nouveau runner, scheduler, Task, stockage métier ou framework de tests.
La validation finale redondante de decide_routing et une garde devenue impossible après validation ont été retirées.
Les trois DTO privés du flux portent des règles de sécurité et l’emprunt RawValue.
Potentiel minimalisme restant : environ0 ligne identifiée dans ce périmètre après cette checklist.
Les lignes de tests couvrent des frontières de processus et HTTP réelles ; elles ne prouvent pas encore leur succès.

Vertus LLM & Responsabilité Future : le contrôle du groupe a un seul chemin de sortie.
L’emprunt brut évite un parseur artisanal et rend la borne4K explicite.
La charge de maintenance ajoutée correspond aux limites du rôle décideur et à ses preuves.
Les abstractions conservées portent la compatibilité163, l’isolement ou la durée de vie du permis.
La qualification CLI antérieure reste distincte du transport codé : aucun succès runtime168 n’est déduit.

## Commandes futures exactes, sous permis Cargo ROOT uniquement

ROOT garde l’unique permis Cargo. Paramètres annoncés : jobs2, testthreads2.
Ces commandes sont proposées, pas exécutées par le présent lot.

/home/moi/.cargo/bin/cargo test --manifest-path /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.toml -j 2 -p bridget-transport --lib claude_stream_json::tests::spec168 -- --test-threads=2

Après gel du raccord daemon C :
/home/moi/.cargo/bin/cargo test --manifest-path /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.toml -j 2 -p bridget-daemon --lib decision_service::tests::spec168 -- --test-threads=2

Régression du banc historique163 :
/home/moi/.cargo/bin/cargo test --manifest-path /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.toml -j 2 -p bridget-daemon --lib decision_service::tests::spec163 -- --test-threads=2

Le raccord chaîne/CAS et sa validation tardive appartiennent à C.
Les tests ajoutés ne ferment aucune tâche de validation avant leur exécution par ROOT.

## Empreintes du gel source

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs
SHA-256 : 8797baddebd73b50d186e6ff8bd6de9b8bcc581e930ddfd637ddbf34c83913ac

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs
SHA-256 : e1c02b307c518be18eaef4004042df728e6da74bff1cbb18949861b62f8ee8b0

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/Cargo.toml
SHA-256 : 6e4e189b9882d34eb074828e4c213b4f489e937b52e428af8788632fdae68cce

Ces empreintes identifient les sources gelées. Elles ne sont pas une preuve de compilation ou de test.
