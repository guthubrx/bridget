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
- **Côté Codex, le protocole natif va PLUS loin que celui de Claude —
  inventaire fait sur le schéma généré.** Là où le flux Claude ÉMET des
  événements qu'on subit, l'app-server permet aussi d'INTERROGER. Relevé sur
  les 579 définitions : 17 types de limites, dont
  `AccountRateLimitsUpdatedNotification` (avertissement proactif quand les
  limites bougent), `GetAccountRateLimits` (on peut demander l'état AVANT de
  confier une mission : cet agent a-t-il de quoi finir ?), `CreditsSnapshot`
  et `RateLimitResetCredit` ; 11 types de consommation, dont
  `ThreadTokenUsageUpdatedNotification` — la consommation PAR FIL mise à
  jour en continu, c'est-à-dire le coût par mission que les métriques
  d'efficience déclarent non-mesurable ; 15 types autour du modèle, dont
  `ModelListResponse` (la liste des modèles réellement disponibles — la
  panne `terra` de cette nuit aurait été détectée AVANT de lancer l'agent,
  pas après sa mort) et surtout `ModelReroutedNotification`, qui signale
  qu'un modèle a été remplacé par un autre. Ce dernier point répond
  exactement au cas fable-reviewer, qui tournait en Opus 4.6 alors qu'on lui
  demandait Opus 5, sans que rien ne le signale : la dégradation silencieuse
  deviendrait détectable.
  CE QUE ÇA CHANGE POUR LA CONDUITE D'ÉQUIPE, au-delà de l'observabilité :
  on passe d'un référent qui CONSTATE les morts à un référent qui les
  ANTICIPE — vérifier le quota avant d'affecter, connaître l'heure de
  retour, voir la dégradation de modèle, mesurer le coût de chaque mission.
  C'est un changement de nature, pas de confort.

## Bloc G — Sortir du pont Zed vers les protocoles natifs

Ajouté au plan le 2026-08-24. Ordre recommandé : G1 à G4 forment le socle
Codex (le cas urgent, celui qui a tué des agents) ; G5-G6 sont le socle
Claude ; G7-G9 sont les gains qui justifient l'opération ; G10 est la sortie.
Chaque étape doit laisser le projet buildable et l'équipe opérationnelle :
aucune ne coupe le pont existant avant que son remplaçant soit prouvé.

- **G1 — Décider du vocabulaire interne.** Aujourd'hui le daemon écrit
  directement les messages ACP. Deux protocoles natifs vont s'ajouter, aux
  noms différents (`thread/start` + `turn/start` côté Codex, messages typés
  côté Claude). Trancher : soit une couche interne neutre avec un traducteur
  par fournisseur, soit du code par fournisseur assumé. ADR obligatoire —
  c'est LA décision structurante, tout le reste en découle.
- **G2 — Pont Codex app-server, minimal et prouvé.** Lancer `codex
  app-server` au lieu de `npx @zed-industries/codex-acp`, faire la séquence
  `initialize`/`initialized`, ouvrir un fil, envoyer un tour, lire le flux.
  Pièges déjà connus : JSON-RPC délimité par newline SANS le champ
  `"jsonrpc":"2.0"` ; saturation signalée par le code -32001, à retenter en
  backoff exponentiel avec gigue. Preuve exigée : un agent Codex géré,
  spawné, qui reçoit une mission et répond — avec `gpt-5.6-terra`, le modèle
  qui échoue aujourd'hui.
- **G3 — Types du protocole.** Évaluer `codex-app-server-protocol` (crate
  officielle, 0.63.0) contre une génération depuis le schéma
  (`generate-json-schema`, 579 définitions). Vérifier la correspondance de
  versions avec le CLI local (0.149.0) — numérotations différentes, à ne pas
  supposer alignées.
- **G4 — Parité d'observabilité Codex.** `who` et `attach` doivent rendre au
  moins autant qu'aujourd'hui : mode, localisation, appels d'outils titrés,
  journal. Gate de non-régression : rejouer les scénarios de la 014 sur le
  nouveau pont, comparer les sorties.
- **G5 — Pont Claude en flux natif.** `claude -p --input-format stream-json
  --output-format stream-json --include-partial-messages`. Vérifié en réel :
  le flux porte le modèle exact, l'état d'occupation, les deltas, la
  corrélation d'outils, la consommation et les limites.
- **G6 — Parité d'observabilité Claude**, même gate que G4.
- **G7 — Modèle véridique et dégradation détectée.** Afficher le modèle réel
  dans `who` depuis le flux, et lever une alerte sur reroutage
  (`ModelReroutedNotification` côté Codex). Ferme l'angle mort « colonne
  modèle aveugle » ET le cas Opus 5 demandé / Opus 4.6 servi.
- **G8 — Quota anticipé.** Lire les limites et l'instant de réinitialisation,
  les exposer dans `who`, et VÉRIFIER AVANT D'AFFECTER une mission longue.
  Côté Codex on peut aussi interroger activement. C'est la panne du
  2026-08-23 22h27 rendue prévisible.
- **G9 — Coût par mission.** Brancher la consommation par fil sur le greffe :
  chaque objectif clos porte ce qu'il a coûté. Ferme le trou « tokens par
  mission : non mesurable » des métriques d'efficience.
