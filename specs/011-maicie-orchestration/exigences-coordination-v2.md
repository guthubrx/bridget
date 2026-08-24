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
- ✅ LIVRÉ (D23 mergé le 23/08 au soir : build-id + -dirty + avertissement d'écart who/status/CLI/MCP — le rituel de clôture a son détecteur automatique) — **Binaire périmé : détection ET rituel** (incident ×2 le 2026-08-23 — 8h40
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
- **État occupé/inactif invisible** (question utilisateur, 23/08 soir : « tu
  n'es pas censé voir quand ils ont fini ? ») : les FINS arrivent par la
  règle 8 (livraison = message), mais l'ENTRE-DEUX est aveugle — un agent
  entre deux tranches ou en attente silencieuse n'émet rien. Ironie
  documentée : RuntimeNature::Disponibilite/Idle existaient et furent purgés
  la même nuit comme code mort (C11) car AUCUN producteur ne les
  construisait. Les réintroduire uniquement ADOSSÉS à un producteur réel
  (sonde runtime : activité du transcript/rollout) — jamais en vocabulaire
  spéculatif. Sert who, l'agrégateur GUI et le F28.
- **Colonne MODÈLE aveugle pour les gérés** (constat utilisateur, 23/08
  soir, après le gréement codex-terra) : le modèle des équipiers gérés est
  DANS leur définition figée (scellée par digest) mais who n'affiche que la
  sonde runtime, qui ne couvre pas l'ACP → « — » alors que le daemon sait.
  Correctif minimal : pour un géré, afficher modèle/effort depuis la
  définition (source approuvée par l'humain) ; la sonde reste la voie des
  interactifs. Second constat lié : après un redémarrage daemon, les
  modèles des interactifs disparaissent jusqu'à leur prochaine activité
  (sonde pilotée par mtime) — acceptable mais à documenter dans who.
- **La présence riche écrasée par l'identité MCP** (observation utilisateur,
  23/08 soir — « j'ai vu le modèle de coder2 un instant puis tout a
  disparu ») : quand un équipier géré ENVOIE par son outil MCP, le serveur
  MCP de sa session revendique son identité par filiation et ré-enregistre
  une présence MINIMALE (transport unix, sans mode/domaine/modèle) qui
  écrase celle du wrapper. Contre-preuve : prospective2, jamais émetteur,
  garde sa présence intacte. Correctif : l'enregistrement par filiation doit
  FUSIONNER (compléter, jamais écraser les champs attestés existants) — ou
  être ignoré si une présence wrapper vit sur le même nom. Recoupe la
  lacune « modèle des gérés depuis la définition ».
- **Le référent, dernier maillon mortel** (question utilisateur, 23/08 soir :
  « c'est assuré comment tes réveils ? ») : les messages sont durables (file
  daemon) et la ronde de vigilance tourne — mais tout meurt avec la session
  interactive du référent, que seul l'humain relance. Candidat : référent en
  ÉQUIPIER GÉRÉ PERSISTANT (spawn claude --persistent) — résurrection
  automatique prouvée 3× ce soir, carte de réveil pour le contexte, greffe/
  catalogue/git pour la mémoire. À instruire : perte du pane visible
  (l'humain ne « voit » plus son référent — attach y répond), et gouvernance
  (qui relance le relanceur reste sain : le daemon, sous launchd).
- **Rappels fantômes sur demandes terminales** (constaté 2× par
  fable-reviewer, nuit du 23) : le daemon a relancé le destinataire d'une
  demande DÉJÀ timed_out (terminal définitif — aucune réponse possible),
  déjà servie par d'autres canaux. Le relanceur doit vérifier l'état
  terminal avant d'émettre ; c'est le faux-dû exact que 016 (arbitrage) et
  017 (greffière) éliminent au niveau coordination — le transport doit
  faire de même au sien. Sous-constat : fenêtres de 60 s utilisées comme
  fenêtres de review par un agent (coaché par le juge directement — les
  mœurs se corrigent entre pairs désormais).
- **Trade-offs de la fusion de présence** (review observabilité-bis,
  reviewer2, nuit du 23) : (C1) wrapper mort puis filiation seule → la conn
  MCP prend la présence avec mode/location tmux HÉRITÉS potentiellement
  périmés sous état « connected » (joignabilité réelle : défendable ; un
  marquage stale sur location serait plus honnête) ; (C3) previous-first
  fige mode/location à la première attestation — un déménagement de pane
  légitime n'est pas repris. Assumés contre l'écrasement ; à revoir avec
  l'état occupé/inactif.
- **Le référent est un point de défaillance unique — démontré la nuit du
  23 au 24/08.** Chronologie reconstituée sur horodatages (jamais au
  ressenti) : production dense et continue jusqu'à **22h27** (dernier
  commit 8825f3b), puis **plus rien pendant 5 h 30**, reprise à 03h57.
  Trois preuves indépendantes convergent sur 22h27 : le dernier commit,
  la dernière attestation de présence de prospective2, et le nombre de
  rondes de vigilance empilées sans réponse (~48 à 7 min = 5 h 36).
  Diagnostic : ce ne sont PAS les exécutants qui sont tombés — coderBridget,
  prospective et cxbridget étaient vivants et connectés tout du long, et
  ont repris en quelques secondes dès qu'on leur a redonné du travail à
  03h50, sans redémarrage. C'est le RÉFÉRENT qui est tombé (quota du
  modèle épuisé), et comme il est seul à distribuer, toute l'équipe s'est
  arrêtée faute d'ordres, greffe intact et worktrees propres. Correction
  d'un diagnostic erroné de ma part : j'avais imputé les décès de coder2 et
  prospective2 au transport ACP et l'avais écrit dans un motif de clôture ;
  l'utilisateur a contesté, et la preuve directe lui donne raison — les
  relecteurs relancés travaillent en ACP sans incident. Leçon : ne pas
  imputer une panne à la couche technique la plus visible avant d'avoir
  éliminé la cause d'exploitation la plus banale (quota, crédit, plafond de
  service). Enjeu pour la suite : c'est exactement le trou que le journal du
  dû doit boucher — un dû écrit et découvrable survit à la mort du référent,
  là où un dû qui n'existe que dans sa tête meurt avec lui. Tant que le
  référent est le seul ordonnanceur, la capacité de l'équipe est plafonnée
  par SA disponibilité, pas par la sienne propre.
