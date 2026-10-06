# Preuves Agent Loop — SPEC-138

Date : 2026-10-06, vérification finale après correctif E/S T020.
Périmètre isolé : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project.
Base communiquée : 124 tests existants (99 noyau, 25 systémiques).
Aucun commit, installation de skill, changement de run métier ou appel fournisseur.

## T011 — Tests avant modification

Commande, depuis /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop :

```text
python3 -m unittest discover -s tests -p test_agent_loop.py
```

Premier passage après ajout des huit cas138, avant code : exit1 ; 107 tests ;
2 failures, 6 errors. Échecs observés : pas de --project-root ; annuaire ne prend
pas le contexte ; attachement extérieur accepté sans motif ; ROOT extérieur
réveillé ; pas de racine ni motif dans le lot ; transport omet les options.

Passages RED supplémentaires, avant leur correctif :

- 111 tests, 5 failures/1 error : motif invalide CLI, dry-run hors projet accepté,
  absence de dispatch_delivery durable.
- 112 tests, 2 failures/1 error : contrôles au bord supprimés par trim ; défaut
  argparse traité comme motif présent.
- 113 tests, 1 failure : reçu de livraison remet review en dispatched.
- 113 tests, 1 error : décision ROOT sans code de motif requis.
- 116 tests, 1 failure : résultat arrivé durant résolution écrasé.
- 118 tests, 2 failures : résultat canonique archivé avant CAS ; warning perdu.
- 119 tests, 1 failure : deux missions de motifs différents fusionnées sous le
  premier motif. Assertions : task=two dans le digest de task=one, interdit.
- 121 tests, 1 failure : publication réelle via write_task_result après retour
  du CAS et avant l'archivage extérieur ; result_path canonique pointait vers
  un fichier qui venait d'être déplacé. Résidu découvert en contre-revue T020.
- 125 tests, 3 failures/1 error : panne injectée de journal après rename,
  panne de JSON de tâche après archivage, publication échouée avec ancien
  document présent ou absent. Les contenus et chemins réels étaient incorrects.
- 127 tests, 1 failure : rename de l'archive échouait avant la publication ;
  l'ancien document était supprimé par le rollback. Un booléen result_written
  limite maintenant le retrait au document effectivement écrit par l'opération.
  Le test de panne write_json(rp) vérifie aussi présence/absence initiale exactes.

Le test réel de deux réservations concurrentes était déjà GREEN avec la garde
updated_at initiale ; aucun RED artificiel n'est revendiqué pour ce test.

## T012 — Implémentation et assertions observables

Le run conserve project_root séparément de domain. L'annuaire de suggestions
est local ; legacy sans racine donne zéro suggestion sans consulter le cwd.
Le global sert seulement à résoudre ou observer une cible déjà déclarée.
Les faits same/other/unknown viennent du daemon, pas du domaine affiché.

Le mandat contient UUID, rôle et motif borné. Modifier UUID ou rôle ne transporte
pas le mandat. ROOT extérieur sans mandat produit une décision durable avec
reason_codes=[cross_project_reason_required], aucun message extérieur. Worker,
coordinateur et ROOT explicitement mandatés reçoivent leurs trois rappels, puis
silence. Les agents busy restent connectés ; ils ne justifient aucun fallback.

Les lots de heartbeat et de dispatch figent racine, motif, cible, corps, id,
date et issuer_scope avant la première tentative. Une modification du mandat,
du run, du destinataire ou du corps après erreur ne change pas le rejeu.
Une reprise de remise ne consomme pas une nouvelle tentative de mission.