- **G10 — Retirer le pont Zed pour Codex et Claude.** Seulement une fois G2
  et G5 prouvés en production. ARBITRÉ PAR L'UTILISATEUR le 2026-08-24 : le
  pont ACP est CONSERVÉ pour Gemini seul, et inscrit comme OBSOLÈTE. Ce
  n'est pas une exception au principe « zéro code mort » : le pont sert
  encore pour un fournisseur réel, mais il ne reçoit plus d'investissement
  et sa disparition est décidée par avance — le jour où Gemini expose un
  accès natif, ou le jour où Gemini sort du périmètre. À inscrire dans
  DEPRECATIONS.md au moment de G10, avec sa condition de retrait. Corollaire
  opérationnel : plus aucune nouvelle capacité ne passe par ACP ; toute
  fonction ajoutée aux blocs G7 à G9 (modèle véridique, quota anticipé, coût
  par mission) est développée sur les ponts natifs, et Gemini en est
  simplement privé. Un adaptateur obsolète qu'on maintient à parité coûte
  plus cher que deux ponts vivants.
- **T3 Code valide l'architecture à registre — et garde ACP en repli, comme
  nous.** Inventaire de leurs adaptateurs : Claude (SDK officiel Anthropic),
  Codex (app-server natif), OpenCode (SDK natif `@opencode-ai/sdk`), Cursor,
  et **Grok — qui passe par ACP** (`effect-acp/schema`). Aucun Gemini chez
  eux. Leur règle de fait est donc exactement celle que l'utilisateur a
  arbitrée pour nous : protocole natif quand il existe, ACP en repli pour les
  fournisseurs qui n'en offrent pas. Notre Gemini est leur Grok.
  Ils ont par ailleurs un `ProviderAdapterRegistry` qui associe un type de
  fournisseur à son adaptateur, l'adaptateur étant construit par instance et
  porté par elle. C'est l'option « vocabulaire interne neutre + traducteur
  par fournisseur » du G1 — décidée par un produit réel, pas par supposition.
  Trois composants voisins méritent d'être regardés au moment du G1, parce
  qu'ils nomment des besoins qu'on découvrira sinon en production :
  `ProviderSessionDirectory` (où vivent les sessions),
  `ProviderSessionReaper` (le nettoyage de celles qui traînent — nous avons
  eu 15 processus orphelins cette nuit) et `ProviderEventLoggers`.
- **CLARIFICATION qui change le critère du bloc G : ACP n'est pas le
  problème, le PONT TIERS FIGÉ l'est.** Vérifié sur pièces : chez T3 Code,
  Cursor et Grok passent tous deux par ACP — mais pour Cursor, c'est le CLI
  du fournisseur LUI-MÊME qui expose le mode (`cursor-agent acp`, confirmé
  sur la machine : binaire présent, version 2026.08.11, l'aide annonce
  « Start the Cursor Agent as an ACP server »). Rien à voir avec
  `@zed-industries/codex-acp`, qui est un tiers embarquant une copie figée
  de Codex et qui a tué nos agents. Le bon critère n'est donc pas
  « natif contre ACP » mais « maintenu par le fournisseur contre pont tiers
  figé ». Un ACP publié et maintenu par le fournisseur est aussi sain qu'un
  protocole propriétaire.
- **Cursor est ajoutable presque gratuitement — type déclaré le 2026-08-24.**
  Le CLI est installé, il parle ACP nativement, l'authentification passe par
  le compte (`cursor_login` chez T3 Code) et non par une clé API, ce qui
  respecte la politique du projet. Notre pont ACP existe déjà : c'est le
  seul fournisseur qu'on peut brancher SANS écrire de pont. Type `cursor`
  inscrit dans le registre (commande `cursor-agent acp`, clés API interdites
  en dur). Reste la naissance d'agent, qui appartient à l'utilisateur.
  Conséquence sur G10 : le pont ACP ne sert plus seulement un fournisseur
  obsolète, il sert Cursor — il n'est donc PAS à retirer, seulement à ne
  plus être le chemin par défaut pour Codex et Claude.
- **G11 — Nommer le protocole, pas seulement le canal (remarque de
  l'utilisateur, 2026-08-24).** Aujourd'hui `who` affiche `acp` à la fois en
  TRANSPORT et en MODE, ce qui confond deux choses distinctes : COMMENT
  l'agent est relié (socket locale pour un pane tmux, processus fils piloté
  pour un agent géré) et QUELLE LANGUE il parle. Tant qu'il n'y avait qu'un
  protocole, l'amalgame était sans conséquence. Avec trois — ACP pour Cursor
  et Gemini, app-server pour Codex, flux natif pour Claude — la colonne
  devient mensongère : deux agents affichés `acp` ne parleront plus la même
  langue. À faire pendant G4 et G6, pas après : distinguer le canal du
  protocole dans le modèle de présence ET dans l'affichage, et rendre le
  protocole visible (`json/acp`, `json/codex`, `json/claude`). Corollaire de
  diagnostic, appris cette nuit : quand un agent meurt, savoir quel protocole
  il parlait est la PREMIÈRE information utile — c'est le pont figé qui a tué
  les agents Codex, pas le canal.
- **Observation connexe, mesurée le 2026-08-24 : seuls les agents gérés
  savent dire qu'ils travaillent.** `state: busy` remonte pour les agents
  connectés en agent géré (Cursor observé en `busy` pendant sa mission ; les
  relecteurs Claude aussi), jamais pour les panes tmux — qui affichent
  `connected` même en plein travail. C'est la cause racine de l'angle mort
  occupé/inactif reproché par l'utilisateur, et la raison d'être des rondes
  de vigilance à sept minutes : faute de pouvoir distinguer, il faut
  demander. Conséquence pour le plan : les ponts natifs porteront cet état
  (vérifié dans le flux Claude), donc migrer les agents interactifs vers le
  mode géré supprimerait le besoin de ronde. À instruire après G4/G6 — c'est
  un changement d'ergonomie pour l'utilisateur, qui perd le pane visible.

