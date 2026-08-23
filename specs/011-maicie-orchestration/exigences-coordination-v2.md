# Exigences amont pour Maicie v2+ — jurisprudence d'orchestration

Extrait de la coordination manuelle des sessions 007-012 et 011 (2026-08-22/23),
tenue par le référent humain-agent. Chaque règle exécutée à la main ici est une
candidate à devenir un comportement produit de Maicie. Hors périmètre de la
v1 (spec 011) — intrants pour les itérations suivantes.

## Politiques de délégation observées (candidates à l'automatisation)

1. **Tout événement attendu a un messager.** Un commit, une fin de banc, un
   verdict sont des événements muets : personne n'est réveillé par git. La
   règle manuelle « livraison = hash annoncé par message » devrait devenir :
   Maicie observe l'attendu (par abonnement ACP ou déclaration) et notifie
   les dépendants elle-même.
2. **Déblocage des dépendants.** Quand une délégation se termine, la
   coordinatrice sait qui attendait quoi (graphe de dépendances des
   délégations d'un même objectif) et débloque sans qu'on le lui demande.
3. **Réassignation après N relances sans livraison.** Politique manuelle
   éprouvée : 2 relances factuelles sans commit → réassignation. Maicie
   pourrait porter cette politique par classe de délai (config T003).
4. **Contrat d'interface avant commit du fournisseur.** Quand deux
   délégations partagent une frontière, la coordinatrice fait négocier
   l'interface AVANT l'implémentation (observé : API store T006↔T008 —
   zéro STOP de review sur la frontière ensuite).
5. **Propriété des ressources partagées par phase.** Un fichier commun a un
   propriétaire unique par phase ; les autres passent commande. Maicie
   pourrait tenir ce registre de propriété par objectif.
6. **Vérification factuelle avant relais.** Aucun verdict/blocage relayé sans
   vérification indépendante (grep/exécution). Pour Maicie : ne jamais
   propager un état déclaré sans observable (transport_snapshot, fraîcheur).
7. **Diffusion de groupe.** Le transport reste point-à-point à dessein ;
   « informer tous les participants d'un objectif » est une sémantique
   Maicie (qui doit savoir, qui doit accuser) — pas une primitive transport.
8. **Escalade humaine bornée.** Les questions remontées à l'humain pendant la
   nuit : gates externes (SSH), arbitrages de périmètre, dérogations. Tout le
   reste s'est arbitré au niveau coordination — bon partage à conserver.

## Anti-patterns constatés (à ne pas reproduire dans Maicie)

- Polling aveugle (sleep + vérifier) au lieu d'attente événementielle.
- Relance d'un agent qui a déjà répondu par un canal non lié (cause :
  corrélation perdue — d'où l'exigence d'identifiants intégraux de bout en
  bout, réglée côté transport le 2026-08-23).
- Double réponse à une même demande après rappel.

## Exigence amont Phase 5 (question utilisateur, 2026-08-23)

**Un profil approuvé DOIT rendre visibles la nature et l'intensité de l'agent
qu'il fait naître** : type (codex/claude/...), modèle, niveau d'effort — car
c'est une décision de gouvernance et de coût d'abonnement, qui appartient à
l'écran d'approbation, pas à une config enfouie. Voie d'implémentation
minimale (zéro code Bridget) : les profils référencent des TYPES du registre
agents.json, et les variantes modèle/effort sont des entrées de registre
dédiées (ex. codex-effort-haut = args -c model_reasoning_effort="high") —
le profil Maicie affiche le type et ses paramètres épinglés à l'approbation.
Évolution Bridget possible plus tard : paramètres modèle/effort par spawn.
À intégrer à T021 (validation des profils : champs type/modèle/effort
obligatoires dans la fixture).

