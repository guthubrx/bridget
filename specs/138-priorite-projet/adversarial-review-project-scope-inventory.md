# Contre-revue du plan 138 — project_scope_inventory

Date: 2026-10-06
Relecteur: project_scope_inventory, modèle gpt-6-sol.
Nature: lecture indépendante du plan et de ses points d'extension.
Le relecteur appartient au même fournisseur que le principal.

## Disponibilité des autres fournisseurs

Le principal a consulté Bridget. Aucun agent d'un autre fournisseur admissible
dans le même projet n'était disponible. Le seul agent admissible connecté,
bdget, était Codex. La contre-revue inter-fournisseurs n'a donc pas été réalisée.
La lecture indépendante ci-dessous ne lui est pas présentée comme équivalente.

## Question et périmètre

Vérifier que le plan réduit les échanges involontaires hors projet tout en gardant
les mandats explicites, les clients historiques et les reprises idempotentes.
Chercher les preuves réelles du client de fond, de l'outbox et des négociations
anciennes. La revue ne modifie pas le produit et ne déclenche aucune dépense.

## Objections et traitement

| Objection | Vérification par le principal | Retenue | Action |
|---|---|---|---|
| La connexion background retire l'identité T3 ; le projet doit venir d'un contexte autonome | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:350, environnement background | Oui | Contexte Client propre négocié, racine/hôte validés sans Register ni identité empruntée ; plan/modèle/contrat et T004 corrigés |
| Une reprise de notification peut réutiliser un mandat courant modifié | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3602 et /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3626, création/reprise d'outbox | Oui | Figer corps, destinataire, racine et motif avant tentative ; T011 teste échec puis changement de mandat |
| Un ancien serveur peut ignorer le champ nouveau de BridgetMessage | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs:74, absence de deny_unknown_fields | Oui | Exiger la capacité138 côté client avant de transmettre une enveloppe à motif ; pas de downgrade ni supposition de refus serveur |

## Résultat après correction

Les trois objections sont vérifiables et retenues. Le principal a corrigé le
plan, le modèle et le contrat. Les tâches et le Gherkin reprennent leurs assertions.
La seconde lecture documentaire ne conserve aucun CRITICAL ouvert. Les tests
doivent encore prouver les comportements avant une validation du produit.

La portée projet est une garde de contexte, pas une nouvelle frontière de sécurité.
Un warning est établi avant l'effet et retourné au caller après traitement.
Il ne provoque pas de confirmation humaine répétée pour une demande autorisée.

## Revues d'implémentation — historique distinct du plan

Les verdicts suivants sont rapportés et confirmés par le principal. Ils ne
remplacent pas la contre-revue documentaire historique ci-dessus. Ils ne sont
pas présentés comme des revues inter-fournisseurs.

| Revue | Verdict | Périmètre et borne |
|---|---|---|
| Rust pré-T019 | APPROVE | Revue indépendante du Rust disponible avant l'ajout du contrôle SteerCurrent ; même fournisseur. Ne couvre ni T019, ni la migration de warnings, ni les correctifs Loop ultérieurs, ni le workspace complet |
| Corrective Agent Loop T020 | APPROVE | Revue indépendante après correctifs publication/archive, pannes E/S ordinaires et tests interleave/CAS ; confirmée par le principal après sa propre lecture des sources/tests et son exécution152 PASS, exit0 |

La revue Loop a successivement retenu la course entre unlock/CAS et archive,
puis la panne E/S après archive et avant écriture de tâche. Les tests RED ont
précédé leurs correctifs. Publication et réservation réutilisent le verrou
commun. Le document antérieur est restauré avant commit sur les pannes E/S
ordinaires testées. Après commit, la panne de journal produit un warning fermé.

Preuves relues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop.md
et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-final-validation.log.
Résultat final : 152 tests, dont 124 existants et 28 nouveaux, OK, exit0 confirmé.
T020 est clôturée sur autorisation du principal ; T016 demeure ouverte.

Limites explicitement conservées : aucun crash machine entre remplacements,
disparition de volume ou rollback lui-même défaillant n'est garanti. Un rollback
échoué peut laisser l'ancien document dans son archive de sauvegarde. Les writers
externes sans verrou restent hors contrat. Les chemins legacy existing_tmux,
background_process, llm_process et spawn_tmux ne sont pas convertis à la
réservation couplée. La corrective vise existing_bridget initial et write_task_result.
Cette revue bornée ne constitue pas un PASS global du produit138.

## Revue indépendante T019 après implémentation

Verdict APPROVE confirmé par le principal. Le relecteur indépendant reste du
même fournisseur ; aucune revue inter-fournisseurs n'est revendiquée.
Cette lecture est bornée à la garde SteerCurrent.message, aux refus fermés
durables, au warning caller hors corps, à sa persistance dans la table de contrôle
existante et au rejeu accepté avant la garde mutable. Elle ne valide pas à elle
seule le workspace, clippy, release ou la convergence finale.