## Bloc J — Rattraper T3 Code sur l'outillage d'agents

Ajouté le 2026-08-24, sur demande de l'utilisateur, après comparaison
détaillée de notre architecture de ponts avec la leur. Constat : **notre
socle est au moins aussi bon, notre outillage autour l'est moins.** Nous
avons deux avantages qu'ils n'ont pas — un daemon persistant qui tient
l'état en base, et le greffe qui journalise des décisions avec leurs
motifs, dont ils n'ont aucun équivalent. Les manques ci-dessous sont ceux
que la comparaison a révélés, et deux d'entre eux nous ont coûté la nuit.

- **J1 — Capacités déclarées par pilote ET par modèle.** Ils associent à
  chaque fournisseur, et à chaque modèle, ce qu'il sait faire : changer de
  modèle en cours de session, reprendre, être interrompu. Nous, nous
  supposons. C'est exactement ce qui nous a fait lancer deux agents sur un
  modèle que le pont ne savait pas servir, et découvrir la panne sur leur
  cadavre. Une capacité déclarée aurait refusé le spawn avec un motif clair.
  À faire pendant les blocs G : un pilote annonce ce qu'il sait faire, et le
  daemon refuse ce qu'il ne sait pas, au lieu de le tenter.
- **J2 — Ramasseur de sessions abandonnées.** Ils en ont un ; nous n'avons
  rien, et la nuit du 24 a révélé 15 wrappers orphelins, 7 daemons de test
  et une cinquantaine de répertoires temporaires. Le correctif en cours
  traite la CAUSE (gardes à la destruction dans les harnais) ; il ne traite
  pas les survivants ni les fuites futures d'une autre origine.
  LEÇON À REPRENDRE TELLE QUELLE, lue dans leur code : leur ramasseur refuse
  de tuer une session qui a un tour actif, et refuse aussi celle qui a du
  travail de fond en cours — commentaire d'origine : « ce sont des processus
  du fournisseur, les arrêter les tuerait silencieusement ». Ils ont donc
  appris qu'un nettoyage naïf casse plus qu'il ne répare. Notre ramasseur
  devra porter les mêmes gardes dès le premier jour, sans avoir à commettre
  l'erreur nous-mêmes.
  ✅ **Phase 0 démarrée (2026-08-24)** — arbitrage (a)–(e) approuvé ; spec
  `j2-session-reaper-phase0.md` ; commande `bridget reaper report` ;
  observateur seul ; fail-closed sur l'absence de signal de travail de fond
  (manque à instruire). Phase 1 = arbitrage explicite après lecture multi-jours.
- **J3 — Répertoire des sessions.** Ils tiennent une table des sessions
  ouvertes avec leur fournisseur, leur fil et leur curseur de reprise. Chez
  nous cette information est éparpillée entre la base du daemon, les
  définitions figées et les marqueurs d'agents gérés. À instruire pendant
  G1 : est-ce que notre base couvre déjà le besoin, ou manque-t-il une vue ?
  Ne pas créer une table par mimétisme — vérifier d'abord.
- **J4 — Journaux d'événements par pilote.** Ils séparent les journaux par
  fournisseur, ce qui rend un incident attribuable sans fouiller. Nous avons
  le journal d'attache, qui est meilleur pour suivre un agent, mais rien
  pour comparer un pilote à un autre. Utile seulement quand deux ponts
  tourneront — à ne pas faire avant.

Ordre recommandé : J1 pendant G2 (le refus informé est ce qui aurait évité
la panne), J2 dès que le correctif de cause est mergé, J3 en instruction
dans G1, J4 après le second pont. Aucun de ces items ne justifie de retarder
les blocs G : ce sont des compléments, pas des prérequis.
- **La suite de tests est devenue un goulot d'étranglement — constat
  systémique, nuit du 24/08.** QUATRE agents différents ont perdu du temps
  sur la même chose en quelques heures : `cargo test --workspace` prend
  environ trois minutes, contient des oracles instables, et chaque agent qui
  veut valider son lot doit la traverser. Trois l'ont vue interrompue après
  quinze à vingt-cinq minutes ; un quatrième a été bloqué en fin de lot par
  un test qui n'appartient pas à son périmètre. Le coût cumulé dépasse
  largement celui des défauts qu'elle a détectés cette nuit.
  DEUX EFFETS, et le second est le pire : (1) du temps perdu à attendre ;
  (2) une incitation à requalifier un rouge en « flake » pour avancer. Deux
  agents ont explicitement refusé de le faire cette nuit — et ils ont eu
  raison les deux fois, il y avait un vrai défaut derrière. Mais compter sur
  la vertu de chacun face à une friction structurelle est une mauvaise
  politique : la friction finira par gagner.
  DÉCISION DE CONDUITE, appliquée immédiatement : un agent bloqué par un
  test hors de son périmètre LIVRE avec une validation ciblée, en annonçant
  explicitement ce qui n'a pas pu être joué et pourquoi. Une preuve
  partielle déclarée comme telle vaut mieux qu'une attente d'une demi-heure,
  et infiniment mieux qu'une preuve verte affirmée sans avoir été obtenue.
  Le référent rejoue la suite au moment du merge, quand les correctifs
  d'instabilité sont en place.
  À INSTRUIRE quand les niveaux 1 et 2 de l'audit seront soldés : séparer
  les tests rapides et déterministes, jouables par chaque agent en quelques
  secondes, des campagnes lentes ou mesurées, jouées au merge. Aujourd'hui
  tout est mélangé, donc tout coûte le prix du plus lent.