**Compléments (dialogue utilisateur, suite)** : (a) le pin gpt-5.5 des
équipiers codex est une CONTRAINTE de l'adaptateur codex-acp@0.16.0 (son cœur
refusait gpt-5.6-* au spike 007), pas un choix — à RE-TESTER à chaque montée
de version de l'adaptateur (vérification périodique, les 5.6 terra/luna/sol
sont la préférence utilisateur) ; (b) la palette cible est PAR CLASSE DE
TÂCHE : revue hostile/arbitrage = haut calibre (fable ou 5.6 effort haut),
codage = 5.6 effort haut, mécanique/smoke = modèle léger effort bas — jamais
de haut calibre pour du travail de greffe. L'agent coordinateur propose le
profil par nature de tâche ; l'humain approuve la palette ; Maicie n'accepte
que l'approuvé.

## Amendement post-revue adverse de la palette (2026-08-23, fable-reviewer)

Verdict : cohérente avec conditions, UN amendement structurant — la
revendication « zéro code Bridget » est abandonnée : l'intégrité exige une
surface publique Bridget minimale.

**Prérequis Bridget AVANT la Phase 5 (T021)** :
1. Écho de la DÉFINITION RÉSOLUE (command/args/forbidden_env + digest) dans
   SpawnAccepted (ou digest dans l'annuaire) — Maicie l'épingle dans son
   context_hash d'approbation et revalide au TOCTOU. Ferme les deux vecteurs
   de dérive : édition d'agents.json ET montée de version du binaire
   (défauts compilés).
2. Warning au chargement d'une entrée utilisateur SANS forbidden_env quand le
   défaut du même type en a un (la fusion extend() remplace tout — preuve :
   test user_entry_replaces_its_default) ; T021 exige forbidden_env NON VIDE
   sur toute variante de fixture. C'est LA garde de facturation.
3. Durcissement lecture agents.json (permissions, refus symlink) — il entre
   dans la TCB d'approbation.

**Conditions d'écran d'approbation** : args VERBATIM de l'entrée résolue
(jamais les seuls champs déclaratifs) ; effort = étiquette OPAQUE par
fournisseur, AUCUNE échelle comparative normalisée ; mention honnête de la
limite du pin (« épinglé jusqu'à l'alias fournisseur, pas au-delà »).

**Gouvernance** : palette approuvée = approbation de CONFIGURATION
(mapping classe→profil) ; chaque activation reste mono-usage FR-014,
journalisée séparément. La classe de tâche est portée dans le motif de la
DécisionCoordination (auditable) ; reclassification = nouvelle décision
journalisée. Sonde de fumée post-spawn (tâche mécanique triviale) avant tout
contexte réel. Re-test du pin modèle = tâche NOMMÉE avec propriétaire à
chaque montée de version d'adaptateur. Compteur de spawns par classe/profil
(visibilité du sur-calibrage). Backlog v2 : plafond de spawns actifs par
objectif ; champ classe dans le modèle de données.

## Suivis post-revue hostile finale — C5 et C7

- **C5 — rendre l'approbation opposable côté Bridget** : la v1 émet
  `SpawnOrder` et `CancelRequest` par des connexions fraîches de rôle wrapper,
  parce que le rôle client négocié ne les admet pas. Ce rôle reste coopératif :
  tout processus local peut produire le même ordre, indépendamment d'une
  approbation Maicie. Candidat v2 : une session Bridget dédiée au client Maicie
  négocie une capacité `spawn` ; Bridget n'admet ces ordres qu'après cette
  négociation et devient l'autorité capable d'imposer la décision approuvée.
- **C7 — budget global d'activation** : la passe d'activation additionne encore
  des délais locaux de lookup, connexion, replay et accusé. Elle ne propage pas
  une échéance absolue unique comme la voie de délégation. Suivi mineur : borner
  toute la passe par un budget global consommé, sans modifier la sémantique
  idempotente de l'outbox.

## Conclusion T015b (2026-08-23, prospective — instruite après T017)

**NON : Subscribe seul ne suffit pas à la boucle de réponse.** Le journal
public expose le message_id du prompt mais pas l'in_reply_to de la réponse ;
le timeout est une issue de demande Bridget, pas un fait ACP observable.
**Évolution Bridget nommée, désormais prouvée nécessaire** (conforme au
principe « pas de surface avant preuve ») :
1. une identité Maicie JOIGNABLE (livraisons entrantes) ;
2. une surface publique de statut/événements de demande CORRÉLÉS
   (réponse liée, timeout typé).
C'est le prérequis de la boucle réponse→décision complète (Maicie v2 /
future session Bridget). D'ici là : issues de LIVRAISON seules (T015a),
consultation passive des échéances (T019), observations ACP datées (T017).