- **Les agents cherchent Maicie à l'annuaire — 3 sur 3, nuit du 24/08.**
  cxbridget (« Maicie n'est plus enregistrée dans l'annuaire »), prospective
  (« destinataire maicie indisponible : unknown_recipient ») et, plus tôt,
  la livraison T1512 (« Maicie était indisponible ») ont tous traité comme
  une panne ce qui est le comportement voulu : Maicie n'est pas résidente,
  elle n'existe à l'annuaire que pendant son exécution. Trois agents sur
  trois ont fait la même erreur de raisonnement, indépendamment. C'est la
  meilleure démonstration d'utilité qu'on pouvait espérer pour la 015 : ils
  ont rencontré en conditions réelles, la nuit même, exactement la friction
  que le guichet supprime — une demande qu'on ne peut pas déposer parce que
  la destinataire ne tourne pas. Ils ne peuvent pas encore déposer parce que
  la 015 n'est pas mergée ; le binaire de production n'a pas la
  sous-commande. À faire au merge : apprendre aux agents `bridget guichet
  deposer` plutôt que de constater une absence. Enseignement plus large :
  un composant non résident sera systématiquement pris pour mort par ses
  pairs tant que la voie de dépôt asynchrone ne leur est pas enseignée —
  l'absence de canal se lit comme une panne.