- **ARBITRAGE : une sévérité DÉRIVÉE n'est pas une sévérité JUGÉE.** Question
  remontée le 2026-08-24 par l'agent de la session 017, qui a refusé de
  trancher seul : pour que le dû s'inscrive tout seul, il faudrait qu'un
  gate échoué ou une revue qui amende laissent une trace — mais toute trace
  porte une sévérité, et l'inventer reviendrait à laisser Maicie juger, ce
  que le cadre-loi interdit. Il s'est arrêté et m'a passé la question. Bonne
  conduite : c'est un arbitrage de frontière, pas un détail d'implémentation.
  DÉCISION. La sévérité peut être DÉRIVÉE du TYPE DE FAIT, à trois
  conditions cumulatives : (1) la correspondance est écrite À L'AVANCE dans
  la conception, pas décidée par le code au moment de l'écriture ; (2) elle
  est TOTALE et sans exception sur les types couverts — pas de « selon le
  contexte », qui serait du jugement déguisé ; (3) tout fait qui ne tombe
  dans AUCUNE case part en attente de qualification, donc chez un humain.
  Le défaut est l'attente, jamais l'invention.
  MOTIF. Traduire « ce gate a échoué » en « bloquant » n'est pas une
  évaluation : c'est la définition d'un gate. Traduire un verdict AMENDER en
  « majeur » n'est pas une opinion : c'est ce que le relecteur a déjà décidé
  en écrivant AMENDER. Dans les deux cas la décision est prise EN AMONT par
  un humain ou par un fait vérifiable ; Maicie ne fait que la transcrire.
  Ce qui reste interdit : classer par importance, calculer une priorité,
  décider qu'un constat mérite attention plus qu'un autre. La frontière est
  entre TRANSCRIRE une décision prise ailleurs et FORMER un avis.
  CONSÉQUENCE PRATIQUE : la table de correspondance doit être versionnée
  dans la spec 017 et relue comme un contrat. Si un jour on hésite sur une
  case, c'est le signe qu'elle n'appartient pas à la table — elle part en
  attente.
- **Redémarrer le daemon TUE les sessions des agents gérés — leçon payée le
  2026-08-24 à 06h37.** J'ai redémarré pour déployer le correctif qui me
  rendait aveugle sur cinq agents. Les agents en panes tmux ont survécu sans
  broncher ; les SEPT agents gérés ont perdu leur session et sont revenus
  vides. Leur travail n'était pas perdu — il était sur disque, non commité —
  mais eux ne savaient plus ce qu'ils faisaient. Il a fallu leur réécrire à
  chacun où en était leur propre travail, fichier par fichier.
  CE QUE ÇA COÛTE, mesuré : deux agents à relancer avec un mandat de reprise
  détaillé, une bissection en cours perdue, et le contexte de trois autres
  missions envolé. La résurrection automatique des agents persistants
  fonctionne parfaitement — mais elle ressuscite le processus, pas la
  mémoire.
  RÈGLE POSÉE, applicable immédiatement : avant tout redémarrage du daemon,
  demander aux agents gérés de COMMITER leur travail en cours, pas seulement
  de retenir leur livraison. Retenir un message ne protège que le message ;
  seul un commit protège le travail. Les agents tmux n'ont pas besoin de
  cette précaution — ils survivent.
  À INSTRUIRE : le mandat de reprise pourrait être automatique. Un agent
  géré qui ressuscite reçoit aujourd'hui une session vide ; il pourrait
  recevoir l'état de son worktree — branche, diff non commité, dernière
  mission au greffe. C'est exactement la carte de reprise, mais à l'échelle
  d'un agent au lieu du référent. Même problème, même remède.
- **Les missions longues tuent les agents gérés — timeout de transport,
  constaté le 2026-08-24 à 06h21.** Un agent Cursor est mort en plein
  travail sur « timeout ACP pour session/prompt ». Sa mission exigeait vingt
  exécutions d'un test en isolation PUIS une campagne sous charge : un tour
  de plusieurs dizaines de minutes. Le transport a expiré avant la fin.
  Établi par lecture du journal, pas par déduction — et il était mort SEIZE
  MINUTES avant le redémarrage du daemon, qui n'est donc pas en cause.
  RÈGLE POUR LES MANDATS : ne jamais demander une campagne longue dans le
  même tour qu'une analyse. Découper — mesurer d'abord, conclure ensuite —
  ou borner explicitement le nombre d'exécutions. Une campagne de vingt
  itérations qui confirme ce que cinq suffisent à démontrer coûte la vie de
  l'agent qui la mène. J'ai d'ailleurs raccourci sa consigne en cours de
  route pour cette raison ; c'était trop tard.
  À INSTRUIRE : le timeout est-il configurable par mission ? Un mandat qui
  annonce sa durée attendue pourrait obtenir une fenêtre adaptée, au lieu de
  mourir silencieusement contre une valeur fixe.