La réservation initiale relit sous flock et compare le snapshot de statut,
résultat, enveloppe, tentative, cible, rôle et mandat avant tout archivage.
L'archivage et la réservation sont maintenant dans cette même section critique.
La publication interne write_task_result écrit le document et le verdict sous
ce même verrou. Le test de deux threads bloque volontairement l'archivage : le
writer concurrent attend ; seule l'ancienne preuve est archivée, la nouvelle
publication reste ensuite canonique avec son contenu exact et son verdict.
La panne E/S avant le commit du JSON de tâche remet maintenant l'archive à son
chemin canonique sous ce même verrou. Une publication interne conserve l'ancien
document dans le mécanisme d'archive existant jusqu'au commit. Si elle échoue,
l'ancien document exact revient ; si aucun document n'existait, l'absence
initiale revient. Les assertions comparent tous les octets de tâche et résultat.
L'événement d'archive intervient après le commit : une panne de journal produit
un warning fermé, sans défaire le verdict ni le chemin d'archive valide.
Le test avec deux threads et une Barrier impose deux lectures pending : un seul
envoi, une tentative, un refus concurrent. Le résultat apparu durant lookup
reste canonique et non archivé. Le résultat review apparu durant remise garde
son verdict et son result_path lorsque le reçu est enregistré.

Un digest ne fusionne que des contextes racine/motif identiques. Deux mandats
différents pour le même UUID utilisent deferred existant : R1 au premier tick,
R2 au deuxième, silence au troisième. Toujours un digest par UUID et par tick.

Le transport passe les options structurées depuis le lot figé et retire toute
identité T3 héritée. Seules les lignes AVERTISSEMENT PROJET avec JSON de code
cross_project/project_unknown sont rendues au caller. Le stderr libre et les
champs body supplémentaires restent absents du rendu. Le corps livré est exact.

## T013 — Skill et copie Claude

Skill modifiée : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/SKILL.md.
Le texte conserve l'exception volontaire, distingue global du mandat et décrit
les reprises figées. Aucun nouveau moteur de contrôle ou transport.

Wrapper : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/claude/.claude/skills/agent-loop/scripts/agent_loop.py.
Il résout le canon voisin dans le dépôt si présent, sinon le canon installé.
Un test subprocess exécute réellement init dans un dossier temporaire avec
--project-root et contrôle le run JSON ; il ne charge pas la skill vivante.

Commandes réelles depuis /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project :

```text
python3 /Users/moi/.codex/skills/.system/skill-creator/scripts/quick_validate.py codex/.codex/skills/agent-loop
python3 /Users/moi/.codex/skills/.system/skill-creator/scripts/quick_validate.py claude/.claude/skills/agent-loop
python3 -m py_compile codex/.codex/skills/agent-loop/scripts/agent_loop.py claude/.claude/skills/agent-loop/scripts/agent_loop.py
git diff --check
```

Résultats : deux Skill is valid!, exit0 ; compilation Python exit0 ; diff exit0.

## Dernier GREEN

Commande complète depuis /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop :

```text
python3 -m unittest discover -s tests
Ran 152 tests
OK
```

Exit0, aucun skip dans ce worktree. 124 tests existants et 28 nouveaux tests138.
Le dernier test Claude effectue un vrai subprocess local. Les autres transports
sont simulés, les fichiers et verrous sont réels. La recette avec le daemon
Rust réel et son binaire isolé relève de T017 ; elle n'est pas revendiquée ici.
Les dry-runs comparent tous les octets du run et n'appellent aucun envoi.

## Responsabilités et charge future

validate_cross_project_reason centralise la même borne pour les CLI, mandats
et lots ; bridget_scope_context contrôle UUID/rôle avec les faits de l'annuaire.
Elles ont plusieurs callers réels.
Trois fonctions de dispatch remplacent la grosse branche existing_bridget :
prepare_bridget_dispatch vérifie cible/rejeu/mandat et prépare l'enveloppe ;
reserve_bridget_dispatch conserve la réservation CAS et l'archive atomiques ;
dispatch_existing_bridget effectue dry-run, remise et reçu séparé du verdict.
Chacune possède un caller réel. Aucun backend historique n'a été refactorisé.
Le reste étend les responsabilités existantes de lecture, réservation et remise.
expected_fields renforce update_task sous le même flock ; aucun verrou parallèle.
archive_result/result_document associent le fichier résultat à l'écriture de
tâche sous ce même verrou. Les publications internes participent au protocole ;
un writer externe qui écrit directement les fichiers sans verrou n'est pas
présenté comme couvert par cette garantie.