Les preuves relues par le principal comprennent
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core-green-final.log
(25 PASS) et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core-control-real-green.log
(1 PASS processus réel avec restart ; 1 helper ignoré). Les refus et le résultat
accepté restent durables ; le corps est exact et aucune seconde remise n'est créée.
T019 est clôturée sur autorisation du principal. T016 est aussi clôturée après
validation des skills et de la recette CLI/MCP/Agent Loop réelle. T017/T018
restent ouvertes, produit global non encore validé.

## Revues finales avant la phase suivante

Les verdicts suivants sont confirmés par le principal après sa lecture des
sources, tests et captures. Tous les relecteurs relèvent du même fournisseur.
Aucune revue inter-fournisseurs n'est présentée comme réalisée.

| Lecture indépendante | Verdict confirmé | Périmètre et limite |
|---|---|---|
| Deux dépôts Git réels homonymes et extraction Agent Loop | APPROVE | Test homonyme réel1 PASS ; responsabilités préparation/réservation/remise réutilisées ; complexité139→104, base113, nouvelles fonctions23/10/6 ; aucun framework |
| Legacy089 concurrence/skill | APPROVE | Contrats legacy et concurrence relus ; oracles conservés ; ce verdict ne couvre pas tous les résultats du workspaceV4 |
| Synchronisations des tests transport | APPROVE | Adaptations de synchronisation relues sans supprimer les oracles ni modifier la logique de production |
| Scan sécurité/fiabilité/minimalisme | Aucun défaut de production supplémentaire prouvé | Lecture bornée aux sources et preuves disponibles ; aucune garantie universelle d'absence de défaut |

Le principal a validé152 tests Python après extraction et la recette réelle
Agent Loop/CLI/daemon :1 PASS, zéro FAIL, aucun ignoré. Le build release est PASS.
Le comptage unique138 est35 Rust+28 Python=63, sans addition des filtres répétés.
Preuves relues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core-homonyms-green.log,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-after-extraction-final.log,
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop-real-after-extraction.log,
et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/release-final.log.

Le workspaceV4, annoncé en cours au signal initial, a ensuite rendu un FAIL
dans managed_parity_test à la relecture de sa capture. Le principal relève
une relance légitime au cinquième tour. Aucun verdict favorable de ces revues
n'efface cet échec ni ne prédit le résultat de son diagnostic. T017/T018 restent
ouvertes ; aucun PASS global du produit138 n'est annoncé.

Bornes T020 maintenues : initialexisting_bridget et write_task_result seulement,
pannes OSError ordinaires testées, verrou commun. Ni crash machine entre
remplacements, volume disparu, rollback lui-même défaillant, writer externe
sans verrou ou conversion des autres backends legacy ne sont garantis.
Ces limites empêchent d'étendre le constat adverse à une isolation de sécurité
ou à une transaction multi-fichiers universelle. La revue reste une garde de
contexte et de fiabilité dans le périmètre choisi.

Gel documentaire de cette lecture annoncé au principal avant la phase suivante.
Aucun verdict Converge ou audit, aucun résultat futur, aucune clôtureT017/T018.

## Revue de clôture après Converge pass1 et audit validé

Verdict indépendant confirmé par le principal : APPROVE borné, même fournisseur uniquement. Aucun cas exigé oublié confirmé. Pas de revue inter-fournisseur ni nouveau test revendiqué.

- Fil A/B/U et notify[] : spec102_threads_test.rs:130 vérifie refus avant dépôt/ACK ; U n'annule pas le cross connu.
- Looptests:167/239 vérifient la reprise de l'enveloppe après mutation du mandat, racine, corps et cible.
- Looptests:86/125 vérifient ROOT non mandaté sans envoi et les trois rappels mandatés.
- Managed parity final (+106/−10) : quatre IDs métier stricts ; rappels système attestés SQL, génération et corps exacts ; journal complet vérifié. APPROVE ; aucun filtre libre, aucune relance production désactivée.

V5 confirmé :1633 PASS/0 FAIL/55 ignored (78 résumés externes filtered0). Python152 PASS ; opt-in réel final1 PASS ; fmt/clippy/release exit0 ; skills valides. V4 et les RED réels restent historiques.63 tests138 uniques sont un sous-ensemble, pas une addition aux suites.29 Gherkin écrits, non exécutés.

Audit v14 relu par le principal : A98,333 du diff,0C/H,5MED de maintenance ouverts non bloquants. Validator canonique0erreur/0warning,exit0 ; compteur de contexte177 corrigé puis revalidé. Les recommandations d'extraction et d'annotation n'ouvrent aucune exigence manquante. A diff n'est pas A dépôt entier. Phase9 non engagée, pas de correction audit ni commit.

Les limites T020 restent celles du verrou commun et des pannes E/S ordinaires des writers participants. Crashmachine, volume perdu, rollback défaillant et backends anciens non convertis restent hors garantie. UNKNOWN legacy averti reste possible. Ce verdict n'est ni une certification de sécurité ni une autorisation de déploiement. T018 clôturée après l'audit validé ; état actuel20/20 Implemented. Les passages précédents18/20 et « ouvert » restent des snapshots historiques.