- **Deux branches peuvent fusionner SANS conflit et produire un système
  cassé — démontré le 2026-08-24 sur le gate fondateur.** Le gate G1504
  était rouge sur `main` depuis le merge de la 015. J'ai cru à une
  régression et lancé une bissection ; l'agent a démontré que les DEUX
  bornes étaient rouges et que la bissection était donc impossible — au lieu
  de produire un coupable plausible. Bonne conduite : dire qu'on ne peut pas
  conclure vaut mieux qu'une réponse fausse.
  LA VRAIE CAUSE : côté `main`, un commit antérieur avait renommé un
  libellé du CLI ; côté 015, l'oracle du gate cherche l'ANCIEN libellé par
  `grep`. Fichiers différents, donc **aucun conflit textuel** : Git a
  fusionné proprement deux moitiés incompatibles. Le contrat n'a pas changé,
  le code est correct des deux côtés — c'est leur RENCONTRE qui est fausse.
  MON ERREUR, à retenir : ma borne « connue bonne » était une illusion
  d'optique. Le gate ne passait que parce que je pointais le binaire du
  worktree 015, qui imprimait encore l'ancien libellé. Reconstruit depuis ses
  sources, ce même commit est rouge. **Un gate doit toujours être joué avec
  le binaire de la branche testée**, jamais avec celui d'un worktree voisin.
  DEUX LEÇONS DE FOND. (1) Un oracle qui reconnaît un LIBELLÉ HUMAIN par
  `grep` est fragile par construction : il casse à chaque reformulation de
  message, sans qu'aucun comportement ne change. Préférer un signal stable —
  code de sortie distinct, sortie structurée, état en base. (2) Le vrai
  risque du travail en couloirs parallèles n'est pas le conflit d'édition,
  que Git signale, mais le **désaccord sémantique**, qu'il ne voit pas. À
  instruire : rejouer les gates des sessions voisines AVANT de merger, pas
  seulement les siens.
  PIÈGE ÉVITÉ DE JUSTESSE : le remède naïf aurait été de faire renvoyer
  `Accepted` au premier envoi pour que le test passe. Cela aurait cassé le
  contrat d'idempotence — accusé après transmission, réponse au retry — pour
  satisfaire un oracle faux. L'agent l'a signalé de lui-même comme un piège
  pour l'auteur du correctif. C'est exactement le genre de correction qui
  fait disparaître un rouge et apparaître un bug.

## Bloc K — Déployer Bridget et Maicie ailleurs

Ajouté le 2026-08-24 sur demande de l'utilisateur, non urgent mais à prévoir.
Question posée : peut-on installer l'ensemble sur une autre machine, pour un
autre projet, et retrouver les mécanismes dont on se sert depuis des heures ?
**Réponse honnête : non.** Le dépôt contient le code et la doctrine ; tout ce
qui fait fonctionner l'ensemble au quotidien vit HORS du dépôt et a été créé
à la main, geste par geste, pendant des semaines.

Inventaire de ce qui manquerait sur une machine neuve, établi en regardant
ce qui existe ici :

- `~/.config/bridget/agents.json` — le registre des types d'agents, en 0600,
  avec les commandes, protocoles et variables interdites. Sans lui, aucun
  agent ne peut être lancé.
- `~/.config/maicie/config.json` — les profils, sans lesquels aucun agent
  n'est missionnable. Les DEUX inscriptions sont nécessaires, et l'oubli de
  la seconde ne se voit qu'au premier échec de délégation.
- `~/Library/LaunchAgents/com.bridget.daemon.plist` — le daemon lui-même, en
  service géré.
- `~/Library/LaunchAgents/com.bridget.maicie.releve.plist` — le battement de
  cœur de Maicie, créé le 24/08 : sans lui, elle ne relève rien en l'absence
  du référent.
- `~/.local/bin/bridget` — le lien vers le binaire, ce que les agents
  appellent réellement.
- `~/.local/bin/maicie-suivi` — la vue horodatée du greffe.
- La ronde de vigilance — aujourd'hui une tâche planifiée dans la session du
  référent, donc ni portable ni versionnée. C'est le mécanisme le plus utilisé
  de tous et le moins reproductible.
- Le chemin du journal du dû, déclaré en configuration.

- **K1 — Écrire l'installateur.** Une commande qui crée les deux
  configurations avec un contenu minimal viable, installe les deux services,
  pose les liens, et VÉRIFIE son travail — daemon joignable, un agent de test
  lancé puis arrêté, un dépôt au guichet relevé. Un installateur qui ne
  vérifie pas est un générateur de fausse confiance.
  **État 2026-08-24** : `scripts/install-k1.sh` + `make install-k1`.
  Cibles : Darwin (launchd) et Linux (systemd --user) ; Windows refusé
  proprement. Voie Rust : rustup + compilation sur place (autoportant).
  Preuves Mac — idempotence ici + pose bac à sable. Preuve Linux cartae.app
  — **attestée** : daemon systemd actif, spawn/stop fixture, dépôt guichet
  `queued` puis relève Maicie sans panne (`--verify-guichet`).
- **K2 — Rendre la ronde portable.** Elle doit vivre dans le dépôt, pas dans
  la session d'un référent. Soit un service au même titre que la relève, soit
  une commande que n'importe quel référent lance. Aujourd'hui, changer de
  machine ou de session la fait disparaître — c'est arrivé le 24/08.
  **Arbitrage 2026-08-24** : hors lot K1 ; la ronde vivra en service ou en
  commande versionnée dans le dépôt, pas dans la session interactive du
  référent.
