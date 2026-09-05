# Spécification 089 — Bridget, noyau de communication autonome

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 089-communication-core
Titre: Extraire et consolider la communication inter-agents
Statut: In Progress
Priorité: P1
Tâches: 21/36 (58%)
Tests: 0/12 (0%)
Résumé: Conserver les communications éprouvées, leur reprise et leur fédération SSH, sans dépendance obligatoire à une interface graphique ou à Maicie.
Fichiers: spec.md, plan.md, tasks.md, research.md, data-model.md, contracts/communication.md, quickstart.md, implementation.md
<!-- SPEC-FORMALISM:END -->

**Date :** 2026-09-05. **Périmètre approuvé par l'utilisateur**, conception à relire avant suppression de code. Branche : `session-089-communication-core`.

## Pourquoi

La valeur constatée par l'utilisateur est la communication entre agents de fournisseurs différents, d'abord rendue accessible par une skill. Le coût de développement du produit élargi a ensuite dépassé cette valeur d'usage. Il demande une extraction, pas un remplacement des mécanismes fiables par une démonstration minimale.

Le résultat doit permettre de se servir de Bridget sans développer une GUI, administrer un runtime Docker ou installer une coordinatrice. Bridget transporte des demandes, leurs réponses et les faits de livraison ; il ne garantit pas qu'un travail intellectuel est correct ou terminé. Maicie demeure une consommatrice extérieure possible de ces faits.

## Scénarios utilisateur

### US1 — Communiquer simplement entre fournisseurs (P1)

Un agent Codex envoie une demande à un agent Claude Code ; celui-ci répond avec l'identifiant de la demande, par outil MCP ou CLI. GLM via Claude Code reste un chemin admissible sans bascule forcée vers une API facturée.

- Étant donné deux sessions gérées, quand une demande suivie est livrée puis reçoit une réponse liée valide, alors la même demande devient answered et ne produit plus de rappel.
- Quand la réponse de transport est perdue, un retry aux mêmes paramètres n'injecte pas un deuxième prompt dans le domaine de garantie documenté.
- Quand une session n'offre pas une capacité, le refus la nomme avant lancement ; une absence de signal ne devient pas un état inventé.

### US2 — Continuer entre deux serveurs via SSH (P1)

- Un wrapper distant rejoint le même annuaire que les wrappers locaux par transfert de socket SSH vers un daemon maître unique.
- À la coupure du tunnel, l'état devient indisponible ou indéterminé selon les faits ; il ne devient ni livré ni terminé par déduction.
- Après reconnexion, l'identité stable et la corrélation survivent, le journal reprend au curseur conservé ou annonce sa lacune. Le processus fournisseur n'est pas relancé pour rétablir le seul tunnel.

### US3 — Observer et reprendre sans GUI (P1)

- `who`, `ledger` et `attach` exposent les faits du daemon, y compris à distance, sans lire une base locale vide en repli.
- L'annuaire distingue identité, nom affiché, mode de présence, protocole du pilote et transport réseau.
- L'attache repose sur la disponibilité attestée du journal, non sur une inférence « pas ACP donc tmux ». Les octets source et leur provenance survivent au relais.

### US4 — Un cœur petit et maintenable (P1)

- Le noyau compile et fonctionne sans les sources GUI, le plugin Maicie et les modules de runtime Docker de projet.
- Une modification de la communication ne nécessite plus de modifier un renderer web ou une règle de coordination métier.
- Une skill courte enseigne envoyer, répondre une fois, retrouver l'état et traiter honnêtement l'incertitude ; elle ne promet pas plus que le protocole.

## Exigences fonctionnelles