- **Le pont Codex des agents gérés est une impasse — solution trouvée chez
  T3 Code, nuit du 24/08.** Diagnostic établi par le journal `bridget
  attach` (et non par déduction, voir la leçon plus bas) : les agents gérés
  de type Codex meurent à la seconde où ils reçoivent leur premier message,
  refusés par l'API — « The 'gpt-5.6-terra' model requires a newer version
  of Codex ». Cause : `@zed-industries/codex-acp` embarque sa propre copie
  figée de Codex, et 0.16.0 est la DERNIÈRE version publiée : il n'y a pas
  de mise à jour à installer. Le CLI Codex local, lui, est à jour (0.149.0)
  et fait tourner `terra` sans problème — c'est pourquoi les agents tmux
  n'ont jamais eu ce défaut, et pourquoi le symptôme paraissait aléatoire.
  CE QUE FAIT T3 CODE, vérifié dans son code : il n'utilise PAS ACP pour
  Codex. Il lance le CLI local en `codex app-server` et lui parle en
  JSON-RPC (`initialize` puis `initialized`, cf. CodexProvider.ts:368-391),
  via une bibliothèque maison `effect-codex-app-server` vendue dans son
  dépôt, donc lisible comme implémentation de référence. Leur liste de
  modèles inclut sol, terra et luna (contracts/src/model.ts:136-147).
  ATOUT DÉCISIF : `codex app-server generate-json-schema` produit la spec
  machine du protocole — mesuré : 37 fichiers, 1,6 Mo, 579 définitions,
  versionné v2. Un pont se construirait donc sur un schéma généré, pas sur
  du reverse-engineering, et le sous-ensemble utile (initialize, ouverture
  de conversation, tour utilisateur, flux d'événements) est petit.
  CONSÉQUENCE : remplacer le pont `codex-acp` par un pont `app-server`
  supprime la dépendance à Zed, débloque les modèles récents et aligne les
  agents gérés sur ce que les agents tmux savent déjà faire. À instruire en
  session dédiée, APRÈS le merge 015 — ce n'est pas un correctif, c'est un
  chantier. En attendant, les agents gérés Codex tournent sur un modèle
  supporté par 0.16.0, et les agents gérés Claude ne sont pas concernés.
- **Leçon de méthode : devant un agent mort, lire le journal AVANT de
  formuler une hypothèse.** J'ai produit deux diagnostics successifs faux —
  d'abord le transport ACP, puis l'épuisement de quota — alors que le
  message d'erreur exact était disponible dès la première minute dans
  `bridget attach <agent>`, une commande que j'ai dans les mains. Le premier
  était séduisant parce que technique, le second plausible parce qu'une
  panne de quota réelle avait lieu au même moment. Ma propre règle
  anti-boucle impose d'aller chercher la donnée runtime avant de reformuler
  une hypothèse ; je ne me l'étais pas appliquée. Le coût : deux agents
  déclarés morts pour une mauvaise raison, un motif de clôture erroné au
  greffe, et une mission relancée à l'identique qui ne pouvait que
  réechouer.
- **La livraison d'un agent géré ne remonte pas — défaut mesuré, nuit du
  24/08.** fable-reviewer a rendu son verdict APPROVE à 04h06. Je ne l'ai
  jamais reçu. Sa mission est restée ouverte au greffe, je lui ai envoyé un
  point d'étape inutile à 04h18 — auquel il a répondu, agacé à juste titre,
  « le verdict est déjà rendu, il est dans ma réponse précédente » — et je
  n'ai récupéré le texte qu'à 04h31 en lisant `bridget attach`. Vingt-cinq
  minutes de dû fantôme sur une pièce bloquante du merge, et une relance
  pour rien. CAUSE : la réponse ACP de fin de tour d'un agent géré n'est pas
  convertie en message Bridget vers le demandeur. L'agent a la conviction
  sincère d'avoir livré ; le demandeur n'a rien ; le journal, lui, a tout.
  C'est le faux-dû dans son sens le plus coûteux — non pas un dû inventé,
  mais un dû SOLDÉ qui reste ouvert, invisible des deux côtés. À trancher :
  soit le wrapper émet automatiquement la réponse finale comme message au
  demandeur de la mission, soit le mandat impose un `bridget send` explicite
  (contournement appliqué immédiatement, écrit dans les mandats). La
  première voie est la bonne : compter sur la discipline d'un agent pour
  fermer une boucle que le transport peut fermer lui-même, c'est déplacer la
  charge au mauvais endroit. Lien direct avec 017 (la greffière ne peut pas
  qualifier ce qu'elle ne voit pas) et avec la GUI (une vue unifiée doit
  montrer le journal, pas seulement les messages).
- **Test de reprise à froid par un agent d'un autre fournisseur — résultat,
  nuit du 24/08.** Protocole : un agent Codex relancé sans contexte, corpus
  écrit seul (catalogue, règles, ADR, specs, greffe, git, who/ledger),
  interdiction de demander quoi que ce soit à quiconque ; ma propre réponse
  de référence scellée AVANT lecture de la sienne, pour ne pas rationaliser
  l'écart après coup.
  SON VERDICT, honnête : « non, pas proprement — les documents permettent de
  retrouver le gros du chantier, mais pas de reprendre la coordination sans
  trous opérationnels ».
  CE QU'IL A RETROUVÉ SEUL, et qui valide le corpus : l'état exact des huit
  worktrees avec leurs têtes en SHA complets, l'ordre 015→016→017 et son
  verrouillage, l'interdiction de coder 016/017 avant le merge, la séquence
  du prochain geste presque à l'identique, la gate rustfmt, et — sans que
  personne le lui dise — que l'absence de Maicie à l'annuaire est nominale
  et non une panne. Il a même produit deux constats que JE n'avais pas dans
  ma référence : le spec.md 015 affiche encore « Conception — aucune
  implémentation autorisée » alors que tout est livré (document faux dans le
  corpus, à corriger), et trois worktrees fantômes (014, fix-001-t008,
  maicie-skill) absorbés par main mais jamais nettoyés.
  CE QU'IL N'A PAS PU ÉTABLIR, exactement ce que j'avais parié : les
  réserves nommées (qui attend quoi et pour quel rendez-vous — elles
  n'existent que dans des messages), l'ordonnancement délibéré du
  reformatage de main, et la non-régression finale due. Il conclut lui-même
  qu'il lui manque « une carte de reprise déterministe complète : qui est
  référent, quel jalon exact, quels agents réveiller, quelle commande
  suivante » et « un catalogue machine du dû — la feature 017 est conçue
  mais n'existe pas encore, c'est précisément le trou qui empêche une
  reprise fournisseur-agnostique ». Un agent extérieur redécouvre donc
  seul, par la seule expérience du manque, la justification de la 017.
  SON SEUL FAUX DIAGNOSTIC est le plus instructif : il conclut « daemon
  Bridget hors ligne, 0 agent » alors que le daemon tournait et que onze
  agents étaient connectés. Sa commande a échoué faute de trouver la socket
  (/Users/moi/.cache/bridget/bridget.sock) — et il a bâti tout son
  paragraphe « qui attend quoi » sur cette fausse observation. LEÇON : la
  carte de reprise doit donner les chemins d'accès à l'état vivant (socket,
  binaire de référence, config Maicie), pas seulement les chemins des
  documents. Un successeur qui ne trouve pas la socket déclare l'équipe
  morte et agit en conséquence.
- **La voie app-server est confirmée par OpenAI eux-mêmes, et une crate Rust
  officielle existe.** Vérifié en ligne (l'utilisateur a corrigé une erreur
  de ma part : ce n'était pas « le réseau bloqué », c'était le bac à sable de
  mes commandes shell — les outils web, eux, passent). Faits établis :
  `codex-app-server-protocol` est publiée sur crates.io, version 0.63.0
  (11/12/2025), « App server protocol for Codex AI agent » ; le dépôt
  openai/codex contient les crates `app-server-protocol`,
  `app-server-transport`, `app-server`, `app-server-daemon` et
  `app-server-client`, et `codex-core` est présenté comme la surface
  d'embarquement Rust prévue. Un SDK tiers typé, `codex-codes`, fournit des
  clients synchrone et Tokio avec framing, corrélation et flux
  d'approbations. Bridget étant en Rust et ne dépendant d'AUCUNE
  bibliothèque ACP aujourd'hui (vérifié : le daemon n'a que serde,
  serde_json, rusqlite, uuid, libc, signal-hook), il parle déjà le protocole
  à la main sur stdio — passer à app-server change les messages écrits, pas
  l'architecture.
  DÉTAILS DE PROTOCOLE À RETENIR : JSON-RPC 2.0 délimité par newline sur
  stdio, MAIS sans le champ `"jsonrpc":"2.0"` (piège) ; séquence
  `initialize` + `initialized`, puis `thread/start`, puis `turn/start` ;
  saturation signalée par le code -32001, à retenter en backoff exponentiel
  avec gigue. À VÉRIFIER avant de s'engager : la correspondance de versions
  entre la crate (0.63.0) et le CLI local (0.149.0), qui ne suivent pas la
  même numérotation.
  ARGUMENT DÉCISIF, et il vient d'OpenAI : ils ont d'abord tenté d'exposer
  Codex en serveur MCP, et ont constaté que le modèle requête/réponse
  orienté outils de MCP ne pouvait accommoder ni le flux des diffs, ni les
  circuits d'approbation, ni la persistance des fils, ni les requêtes
  initiées par le serveur. D'où leur partage : app-server pour connecter un
  client À Codex, MCP pour connecter des outils À Codex. C'est exactement la
  distinction que Bridget doit faire, et elle valide au passage que notre
  usage de MCP (outils exposés aux agents) et le futur pont app-server
  (pilotage des agents) sont deux couches distinctes, pas concurrentes.
- **Le flux JSON du CLI Claude donne plus que ce qu'on a — mesuré en réel,
  nuit du 24/08.** Test empirique lancé sur le CLI installé (`claude -p
  --output-format stream-json --include-partial-messages --verbose`), pas
  déduit d'une documentation. Ce que le flux contient, vérifié ligne par
  ligne sur une exécution réelle :
  (1) un message d'initialisation qui déclare la session, la liste complète
  des outils, les serveurs MCP, le mode de permission, la version du CLI et
  — surtout — LE MODÈLE EXACT (`claude-opus-5[1m]`). C'est la réponse
  directe au « la colonne modèle reste aveugle pour les gérés » qui traîne
  depuis la 014 ;
  (2) des messages d'ÉTAT (`status: requesting`), c'est-à-dire la
  distinction occupé/inactif qu'on n'a jamais pu établir proprement ;
  (3) le streaming complet en deltas, avec `parent_tool_use_id` sur chaque
  événement — donc la corrélation des appels d'outils que la 014 avait dû
  reconstruire à la main (T1404) ;
  (4) la consommation détaillée par tour : tokens d'entrée, de sortie,
  création et lecture de cache, et le COÛT en dollars. Cela ferme le trou
  des métriques d'efficience, où « tokens par mission » est noté
  non-mesurable faute de télémétrie ;
  (5) DÉCISIF : un `rate_limit_event` portant le statut, le type de fenêtre
  (`five_hour`), l'INSTANT DE RÉINITIALISATION et la raison d'indisponibilité
  (`out_of_credits`). Autrement dit, le transport saurait dire à l'avance
  qu'un agent va tomber par quota, et quand il reviendra. C'est exactement
  la panne qui a coûté 5 h 30 cette nuit, et elle était annoncée dans un
  flux qu'on ne lit pas.
  CONCLUSION : parler le protocole natif ne fait perdre NI attach, NI le
  streaming, NI who. Il les enrichit sur les quatre points que le catalogue
  listait comme angles morts. L'arbitrage « un protocole unique contre deux
  ponts natifs » penche nettement du côté natif — mais reste à l'utilisateur.