- **K3 — Séparer ce qui est propre au projet de ce qui est propre à l'outil.**
  Les règles de chantier, les profils d'agents et le catalogue sont
  spécifiques à CE projet ; le daemon, le guichet et la greffière ne le sont
  pas. Sans cette séparation, déployer ailleurs signifie hériter de nos
  spécificités — nos noms d'agents, nos couloirs, notre catalogue.
- **K4 — Documenter le démarrage à froid.** Que fait un référent qui arrive
  sur une installation neuve, sans historique ? La carte de reprise couvre le
  cas « je reprends un projet en cours » ; elle ne couvre pas « je démarre ».

Priorité : après les blocs en cours. Aucune urgence déclarée par
l'utilisateur, mais le coût augmente à chaque geste manuel non consigné —
c'est pour cela que l'inventaire ci-dessus est écrit MAINTENANT, tant que je
me souviens de chaque pièce.
- **Le guichet est inutilisable tant que le référent ne transmet pas les
  identifiants — constat du 2026-08-24, ma faute.** J'ai écrit dans les
  règles de chantier que tout agent doit déposer son rapport au guichet, et
  j'ai ajouté la consigne à mes mandats. Un agent m'a répondu qu'il ne
  pouvait pas : le dépôt exige un identifiant d'objectif et de délégation, et
  il ne les avait pas. Il a refusé de les inventer.
  Il a raison, et le défaut est dans MA pratique : Maicie me rend ces deux
  identifiants à CHAQUE délégation, dans le JSON de retour. Je ne les
  transmets jamais. La consigne était donc inapplicable dès son écriture.
  CORRECTIF : inscrire l'identifiant d'objectif et de délégation dans chaque
  mandat, comme une ligne fixe. Sans eux, la voie de dépôt reste théorique et
  tous les rapports continueront de passer par moi — c'est-à-dire que le
  bénéfice du guichet reste nul en pratique.
  LEÇON PLUS LARGE, la troisième de cette nature cette nuit : une capacité
  livrée n'est pas une capacité utilisée. Le guichet fonctionne depuis six
  heures, il est prouvé par un gate, et il n'a servi à personne — d'abord
  parce que les agents ne savaient pas qu'il existait, ensuite parce que je
  ne leur donne pas de quoi s'en servir. Vérifier qu'une chose marche et
  vérifier qu'elle sert sont deux gestes différents.
- **PANNE : une seule demande au guichet a paralysé Maicie — 2026-08-24,
  07h45.** Symptôme : toute commande `maicie` échouait sur « traitement
  guichet impossible : état métier incompatible avec la greffe ». Plus de
  status, plus de registre, et le service de relève échouait en boucle. Cinq
  minutes d'arrêt complet de la coordinatrice.
  CAUSE ÉTABLIE en lisant la base : un agent avait déposé un rapport de
  livraison — il suivait ma consigne, et il avait trouvé les identifiants
  seul — sur l'objectif T1607, que je venais de clore À LA MAIN quelques
  minutes plus tôt. Le rapport arrive sur un objectif terminal ; le
  traitement refuse ; le refus est FATAL.
  DEUX DÉFAUTS DISTINCTS, à ne pas confondre.
  (1) ROBUSTESSE. Une demande impossible à traiter doit être REJETÉE avec
  son motif, et l'exécution doit continuer. Aujourd'hui elle est fatale :
  **un seul dépôt mal placé paralyse la coordinatrice**, sans malveillance
  ni bug de l'agent. C'est un déni de service par accident, et il deviendra
  quotidien dès que les agents déposeront en nombre. La conception 015
  prévoyait pourtant ce cas — `request_already_terminal` devait être « greffé
  sans réouverture ni perte » — mais le chemin réel produit une erreur
  fatale, pas un rejet propre.
  (2) COLLISION DE PRATIQUES, et celle-là est de moi. Je clos les objectifs
  à la main dès que je reçois le message de l'agent. Un dépôt asynchrone
  arrive donc TOUJOURS après la clôture. Le guichet et ma clôture immédiate
  sont incompatibles en l'état : soit j'attends le dépôt pour clore, soit la
  clôture accepte un rapport tardif. À trancher — mais ne rien trancher
  signifie que chaque agent obéissant à ma consigne provoquera la panne.
  DÉBLOCAGE : sauvegarde de la base, puis passage de la demande en rejeté
  avec motif. Intervention d'exploitation, pas de code.
  LEÇON : j'ai passé la matinée à demander aux agents d'utiliser le guichet.
  Le premier qui l'a fait a mis Maicie à terre. Une capacité livrée, prouvée
  par un gate, et dont le premier usage réel casse le système — c'est la
  différence entre « ça marche » et « ça tient ».

## Bloc L — Ce que le pont natif rend possible, et le sort des panes tmux

Ajouté le 2026-08-24, une fois le pilote Codex natif livré et mergé. Le
périmètre du pilote a été volontairement borné aux cinq opérations de base
pour qu'il soit prouvé avant d'être enrichi. Ce bloc recense ce qui devient
possible et n'est PAS fait, pour ne rien perdre.

Ces gains sont déjà décrits en G7 à G9 ; ce bloc dit ce qui a changé depuis :
ils ne sont plus hypothétiques, le canal qui les porte existe.

- **L1 — Refuser au lieu de tuer.** Le protocole natif expose la liste des
  modèles réellement disponibles. Un lancement sur un modèle non servi doit
  être REFUSÉ avec son motif, pas tenté. C'est exactement la panne du
  2026-08-23 : deux agents morts à la réception de leur premier message,
  deux diagnostics faux, une heure perdue. Prérequis : J1.