## Suivi Bridget (constaté le 2026-08-23, session 011 Phase 5/6)

- **Parité de repli du binaire — livrée par D24** : `bridget send` et
  `bridget reply` acceptent `--in-reply-to <id>`. Un agent privé de ses outils
  MCP peut donc lier son verdict à la demande suivie et arrêter ses rappels,
  par la même transition transactionnelle que la voie MCP.

## Reprise après crash total (revue adverse du 2026-08-23 — idée utilisateur)

Verdicts : (a) reprise Maicie DÉJÀ-COUVERTE par construction (T023) ;
(b) remonter la même équipe COHÉRENT-SOUS-CONDITIONS ; (c) résumé LLM de
session REJETÉ — remplacé par une carte de réveil déterministe.
Acquis important : FR-014 est déjà compatible — une REPRISE n'est pas une
NAISSANCE (l'approbation a eu lieu au spawn initial, le digest épinglé en est
la preuve durable) ; aucune « approbation en masse » à inventer.

Trous réels retenus :
1. **Composition d'équipe durable minimale** : le `domain` existe dans le
   protocole mais n'est jamais écrit dans fleet.json (perdu au crash) ;
   « la même équipe » = aujourd'hui « les mêmes binaires persistants dans les
   mêmes cwd », sans rôles ni missions. Écrire le domain suffit — pas de
   nouvel objet.
2. **Pertes silencieuses à la reprise** : le quota de flotte ampute les
   surnuméraires de fleet.json sans autre trace qu'un log, et les agents
   non-persistants disparaissent du monde d'après. Exiger une trace durable
   consultable : « ces N équipiers ne sont PAS revenus, et voici pourquoi ».
3. **Chaînon opérationnel Maicie post-crash — mise à jour 2026-08-23** :
   `maicie profile propose` et `maicie profile approve` sont maintenant exposés
   par le CLI, mais `maicie profile refuse` reste absent ; ne pas simuler ce
   refus et conserver son ajout comme écart explicite. La réconciliation après
   reboot reste pull-only : Maicie ne repart que lorsqu'on l'invoque.
