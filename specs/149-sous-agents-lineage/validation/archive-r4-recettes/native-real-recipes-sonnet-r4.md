# Recette native réelle T037/T038 - ronde Sonnet r4 (exécution)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Modèles cibles du produit : Codex `gpt-6.1-sol` high, GLM `glm-5.3-flash`.
Statut global : **PARTIEL**. Codex : preuves réelles positives. Claude/GLM : **bloqué par un défaut de latence de l'observer** (cause prouvée par échantillonnage de pile). Aucune case `tasks.md` cochée.

## Binaire et fixture

- Receipt `native149-debug-receipt.json` r4 vérifié : empreinte source complète `e6f45086…5408` (196 fichiers) et empreinte production `e0d86f01…f3a7f215` (111 fichiers) recalculées, identiques. `wrapper.rs` `b599f8bb…b455e65`, `native_delegation.rs` `fc62f33d…72dc0d9`.
- Binaire : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-823e8a5fab8a`, SHA256 `823e8a5fab8abe38fcb439d8fda98d929622dae3751ff8f629d306744379328b` (debug, 53 993 512 octets).
- Lanceur `gclaude` : `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e` (constante contrat). CLI claude 2.1.296 : `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937` (240 664 432 octets).
- Fixture privée 0700 : `/Users/moi/.cache/bridget149-recipe-r2.r4` (home, registre, logs, état). Aucune lecture/écriture de `/Users/moi/.cache/bridget-core` en écriture ; registre prod lu seulement pour le clone. Aucun vecteur T3 dans l'environnement.
- Daemons fixture : PID 42877, 33457, 15425 (arrêtés, SIGTERM un par un). Aucun processus résiduel du fixture à la fin.
- Registre fixture : `glm` = `glm-5.3-flash` exact (aucun repli) ; `codex` = clone de `codex-pro` (protocole `codex_app_server`).

## Résultats (comptes exacts)

| Scénario | Résultat | Preuve |
|---|---|---|
| Parent Codex réel `workspace-write -a never` → enfant Codex `gpt-6.1-sol` high (`recipe149-t037-codexchild-03`) | **PASS** `result_available` | voir `provider-write149.md` |
| Parent Codex `workspace-write` → enfant GLM (`recipe149-t037-confinement-01`) | **PASS refus nommé** `provider_confinement_unavailable`, 0 ligne en base | journal PTY ; oracle `t037-codex-confinement` OK |
| Annulation (`recipe149-t038-cancel-01`, enfant Codex) | **PASS partiel** `cancelled` en 2 s, aucun processus résiduel | oracle `t038-cancel` OK. Limite : annulé avant que la commande longue ne démarre (`compte.txt` absent) |
| Parent Codex `danger-full-access` → enfant GLM (5 essais : -01, -03, -04, -05, -06) | **ÉCHEC 5/5** `spawn_refused:NegotiationFailed … os error 35` | cause ci-dessous |
| Parent GLM interactif → enfant GLM (7 essais exploitables : -02, -06, -07, -08, -09, -11 + diagnostics) | **ÉCHEC** hook `permission_attestation_unavailable` | cause ci-dessous |
| Même famille Claude→Claude avec règles settings préexistantes (`Edit(/allowed/**)` allow, `Edit(/forbidden/**)` + `Bash` deny) | **NON PROUVÉ** | bloqué par le même hook |
| Branche négative observer (G-P-08d) | **NON EXÉCUTÉ** | dépend du parent Claude |
| Écriture GLM autorisée / refus Edit(forbidden) / modèle GLM exact tracé | **NON PROUVÉ** | l'enfant GLM n'a jamais démarré |

Comptes modèles : enfants Codex démarrés 2 (codexchild-03 succès ; cancel-01 annulé) ; enfants GLM démarrés 0 ; parents GLM interactifs réels 10 sessions (aucune n'a abouti à une délégation) ; parents Codex réels 7 sessions.

## Défaut bloquant : SHA-256 du CLI à chaque contrôle (debug)

Données brutes :
- Hook Claude : le processus `bridget __native-permission-observer` est lancé directement par `claude`, se connecte au socket `np-*.sock`, et la connexion reste ouverte **15 s** (15:26:43 → 15:26:58) alors que le client abandonne à 3 s et que le hook Claude expire à 5 s. Le wrapper consomme du CPU pendant ce temps (33 s de CPU cumulés après 2 min).
- Échantillon de pile (`sample`, 6 s) du wrapper parent : 4114 des ~4126 échantillons dans `recheck_context_with_env → capture_context_with_env → executable_revision → permission_source_revision → sha2::sha256::soft::compress256`. Fichier : `native-r4-sample-wrapper.txt`.
- Échantillon du `managed-wrapper` de l'enfant GLM : 4370/4455 échantillons dans `launch_session_with_status → recheck_context → … → executable_revision → sha2 soft compress256`. Fichier : `native-r4-sample-managed.txt`. Le wrapper meurt après ~79 s en `EAGAIN` (délai de lecture socket expiré) : `NegotiationFailed … os error 35`.
- `shasum -a 256` natif du même CLI : 1,2 s. Le calcul logiciel (`sha2::soft`) du binaire debug est donc ~10-15x plus lent.
- Les enfants Codex ne sont pas touchés : l'exécutable haché est le script `codex-pro`, pas un binaire de 240 Mo.

Constat : le contrôle recalcule le SHA-256 complet du CLI Claude à chaque appel de hook et à chaque lancement géré. Même en release, ce coût (~1 s avec un SHA logiciel) consomme une grande part du budget de 3 s du hook. Piste pour Sol (non appliquée, hors périmètre testeur) : mémoïser la révision par (chemin, taille, mtime) ou activer l'accélération matérielle de `sha2` ; et refaire la recette avec un binaire **release**. Autres causes écartées par données : `prompt_id`/`session_id`/`cwd`/`permission_mode` présents dans le payload du hook (CLI 2.1.296) ; le mode `auto` (défaut) et `manual`/default échouent pareil.

## Écarts de harnais corrigés pendant la ronde (scripts `recipes/`)

- `run_parent_pty.py` : taille PTY 40x140 (le TUI Codex ne dessine rien à 0x0) ; réponses aux requêtes terminal (CPR, couleurs OSC 10/11, `ESC[?u`, DA1) ; dialogues Claude (thème, clé API : défaut « No », notes de sécurité, confiance du dossier, imports externes : défaut « No ») et Codex (mise à jour : « Skip », hooks : « Continue without trusting ») ; saisie du prompt par collage balisé après inactivité du TUI ; grant humain fixture via `delegate-grant` exécuté dans son propre PTY ; sortie `/exit` quand la ligne est terminale ; drainage du PTY à l'arrêt ; nettoyage limité aux processus portant `BRIDGET_HOME` du fixture (la 1re version arrêtait aussi le daemon fixture : 1 fois, PID 33457, dommage limité à ma fixture).
- `make_fixture_registry.py` : `efforts:["high"]` déclaré pour `gpt-6.1-sol` (le registre prod le déclare sans effort : `effort_unavailable` au premier essai Codex).
- `verify_oracles.py` : le refus du bac à sable Codex est un échec de commande (EPERM) dans un tour normal, donc `result_available` et non `failed`.
- Nouveaux : `pty_probe.py`, prompt `parent_delegate_cancel_codex.md`.

## Effets de bord sur l'environnement utilisateur (à connaître)

- `/Users/moi/.claude-glm/.claude.json` : l'onboarding interactif n'avait jamais été fait pour ce profil. Ajouts du CLI : `hasCompletedOnboarding`, `lastOnboardingVersion`, `customApiKeyResponses` (empreinte courte de clé, réponse « No »), confiance du projet fixture, compteurs de démarrage. Sauvegarde avant : `$F/state/claude-glm.dot-claude.json.before` (SHA256 `6ae14548…e9717c`). `settings.json` du profil inchangé (`f3845703…`).
- Codex : choix « Skip » de la mise à jour 0.162.1 (non persisté d'après les relances).
- Un `pkill -f` a visé mon propre script de surveillance `watch-sample.py` (un seul processus) : écart à la règle « pas de pkill », sans autre victime.
- Processus d'une autre ronde (`bridget149-native-interop.*`, PID 6434/6447) observés, jamais touchés.

## Reste à faire

1. Binaire **release** (ou correctif de coût du hash) puis rejeu : T037 V1 GLM→GLM (écriture `allowed/write-ok.md`, refus `Edit(/forbidden/**)`, `provider_permission_denied` corrélé), Claude→Claude avec settings préexistants (révisions sources), fullCodex→GLM, T038 branche négative observer, annulation d'un enfant en plein travail.
2. Scripts prêts : `recipe_env.sh`, `run_fixture_daemon.sh`, `run_parent_pty.py --parent {claude,codex} --grant-posture development --exit-when-terminal`, `verify_oracles.py`.