- **L2 — Détecter la dégradation silencieuse.** Le protocole signale le
  reroutage d'un modèle vers un autre. Un agent lancé sur Opus 5 qui rend son
  verdict en Opus 4.6 doit être visible — aujourd'hui, on ne l'a su que parce
  qu'on avait exigé qu'il annonce son modèle en tête de réponse.
- **L3 — Anticiper le quota.** Le flux porte les limites, leur type de
  fenêtre et l'INSTANT DE RÉINITIALISATION. Côté Codex on peut même les
  interroger avant d'affecter une mission longue. La panne du 23/08 à 22h27 —
  cinq heures et demie perdues, six agents vivants sans ordres — était
  annoncée dans un flux que nous ne lisions pas.
- **L4 — Mesurer le coût par mission.** La consommation est suivie par fil de
  conversation. Le greffe pourrait porter ce que chaque objectif a coûté ; les
  métriques d'efficience disent aujourd'hui « tokens par mission :
  non mesurable, aucune télémétrie ».
- **L5 — Voir qui travaille.** Le flux natif porte un état d'occupation. Les
  agents gérés savent déjà dire qu'ils travaillent ; les panes tmux, jamais.
  C'est la cause racine des rondes toutes les sept minutes : faute de
  pouvoir distinguer un agent occupé d'un agent inactif, il faut demander.

### Quand remplacer les panes tmux ?