4. **Carte de réveil déterministe** (remplace l'idée de résumé) : à la
   relance d'un wrapper, la session ACP est neuve et muette — l'agent ignore
   qu'il est en reprise. Injecter des FAITS BRUTS, zéro LLM : génération N,
   cwd, K demandes ouvertes rerattachées, chemin absolu du handoff, pointeur
   bridget_ledger. Des pointeurs, pas de la prose.

Gadgets rejetés : résumé LLM (par quiconque), résumé continu par tour,
approbation en lot, graphe d'ordre de relance, snapshot d'équipe séparé de
fleet.json, replay de contexte automatique au-delà de la carte de réveil.
- ✅ LIVRÉ (bloc C : Rust 1.92.0 épinglé + rustfmt assumé) — **Toolchain non épinglée** (constaté 2026-08-23) : aucun rust-toolchain.toml
  au repo ; la stable locale (plus récente que le style committé) fait de tout
  `cargo fmt` un générateur de bruit (réordonnancement d'imports style 2024).
  Un WIP fantôme main.rs/store.rs en est né et a été jeté. Épingler la
  toolchain, puis un commit rustfmt unique et assumé.
- ✅ LIVRÉ (014, amorçage causal testé) — **Préambule d'identité sauté sur `codex resume`** (constaté 2026-08-23 soir) :
  l'heuristique has_prompt du wrapper (wrapper.rs:807) traite tout argument
  sans tirets comme un prompt utilisateur ; la sous-commande codex `resume`
  déclenche donc le silence — `bridget claude --resume` reçoit son préambule,
  `bridget codex resume` non. CAUSE RACINE COMPLÈTE (enquête cxbridget,
  2026-08-23 soir, preuves logs SQLite + probe stdio) : l'override -c
  mcp_servers traverse `resume` sans perte et les outils sont bien servis,
  mais Codex 0.149 les DIFFÈRE (ToolSearchAlwaysDeferMcpTools) — ils exigent
  une découverte via ALL_TOOLS puis un appel tools.mcp__bridget__*. Sans le
  prompt d'amorçage (sauté par has_prompt), l'agent repris ne refait jamais
  la découverte et conclut à tort que l'outil manque. Correctif spécifié :
  parser les sous-commandes codex (resume, SESSION_ID et options ≠ prompt),
  et injecter en reprise un bootstrap court expliquant la découverte
  différée ; tests d'argv et de régression du texte. Recoupe la « carte de
  réveil » : même besoin, même endroit d'injection. PROMU EN SESSION 014
  (tâche 6 — décision utilisateur du 2026-08-23 soir, après la chasse au
  fantôme MCP) ; la carte de réveil complète (génération, demandes
  rerattachées, handoff) reste v2.
- ✅ LIVRÉ (014) — **`who` n'affiche pas le mode d'attelage** (confusion utilisateur constatée
  2×, 2026-08-23) : la colonne transport ne décrit que le tronçon
  daemon↔wrapper (unix/ssh) ; rien n'indique si l'équipier est géré ACP ou
  interactif tmux, et les claude ne remontent ni modèle ni effort (sonde
  runtime codex-only). Ajouter une colonne mode (acp/tmux/cli) et étendre la
  sonde aux claude.
- ✅ TRAITÉ (fleet.json né le soir même ; doctrine --persistent adoptée ; première résurrection automatique constatée) — **Aucun équipier persistant en pratique** (constaté 2026-08-23 soir) :
  avant le 2026-08-23 21h, aucun spawn n'avait jamais utilisé --persistent
  (fleet.json créé ce soir-là par le premier spawn persistant — fable-reviewer) ;
  auparavant, après un reboot machine, zéro équipier géré ne revenait : la condition (b) de la
  revue reprise-crash est fausse aujourd'hui, empiriquement. Doctrine à
  adopter : --persistent par défaut pour les équipiers d'équipe.
- ⏳ RELANCÉE (2026-08-23 (soir), G1502 franchi — engagement honoré, mission-d23-coder-2 au greffe) — **Binaire périmé : détection ET rituel** (incident ×2 le 2026-08-23 — 8h40
  de retard l'après-midi, puis le daemon pré-014 servant encore le refus
  menteur le soir même, DEUX HEURES après le merge du correctif). Deux
  volets : (1) DÉTECTION — build-id (hash git) embarqué à la compilation,
  échangé au hello ; tout client dont le build-id diverge de celui du daemon
  affiche un avertissement franc « daemon périmé (X vs Y) : launchctl
  kickstart -k … » ; le même écart apparaît dans who/status ; (2) RITUEL de
  clôture de session — quand un merge touche les crates daemon/wrapper, le
  redémarrage du daemon fait partie de la checklist de clôture, au même titre
  que le rebuild ; consigné comme étape, pas comme souvenir. La résurrection
  automatique des persistants (validée en production ce soir : fable-reviewer
  gen-11) rend ce redémarrage indolore.
- ⏳ PARTIEL (corrélation toolCallId livrée en 014 ; séparateur de génération et titres enrichis restent ouverts) — **Vue attach : générations mélangées et titres d'adaptateur nus** (séance
  d'observation utilisateur, 2026-08-23 soir) : le journal par nom+jour rejoue
  l'archéologie du prédécesseur sans délimiteur (« inconnu »/UTC = vieux
  binaire, normal mais illisible) — ajouter un séparateur de génération au
  rattrapage (recoupe la carte de réveil) ; et enrichir les titres vagues de
  l'adaptateur claude (« Terminal ») d'un extrait de commande, comme FR-003
  le visait (« Bash cargo test… »).

## GUI Bridget (vision utilisateur du 2026-08-23 — mélange grokbot/Cursor)

Client graphique : barre latérale d'agents (who), fil de conversation par
agent (ledger, réponses liées repliables), vue intérieure par tour (journal :
narration, agrégats d'activité calculés côté client, outils dépliables),
composer humain (send rôle local). ~80 % = présentation sur flux existants.

Delta backend, par taille :
1. Session « journal enrichi » (wrapper) : capturer la pensée si l'adaptateur
   l'émet (best effort par fournisseur), les arguments et les RÉSULTATS
   d'outils (troncature bornée) — aujourd'hui seuls l'appel et son statut
   sont journalisés.
2. Abonnement ledger/who (aujourd'hui pull ; polling acceptable jour 1).
3. Agrégateur de lecture local fusionnant who+ledger+journaux par
   conversation.
4. Hérite de la 014 (corrélation toolCallId, heure locale, mode, sonde
   claude). Hors périmètre assumé : changer le modèle d'une session en cours
   (appartient au CLI de l'agent).
   Client : DÉCISION GRAVÉE (ADR 009, 2026-08-23, après double
   reconnaissance dont une sur arbre frais b1670ac7d) — GUI PROPRE
   PROGRESSIVE, Bridget/Maicie plan de contrôle unique ; T3 Code = référence
   UX et carrière de matériaux MIT (emprunts ciblés type effect-acp), PAS de
   fork produit (aucun point d'injection : catalogue de drivers compilé,
   client sans plugins ; dérive upstream ~1 589 commits/4 mois) ; fork très
   court admis en POC jetable seulement. Exécution strictement locale.
5. **Vue unifiée quel que soit le canal ET le mode** (exigence utilisateur
   2026-08-23) : même vue attach/GUI pour les gérés ACP et les interactifs
   tmux. Chemin : généraliser la sonde runtime existante (parse_codex_rollout,
   déjà en prod pour modèle/effort) en traducteur complet — suivre le fichier
   de session du CLI (rollout codex, JSONL claude) et normaliser vers le
   journal v1. Identité des vues garantie par construction (un read-model,
   N renderers). Assumé : parseurs couplés à des formats tiers (best effort
   versionné, dégradation annoncée), latence fichier de quelques secondes.

## Périmètre gelé de la session 014 — Observabilité & reprise (2026-08-23)

**✅ LIVRÉ INTÉGRALEMENT — session 014 mergée le 2026-08-23 soir (7/7, gate MERGEABLE).**

Six tâches validées sur le principe par l'utilisateur (spec à créer au « go ») :
1. Dissocier mode d'attelage et transport (champ surchargé, daemon.rs:1710).
2. Colonne mode (acp/tmux/cli) dans `who` + session:window.pane pour les tmux.
3. Sonde modèle/effort étendue aux claude.
4. **Corrélation toolCallId** : les mises à jour d'un appel titré héritent du
   titre — fin des rafales « inconnu » (suivi 013 resté conversationnel
   jusqu'ici, gravé maintenant).
5. **Heure locale** dans la vue attach (le suffixe UTC de T1305c ne satisfait
   pas l'utilisateur — exigence explicite).
6. Amorçage de reprise : sous-commandes codex hors has_prompt + bootstrap
   identité/outils MCP différés (cause racine 2026-08-23).

## Divers restés conversationnels — gravés le 2026-08-23 soir

- **Skill maicie — livrée le 2026-08-23** : pendant de la skill Bridget,
  installée dans `~/.claude/skills/maicie/` et `~/.codex/skills/maicie/` ; elle
  fixe quand utiliser delegate/status/objective, la doctrine des deux vérités,
  le replay exact comme lookup et l'approbation exclusivement humaine.
- **Décision utilisateur 1b (2026-08-23)** : la règle « zéro trace IA dans
  Git » s'applique aussi au contenu versionné. Les revues conservent la preuve
  d'un croisement par un moteur distinct, sans identifier le fournisseur ni le
  modèle du rédacteur ou du relecteur.
- **Routines planifiées** (vision grokbot, capture utilisateur 2026-08-23) :
  délégations RÉCURRENTES portées par Maicie (routine = delegate + calendrier,
  journalisée, idempotente, visible au greffe et dans le futur panneau GUI).
  Doctrine : une routine délègue, jamais n'approuve (FR-014 sans exception
  horaire). Point 35 du plan — bloc F.
- **Répartition socle GUI** : REMPLACÉE par l'ADR 009 (2026-08-23) après
  double reconnaissance — pas de fork produit, GUI propre progressive ;
  emprunts MIT ciblés au code source de T3 Code (effect-acp, patterns), la
  coquille et l'accès distant restant chez eux ; construire la couche équipe
  (who/ledger/fils liés/missions Maicie/routines). Voir
  docs/decisions/009-gui-plan-de-controle.md.
- **Mode « planifié » chez Maicie** : JUGÉ GADGET en l'état (contre-examen
  Article XIX du 2026-08-23, même soir que sa proposition) — doublerait la
  source de vérité du catalogue, ne supprime aucune friction mesurée
  (l'activation = une commande au « go »). Ne se justifiera qu'ADOSSÉ à un
  besoin porteur : les routines (une routine EST un objectif planifié
  récurrent) ou le panneau de planification GUI. À construire dans CES
  specs-là, jamais seul.

## Suivis post-gate 014 (fable-reviewer, 2026-08-23 soir — non bloquants)

- C1 arbitré : flakiness d'ordre de livraison sur poste QoS arrière-plan
  (préexistant, échoue aussi sur main) — 6/6 vert sur poste sain ; harnais
  receive_replies à durcir un jour comme C2-011.
- C2 : wrapper ACP ancien-binaire re-enregistré sans mode → attach fail-closed
  « mode inconnu » jusqu'à relance du wrapper (conforme FR-1401, à savoir).
- C3 : tool_titles jamais purgée (croissance non bornée sur session longue).
- C4 : options codex à valeur codées en dur (à surveiller aux montées codex).
- C5 : tmux_location capturée au lancement, périmée si pane déplacé.
- C6 : slug claude réimplémenté (dégradation honnête si convention diverge).
- C7 : BRIDGET_MVP_GATE_BIN exige un chemin absolu.

## Ordre de déroulé — amendé le 2026-08-23 soir (décision utilisateur)

Le bloc F est REMONTÉ avant la piste GUI : l'objectif prioritaire est
l'accélération (F27-29 automatisent le référent, condition du passage à une
équipe plus nombreuse). Ordre validé : C (fait) → D18 identité joignable
(prérequis technique de F) → F27-29 + gate mécanique → ÉLARGISSEMENT DE
L'ÉQUIPE → B (journal enrichi, traducteur, GUI) mené par l'équipe élargie,
F-reste et D-reste en fil d'eau. La GUI arrive plus tard mais se construit
plus vite ; le coût net est un report, pas une perte.
- **État « en pause » manquant chez Maicie** (constat utilisateur 2026-08-23 (soir)
  nuit) : une mission suspendue par arbitrage de priorité (bouche-trou D23
  interrompu par le lot A 015) garde son échéance qui court — le greffe la
  dira « en retard » quand la réalité dit « suspendue sur ordre ». Palliatif
  actuel : clôture administrative + re-délégation. Besoin : transition
  pause/reprise journalisée, échéance gelée. Cousin du mode planifié —
  même famille « états intermédiaires du travail », à traiter ENSEMBLE quand
  les routines l'exigeront.
- **outcome_unknown quasi systématique sur premier envoi MCP** (observé sur
  ~40 envois du référent, soirée du 23/08) : le contrat idempotent absorbe
  parfaitement (rejeu → accepted, zéro doublon), mais la latence d'accusé
  sous-jacente n'est ni expliquée ni mesurée — le filet fonctionne si bien
  qu'il masque la chute. Diagnostiquer (fenêtre d'accusé ? contention ?
  timeout client trop court ?) avant que le volume d'agents ne double le
  trafic de rejeu.
- **Maicie greffière du catalogue — CŒUR DE MAICIE, exigence produit**
  (décision utilisateur 2026-08-23 (soir) : « je voudrais éviter que ce soit une
  spécificité de notre session… je voudrais que ce soit dans le cœur de
  Maicie ») — bloc F, après 015. FORMULATION DIRECTRICE (utilisateur,
  2026-08-23 (soir)) : « Maicie écrit déjà tout ce qui a été fait [le greffe des
  missions]. Ce que je veux, c'est systématiser le RESTE À FAIRE — le
  "j'ai découvert, donc je dois faire". » Le journal du DÛ doit avoir la
  même rigueur que le journal du FAIT : entrées nées d'événements (mission,
  incident, review), liées à leur source, à états et transitions journalisés
  — Maicie PROPOSE FORTEMENT (tri sur faits : récurrence, sévérité déclarée,
  âge, gate raté), ne JUGE JAMAIS (la promotion en tâche reste au chief of
  staff/humain — même partage que FR-014/FR-022). Statut : ENGAGEMENT — la
  discipline du journal de bord ne doit dépendre d'aucune session, d'aucun
  référent, d'aucune convention locale ; elle sera un comportement livré,
  testé et documenté de Maicie, utilisable tel quel par toute équipe. l'hygiène du journal de bord collectif repose sur la
  discipline du référent, qui a prouvé ses limites (fichier de métriques
  jamais commité et perdu ; annotations d'état oubliées 2×). Design : PAS de
  base parallèle — le catalogue markdown versionné RESTE la source unique ;
  Maicie en devient la main : capture datée liée à la mission source
  (constat add), et transitions d'état AUTOMATIQUES (l'objectif qui livre un
  remède bascule l'entrée liée en LIVRÉ à sa clôture, commit compris —
  machine à états pure, zéro LLM). S'appuie sur le guichet 015 (l'événement
  de livraison est le déclencheur). Bonus : catalogue interrogeable (GUI) et
  métrologie incident→remède gratuite. AMENDÉE par revue adverse du
  2026-08-23 (soir) (revue-adverse-boucle-amelioration-2026-08-23 (soir).md) : tri sur
  champs DÉCLARÉS seulement, jamais d'écriture dans les plans de l'hôte,
  format d'entrée fermé À LIVRER D'ABORD (le catalogue actuel est de la
  prose non machine-appendable — migration requise), volet « entrée au
  plan » reporté aux routines. DÉCOUVRABILITÉ (question utilisateur
  2026-08-23 (soir)) : trois filets, aucun ne reposant sur une mémoire — (1) la
  skill maicie prescrit « registre list » en début de session et avant toute
  proposition de suite (motif DevKMS/mem context de la constitution) ;
  (2) pied de page DÉTERMINISTE des sorties Maicie aux jalons : « N constats
  OUVERT dont M récurrents, K liés à un gate raté » (un décompte, pas un
  avis — le rappel d'inbox de l'Article XIV, version greffe) ; (3) le rituel
  de clôture commence sa proposition de bloc suivant par la consultation de
  la vue triée.