- **FR-08901 — Isolation :** nouveau dépôt indépendant dans `/Users/moi/Nextcloud/10.Scripts/XX.bridget`, historique conservé. Aucun fichier non committé, secret, registre personnel ou store de production importé automatiquement. L'ancien dépôt et son daemon restent intacts.
- **FR-08902 — Identité :** préserver identité stable, rename, annuaire, provenance et distinction mode/transport ; aucun scope dérivé du seul nom affiché.
- **FR-08903 — Envoi :** préserver canon, clé stable, issued_at immutable, horizon d'idempotence et refus déterministes. Une même clé divergente refuse sans mutation du premier record.
- **FR-08904 — Demandes :** préserver corrélation in_reply_to, timeout déclaré, rappel attesté et annulation. La clôture et ses faits durables restent atomiques ; une commande métier extérieure ne se déduit jamais d'un timeout.
- **FR-08905 — Historique :** chaque message livré entre une seule fois dans le ledger ; ses projections CLI/MCP proviennent d'une lecture commune du store maître.
- **FR-08906 — Pilotes :** conserver les pilotes natifs Codex/Claude et ACP existants derrière le contrat de session. Conserver modèles/efforts opaques, capacités déclarées, garde de facturation et permissions du fournisseur. Aucune dépendance opérationnelle à tmux pour la voie principale.
- **FR-08907 — Journal :** préserver source brute, ordre, curseur borné, rejeu puis suivi, Gap distinct de Unavailable et fraîcheur attestée. Ne pas confondre journal disponible et agent qui travaille.
- **FR-08908 — Fédération :** conserver SSH vers un daemon maître unique, socket distante explicitement choisie, reprise et projection distantes. Aucun serveur HTTP public supplémentaire ni consensus multi-maître.
- **FR-08909 — Surfaces :** CLI, MCP et skill utilisent le même contrat. Le niveau MCP actuellement épinglé n'est pas changé sous prétexte d'extraction. Les opérations retirées échouent explicitement, jamais par succès vide.
- **FR-08910 — Frontière Maicie :** supprimer la dépendance du daemon au crate métier Maicie tout en conservant, comme interface de communication négociée, les contrats publics guichet/événements nécessaires à un consommateur externe. Pas de choix d'objectif, d'approbation métier ni de politique de réassignation dans le noyau.
- **FR-08911 — Simplicité structurelle :** préserver les trois crates existants si suffisants ; séparer protocole pur, stockage transactionnel, sessions et adaptateurs de présentation. Un seul générateur de canon, un seul mécanisme de clôture, aucun renderer dans le stockage.
- **FR-08912 — Sécurité :** fichiers privés dès création, propriétaire/type vérifiés, refus des symlinks sensibles, identité/autorisation par connexion, limites de taille, files et délais globaux ; erreurs partielles d'envoi restent indéterminées, non assimilées à un refus certain.
- **FR-08913 — Reprise contrôlée :** ne pas ouvrir automatiquement le home/socket de l'ancien produit. Tout import ultérieur s'effectue depuis une copie validée, sans mutation de l'original. Toute bascule de production exige un accord distinct.
- **FR-08914 — Apports externes :** ne retenir d'A2A/LangGraph/ACP/MCP que des concepts avec un usage et un test nommés. Aucun framework, serveur A2A ou orchestrateur additionnel dans cette session.
- **FR-08915 — Environnement :** fonctionnement sans navigateur, serveur web, Docker ni binaire tmux installé ; pas de suppression des protections locales nécessaires sous le terme « simplification ».
- **FR-08916 — Contenus référencés :** inventorier et conserver les références/pièces jointes déjà utilisables comme communication (identifiant, contenu exact, provenance et accès), indépendamment de leur rendu. Retirer une prévisualisation HTML n'autorise pas à perdre le document qu'un agent transmet. Aucun nouveau système de fichiers partagé ou transfert supplémentaire inventé dans cette session.

## Critères de réussite (preuves à produire, pas encore acquises)

| Critère | Oracle mesurable |
|---|---|
| SC-08901 | Deux pilotes réels distincts : demande, réponse liée, une injection, un ledger et zéro rappel après clôture ; prompts/comptages attestés. |
| SC-08902 | Corpus CLI/MCP : mêmes bytes canoniques et mêmes issues, références conservées ; divergences body/cible/reply/deadline/in_reply_to refusées sans mutation. |
| SC-08903 | Crashs réels aux frontières de réservation/remise/ACK avec watchdog : résultat dans le domaine de garantie, mêmes bytes au retry, jamais succès inventé. |
| SC-08904 | Deux machines via SSH : who/ledger identiques au maître, coupure puis reprise, aucun double prompt ni redémarrage fournisseur indu. |
| SC-08905 | Attach réel : séquences continues à la jonction rejeu/suivi ; mutation Gap→Fresh ou Gap→Unavailable fait échouer l'oracle. |
| SC-08906 | Sans GUI/Maicie/Docker/tmux dans l'installation testée : communication et observation passent ; graphe de dépendances sans crate métier ni renderer. |
| SC-08907 | Fixtures de stockage historique sur copies : IDs/bytes/terminaux préservés ; source et production inchangées ; migration inconnue refusée. |
| SC-08908 | Matrice hostile : mauvais propriétaire/droits, symlink, absence de capacité, trame trop grande, connexion saturée, délai expiré ; refus attendu sans effet interdit. |
| SC-08909 | Codex natif, Claude natif et ACP : capacités/EOF/stop/reconnexion/attach ; preuve réelle GLM via Claude Code ou gate explicitement non validé. |
| SC-08910 | Skill/CLI/MCP enseignent et exposent les mêmes statuts, y compris in_flight ≠ outcome_unknown ; scénario court exécuté sans connaissance de Maicie. |
| SC-08911 | Corpus des tests de communication retenus : zéro test retiré/ignoré sans disposition motivée ; fmt, clippy, tests et audit dépendances consignés. |
| SC-08912 | Mesures avant/après sur même hôte : modules/dépendances de production diminuent effectivement ; p95 et ressources des scénarios locaux/SSH conservées dans les budgets historiques applicables. |

## Hors périmètre et limites assumées

GUI, fork/module T3, AG-UI, rendu HTML/artifacts, conteneurisation de projet, greffière, orchestration autonome métier, remplacement Routr, stockage des secrets fournisseur, nouveau protocole réseau public, multi-maître. Aucun abandon de l'accès par abonnement via CLI.

La garantie d'injection n'est pas « exactement une fois quoi qu'il arrive » : les limites du crash wrapper et de l'horizon doivent rester visibles (contrat de communication). SSH protège le canal, pas les actions d'un agent autorisé sous le même compte Unix. Un ACK ne certifie pas l'exécution correcte de la mission.

## Hypothèses et décisions

macOS pour la machine de travail, Linux pour le client fédéré ; Rust 1.92.0 existant. Les pilotes installés et les comptes d'abonnement sont à vérifier lors des gates réels, sans changer leur configuration silencieusement. Aucun calendrier en heures n'est promis avant la caractérisation du couplage. Les noms de champs filaires conservés font foi sur les libellés de cette spécification.