## Bornes de T020

La réservation et le rollback couplés concernent la remise initiale
existing_bridget et la publication commune write_task_result. Les chemins
legacy existing_tmux/background_process/llm_process/spawn_tmux non modifiés
appellent encore l'archive séparément avant leur écriture ultérieure de tâche.
Ils ne sont pas présentés comme convertis à cette transaction de réservation.
L'archive directe restaure toutefois son fichier si son événement échoue.

Les tests injectent des OSError ordinaires avant commit. Ils n'éteignent pas
la machine et ne simulent pas la disparition du volume pendant le rollback.
Un crash entre les remplacements de fichiers n'est pas une transaction multi-
fichiers atomique. Une panne du rollback lui-même peut laisser l'ancien document
dans son archive de sauvegarde, sans garantir son retour au chemin canonique.
Il ne faut donc pas annoncer une garantie universelle de panne stockage ou de
writer externe. Aucun nouveau journal de transaction n'a été introduit.

## Correction de qualité T017/T018

Gate anti-doublon complémentaire, relue après l'extraction sans masquer son ordre :
la lecture préparatoire avait retrouvé la branche cmd_dispatch et les helpers
update_task/send_bridget_message avant le code. Le contrôle nominal de base
git grep -n -E '^def (prepare_bridget_dispatch|reserve_bridget_dispatch|dispatch_existing_bridget)\('
sur HEAD et les deux paquets Agent Loop retourne exit1, aucun helper homonyme.
La recherche nominale non ancrée retrouve seulement le test existant
test_dispatch_existing_bridget_records_uuid_and_sends_once ; ce test n'est pas
un helper de préparation, de réservation ou de transport.
La recherche par responsabilité retrouve les éléments suivants dans HEAD :
update_task:621, send_bridget_message:1059, cmd_dispatch:1875 et sa branche
existing_bridget:1966, send_orchestrator_message:3178. start_background_process
et start_llm_process lancent des processus ; ils ne réservent ni ne rejouent une
remise Bridget. Le wrapper Claude délègue au canon et n'offre pas un autre moteur.

Décision : REUTILISER avec extraction ciblée. La préparation provient de la
branche existante ; la réservation appelle le même update_task, sans recopier
son verrou ou son rollback ; la remise appelle le même send_bridget_message,
sans créer un transport ou un moteur de mission. Le registre, les champs et les
résultats restent ceux de la branche. Aucun nouveau fichier ni framework.

Commande réelle depuis /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project :

```text
radon cc codex/.codex/skills/agent-loop/scripts/agent_loop.py -s
```

Avant extraction ciblée : cmd_dispatch139, update_task21 ; la base communiquée
par le principal mesurait cmd_dispatch113. Après : cmd_dispatch104,
prepare_bridget_dispatch23, reserve_bridget_dispatch10,
dispatch_existing_bridget6, update_task21. Chaque nouvelle fonction reste sous25.
La transaction T020 n'est pas déplacée hors du verrou et conserve son rollback.
Les 152 tests restent PASS après extraction ; py_compile et diff --check PASS.
La complexité historique des autres backends n'est pas présentée comme résolue.
Annuaire indexé une fois par passage, décisions et deferred existants réutilisés.
Aucune nouvelle dépendance ou migration de mission.

Complément Article XVIII : les trois docstrings explicitent le coût local de
préparation O(A + taille du message), de réservation O(octets des fichiers
déclarés + JSON de tâche), puis de remise O(taille du message + réponse) après
préparation/réservation. Elles ne bornent ni le réseau, ni l'attente du verrou,
ni la latence du stockage. Contrôle après ces seules annotations : 152 tests
PASS en 1,737 s ; py_compile et diff --check PASS. Mesures finales inchangées :
cmd_dispatch104, prepare23, reserve10, dispatch6, update_task21.
