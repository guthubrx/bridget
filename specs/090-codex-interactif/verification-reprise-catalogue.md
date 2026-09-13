# Correctif reprise par nom — 07/09/2026

État : correctif validé et installé le 07/09/2026.

## Incident et mesure

Commande humaine : `bridget codex --yolo resume horizon-calliope --name calliope`.
Échec `échéance Codex sur thread/list`, puis arrêt du groupe 56591 non confirmé.
Lors du diagnostic, PID et groupe 56591 absents. Aucun signal envoyé à ce groupe.

CLI local `/opt/homebrew/bin/codex`, version 0.153.4. Schéma produit par ce binaire :
`/tmp/b90resume.Bw61OV/schema/v2/ThreadListParams.json`. Il documente explicitement
`useStateDbOnly` comme lecture sans scan des rollouts JSONL pour réparation.

Mesures sur serveurs app-server privés sans aucun thread/start/turn/start :

| Requête | Résultat |
|---|---|
| Catalogue par défaut | page 1 : 6,623 s ; page 2 : 29,588 s ; page 3 ne répond pas dans 60 s |
| searchTerm seul | 46,993 s pour un résultat |
| searchTerm + useStateDbOnly | 0,818 s pour le bon fil |
| useStateDbOnly seul | 4 pages, 364 conversations, 1,811 s au total |

Le bon fil `horizon-calliope`, UUID `01a0755e-317d-7723-baae-1050e16316f8`, est
retourné par l'API. Le correctif conserve la pagination complète et la comparaison
exacte/ambiguïté déjà existantes : aucun nouveau filtre, sélecteur, cache, lecture
de DB/rollout depuis Bridget, allongement de délai ou second serveur de catalogue.

Logs : `/tmp/b90resume.Bw61OV/probe.log`, `search.log`, `search-db.log`, `db.log`.
Les serveurs privés sans scan sortent proprement après fermeture stdin (9–19 ms).
La sonde scan expirée a reçu TERM après 10 s d'attente de fermeture stdin et est
sortie avec -15 ; aucun SIGKILL ni action sur une session utilisateur.
Cette observation ne justifie pas à elle seule de modifier stop_owned_child.

## Validations

Rouge observé avant patch : `cargo test -p bridget-transport
spec090_catalogue_interactif_utilise_state_db_seul_a_chaque_page -- --nocapture`,
un test exécuté / un échec attendu sur le drapeau absent (`None` vs `true`).
Log `/tmp/b90resume.Bw61OV/red.log`. L'oracle utilise le collecteur réel et deux
pages avec un même nom : il ne remplace pas la preuve runtime de scan lent par
une attente artificielle. Aucun nouveau proxy/serveur factice créé.

Après ajout du drapeau : filtre `catalogue`, 3 tests réussis, zéro échec.
Recettes existantes rejouées, sans modification du harnais ni des tests daemon :

- `--named-resume` : PASS, vrai Codex 0.153.4 + TUI + daemon privé. Nom résolu,
  historique/UUID/titre préservés et bypass explicite attesté dans le contexte.
- `--menu-resume` : PASS, même preuve via le menu ; aucun fil provisoire.
- `--name-ambiguous` : PASS, refus avant présence/prompt sans choisir arbitrairement.

Ces recettes utilisent des homes/sockets privés et un fournisseur HTTP local de
test, pas le compte utilisateur. Nettoyage terminé avec code 0 dans les trois cas.
Logs complets : `/tmp/b90resume.Bw61OV/named-resume.log`,
`/tmp/b90resume.Bw61OV/menu-resume.log`, `/tmp/b90resume.Bw61OV/name-ambiguous.log`.

Revue pilote du delta : trois lignes de production (virgule, commentaire, option)
dans le collecteur commun ; aucune modification de timeout, parsing, sélection,
permissions, wrapper, TUI, gestion de groupe ou fichiers Codex.
Les acquis clavier/Markdown restent inchangés.

Formatage workspace et clippy workspace/all-targets `-D warnings` : PASS.
Release optimisé compilé en 40,55 s. La recette `--named-resume` est aussi PASS
sur ce release, avec conservation du fil/historique/nom/UUID et terminaison propre.
Log : `/tmp/b90resume.Bw61OV/release-named-resume.log`.

Consolidation `cargo test --workspace -- --test-threads=1` : code 0,
71 suites, 1252 réussites / 0 échec / 47 ignorés. Log complet :
`/tmp/b90resume.Bw61OV/workspace.log`.

Installation atomique de
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`, comparaison
octet pour octet avec le release testé, et vérification par le chemin usuel
`/Users/moi/.local/bin/bridget` :
`e092f3044f48e87fdcc84b1b687ffada6815a80fc1e82f004755a72a3845a1d3`.
Ancien binaire conservé à
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-resume-catalogue.YKqhje/bridget.previous`.
Aucun commit, changement de réglage ou redémarrage de daemon/agent utilisateur.
La commande humaine peut conserver son nom Codex, sans substitution par UUID.