Question de l'utilisateur, 2026-08-24. Réponse du matin : **pas encore, et
voici les conditions.** *(Caduc depuis 12h25 — les quatre conditions ont été
remplies et la bascule est FAITE ; voir le point d'étape en fin de bloc.)*

Ce que les panes apportent et que les agents gérés n'ont pas encore : le
modèle demandé est réellement servi (les gérés Claude rendent Opus 4.6 pour
Opus 5), la session survit à un redémarrage du daemon (les sept agents gérés
ont perdu la leur le 24/08 à 06h37), et l'utilisateur VOIT son agent.

Ce que les gérés apportent et que les panes n'auront jamais : l'état
d'occupation, le journal lisible par `attach`, et la résurrection
automatique.

CONDITIONS CUMULATIVES avant toute bascule :
1. **Le pont natif servi et prouvé sur les deux fournisseurs** — Codex est
   fait, Claude reste à faire (G5). Basculer avec un seul pont natif
   dégraderait la moitié de l'équipe.
2. **La reprise de session après redémarrage** — un agent géré qui ressuscite
   doit retrouver son contexte, pas repartir vide. Aujourd'hui il faut lui
   réécrire son mandat à la main, ce qui a coûté deux réécritures le 24/08.
3. **Le mode de présence stable à la reconnexion** — encore cassé le 24/08 à
   07h30, `attach` refusant quatre agents qui venaient de se reconnecter.
4. **Une vue pour l'utilisateur** — il perd le pane, donc il doit gagner
   autre chose. C'est le lien avec la GUI, et c'est SA contrepartie : ne pas
   la livrer avant de retirer les panes serait lui prendre sans rendre.

ORDRE : L1 puis L3 (ils évitent des pannes déjà survenues), puis G5 (pont
Claude), puis la reprise de session, puis la bascule. Le retrait des panes
est le DERNIER geste, pas le premier.

### Point d'étape — 2026-08-24, 12h40 : LA BASCULE EST FAITE

Les quatre conditions cumulatives ont été remplies dans la matinée, dans
l'ordre prévu, et le retrait des panes a commencé sur ordre utilisateur.

- **Condition 1 (deux ponts natifs)** : FAIT. Câblage des types par défaut
  mergé (0a3872b) après revue croisée en deux passes — BLOCKED d'abord
  (gel de 600 s sur EOF en plein tour, défaut réel vérifié sur pièces par le
  référent), APPROVE après correctif d6e454c. Chemins ABSOLUS dans les
  définitions embarquées (le PATH minimal de launchd était la cause du refus
  « commande introuvable »). Preuves : spawn réel des deux pilotes,
  daemon volontairement privé de PATH pour Claude.
- **Condition 2 (reprise de session)** : FAIT et prouvé deux fois en réel —
  au redémarrage de 11h47 (agents sans mission) puis à celui de 12h24 :
  cursor4, tué en plein codage, a repris sa mission SEUL en citant ses
  identifiants.
- **Condition 3 (présence stable à la reconnexion)** : PARTIEL et instruit —
  les inscriptions fraîches tiennent (mode, domaine, étiquette de modèle) ;
  la RE-connexion perd encore les attributs enrichis. Constat au registre,
  mission liée --constat-id en cours (cursor5). Jugé non bloquant : le
  défaut est d'affichage, pas d'autorité.
- **Condition 4 (une vue pour l'utilisateur)** : FAIT — `bridget ui` locale
  trois zones + tunnel lecture seule, livrés avant la bascule.

ÉTAT DE LA FLOTTE à 12h40 : coder2, coder3, coder4 en `codex_app_server`
(gpt-5.6-terra épinglé), fable-reviewer en `claude_stream_json`
(claude-opus-5 épinglé), quatre cursor en ACP sain (leur CLI expose ACP
nativement — décision ADR 010 confirmée). La colonne TRANSPORT dit le vrai
protocole ; la colonne LIMITE a affiché son premier fait de forfait réel
(fable-reviewer : fenêtre 5 h, statut, réinitialisation). Extinction tmux :
cxbridget et prospective terminés proprement sur ordre (kill simple, PID par
PID, checklist) ; coderBridget part à sa dernière livraison ; le référent
RESTE en tmux par nature (session interactive de l'utilisateur — un géré
n'a pas de clavier).

LEÇON DE BASCULE consignée : une naissance gérée = spawn + profil Maicie
dans config.json — sinon l'agent est vivant mais indélégable
(target_missing_maicie_profile, constaté sur coder3/coder4, profils ajoutés).

RESTE EN VOL (blocs G/K/L) : effort+forfait Codex dans who (coderBridget,
dernière mission tmux) ; format LIMITE générique par fenêtre « 5h 19%
rst 13:30 · 7d … » + fait par fenêtre + hook référent (coder4, sérialisé
règle 17 avec le précédent) ; attestabilité des limites Cursor
(fable-reviewer, investigation) ; L4 coût par mission (cursor4) ;
reconnexion cursor (cursor5) ; garde M1 des migrations (cursorbridget) ;
vérification G11 (coder3, délégation liée) ; K1 rejoué sur cartae.app au
niveau du jour (coder2). K2 (ronde portable) : LIVRÉ, mergé (ca592ab),
unité com.bridget.ronde ACTIVE sur le Mac, premier rapport daté écrit,
sidecars de la base de prod prouvés intacts.

### Exigence de fond rappelée par l'utilisateur (2026-08-24)

**Trois vues sur le même monde, et le contrôle à distance.** Site web,
application, ligne de commande : ce ne sont pas trois produits mais trois
FENÊTRES sur un état unique. Et il doit être possible de piloter depuis
l'extérieur de la machine.

CE QUE ÇA VALIDE dans l'instruction rendue : le relais est le bon choix
précisément parce qu'il ne crée pas de canal propre à l'interface. Les trois
vues consomment les MÊMES projections — annuaire, demandes, journal
d'attache, état des missions. Une vue qui aurait son propre canal
divergerait des autres au premier changement ; c'est le défaut qu'on vient
de payer deux fois avec les conflits sémantiques.

CE QUE ÇA AJOUTE, et qui n'était pas dans le périmètre instruit :
- **Le distant est une décision de sécurité, pas d'affichage.** Aujourd'hui
  tout passe par une socket locale. La recommandation ajoute une boucle
  locale avec jeton — donc déjà un cran. Exposer hors machine change la
  nature du problème : qui a le droit d'agir, avec quelle authentification,
  et surtout la garde qui doit tenir en priorité — l'approbation Maicie
  exige une frappe humaine sur un vrai terminal, aucun paramètre ne la
  script. Cette garde ne doit PAS être affaiblie pour rendre le distant
  possible.
- **Il existe déjà de la matière** : `scripts/deploy-remote.sh` et
  `scripts/federate-ssh.sh` dans le dépôt. À instruire avant d'inventer :
  peut-être que le chemin distant est un tunnel plutôt qu'une exposition.
- **Lecture et écriture ne se valent pas.** Consulter l'état à distance est
  peu risqué ; lancer un agent, arrêter un daemon ou approuver depuis
  l'extérieur ne l'est pas. Un premier distant en LECTURE SEULE livrerait
  l'essentiel du besoin — voir où on en est depuis ailleurs — sans ouvrir la
  surface d'action.

ORDRE PROPOSÉ : la page locale d'abord (elle fait la projection et le
relais, communs aux trois vues), puis le distant en lecture, puis les
actions à distance avec leur propre arbitrage de sécurité.
- **Une machine d'essai réelle existe : cartae.app (ssh -p 2222) — offerte
  par l'utilisateur le 24/08.** Reconnaissance faite : Ubuntu Linux x86_64,
  accès sudo sans mot de passe, git présent, PAS de toolchain Rust.
  L'ancienne version de Bridget (0.1.0 du 15/08, lien /usr/local/bin vers un
  checkout) a été RETIRÉE sur ordre — archivée et non supprimée, dans
  ~/anciennes-versions/ : le lien est défait, le checkout déplacé. Un
  federation.env d'une ligne subsiste dans ~/.config/bridget, non lu (peut
  porter un secret), conservé.
  CONSÉQUENCES POUR LE BLOC K, et elles sont structurantes :
  (1) K1 est écrit pour macOS — plists launchd. La machine d'essai est
  Linux : il faut un chemin systemd. L'installateur doit DÉCLARER ses cibles
  et refuser proprement une plateforme non couverte, plutôt que d'échouer à
  moitié posé.
  (2) Pas de Rust sur la cible : le déploiement réel exige soit une
  compilation croisée depuis le Mac (aarch64→x86_64), soit l'installation de
  la toolchain sur place, soit des binaires publiés. À trancher dans K1.
  (3) C'est le banc d'essai du chemin VIERGE que le bac à sable ne peut pas
  prouver : l'activation des services sur machine neuve. Le premier
  déploiement réel s'y fera — après que l'installateur aura passé ses preuves
  locales, jamais avant.
  ARBITRAGE UTILISATEUR (24/08, le posant lui-même) : les manques ne sont pas
  des limites à déclarer mais des cas que l'installateur DOIT savoir
  traiter — « il faut que l'installateur se débrouille ». Cibles retenues :
  macOS ET Linux, Windows explicitement différé. Donc : détection de
  plateforme, launchd OU systemd selon la cible, et prise en charge de
  l'absence de Rust — amorcer la toolchain ou livrer des binaires, au choix
  argumenté du lot. Critère d'acceptation : l'installateur prouvé sur LES
  DEUX plateformes — le Mac de l'utilisateur et cartae.app.
