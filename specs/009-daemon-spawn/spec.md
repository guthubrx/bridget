# Feature Specification : Équipiers gérés par le daemon — `daemon-spawn`

**Feature Branch**: `session-09-daemon-spawn` (après la 008 ; spec anticipée
pendant son implémentation)
**Created**: 2026-08-22
**Status**: Clôturée — implémentée, revue et mergée (12/12)
**Input**: User description: « le daemon lance les équipiers lui-même : plus
besoin d'un wrapper-processus ouvert dans un terminal ; les équipiers
persistent, façon équipiers cloud, mais en local et sur mes abonnements. »

## Contexte et problème

Depuis la 007, un équipier ACP est un sous-processus du **wrapper**, lui-même
attaché au terminal où l'utilisateur a tapé `bridget codex --equipier` : fermer
ce terminal tue l'équipier. L'équipier « persistant » — celui qu'on lance le
matin et qu'on retrouve le soir, pilier de la promesse produit — n'existe pas
encore. Cette session déplace la **propriété du cycle de vie** : lancer un
équipier devient un **ordre au daemon**, qui le fait naître, le supervise et
le fait mourir — le terminal de l'utilisateur n'est plus qu'un client qui
donne des ordres et s'en va.

## Dépendances et frontières

- **007** : tout est réutilisé — registre, transport ACP, journal v1, états,
  garde de facturation. La parité de garanties est une exigence (FR-008).
- **008** : `attach` doit fonctionner à l'identique sur un équipier
  daemon-géré (même flux, même rôle de connexion).
- **010** (frontière fixée par sa spec, FR-012) : la 009 ne fait que
  **transmettre** la configuration MCP du registre au lancement — aucune
  logique MCP.
- Fédération (002) : hors périmètre d'évolution — un équipier daemon-géré
  distant reste lancé par le daemon de sa machine.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Lancer un équipier qui survit au terminal (Priority: P1)

L'utilisateur tape `bridget spawn codex` (nom de commande définitif au plan).
La commande retourne dès que l'équipier est enregistré et prêt — nom attribué,
visible dans `bridget who`. L'utilisateur ferme son terminal : l'équipier
continue de travailler, reçoit des messages, répond. Plus tard, depuis
n'importe quel terminal : `bridget stop <nom>` l'arrête proprement.

**Why this priority** : c'est la raison d'être de la session — l'équipier
persistant. Sans elle, rien d'autre n'a de sens.

**Independent Test** : spawn depuis un shell, fermer le shell, envoyer une
demande suivie depuis un autre terminal → réponse reçue et demande close ;
`stop` → état `stopped` dans l'annuaire, processus disparus.

**Acceptance Scenarios**:

1. **Given** le daemon tourne, **When** l'utilisateur ordonne un spawn,
   **Then** la commande retourne avec le nom attribué une fois l'équipier
   `connected`, et l'annuaire l'affiche avec son transport et un marqueur de
   gestion daemon.
2. **Given** un équipier daemon-géré, **When** le terminal qui a donné l'ordre
   se ferme, **Then** l'équipier poursuit : il reçoit, travaille et répond
   (démontré par un échange complet après fermeture).
3. **Given** un équipier daemon-géré, **When** `bridget stop <nom>` est émis
   depuis n'importe quel terminal, **Then** l'arrêt suit le chemin propre de
   la 007 (annulation du tour actif, drain, `Unregister`) et l'annuaire montre
   `stopped`.
4. **Given** un nom déjà actif, **When** un spawn du même nom explicite est
   ordonné, **Then** refus motivé — jamais deux équipiers sur le même nom.

---

### User Story 2 - Un cycle de vie observable et honnête (Priority: P2)

Les échecs et la santé des équipiers daemon-gérés sont visibles : un spawn qui
échoue (commande absente, garde de facturation, adaptateur incompatible)
retourne l'erreur **au client qui a donné l'ordre**, avec le motif exact du
registre ou du lancement ; un équipier qui meurt seul est détecté et son état
reflété ; sa sortie d'erreur est conservée et consultable.

**Why this priority** : déléguer le cycle de vie au daemon sans observabilité
créerait des zombies invisibles — l'inverse de ce que la 007/008 ont bâti.

**Independent Test** : spawn d'un type à commande inexistante → erreur motivée
au client, rien dans l'annuaire ; tuer le processus équipier → transition
d'état conforme à la table 007 + échecs motivés des demandes en cours ;
consulter la sortie d'erreur conservée du lancement raté.

**Acceptance Scenarios**:

1. **Given** une entrée de registre à commande absente, **When** spawn,
   **Then** l'erreur nomme la commande et le fichier de registre, la commande
   client sort en échec, et l'annuaire ne contient aucune trace.
2. **Given** une variable interdite (`forbidden_env`) présente dans
   l'environnement du daemon, **When** spawn, **Then** le refus de la garde de
   facturation (007-T710) est retourné au client — la garde s'applique au
   daemon comme au wrapper.
3. **Given** un équipier daemon-géré dont le processus meurt, **When** la mort
   est détectée, **Then** les demandes en cours reçoivent leurs échecs motivés
   et l'état suit la table 007 (`stopped`, pas `unreachable`).
4. **Given** un lancement qui a échoué après démarrage partiel, **Then** la
   sortie d'erreur du processus est consultable a posteriori (fichier de log
   par équipier, emplacement documenté).

---

### User Story 3 - Reprise déclarative au redémarrage du daemon (Priority: P3)

Un équipier peut être marqué **persistant** au spawn. Au redémarrage du daemon
(reboot, mise à jour), les équipiers persistants sont relancés automatiquement
avec leur nom et leur configuration ; les non-persistants ne le sont pas. Un
`stop` explicite retire toujours le marqueur.

**Why this priority** : complète la promesse « je le retrouve le matin » ;
s'appuie sur les deux stories précédentes sans les conditionner.

**Independent Test** : deux spawns (un persistant, un non), arrêt/relance du
daemon → le persistant revient `connected` sous le même nom, l'autre non ;
`stop` du persistant puis relance du daemon → il ne revient pas.

**Acceptance Scenarios**:

1. **Given** un équipier persistant et un éphémère, **When** le daemon
   redémarre, **Then** seul le persistant est relancé, sous son nom, et
   l'événement de reprise est consigné.
2. **Given** un équipier persistant arrêté par `stop`, **When** le daemon
   redémarre, **Then** il n'est pas relancé (le stop vaut retrait du marqueur).
3. **Given** une reprise dont le lancement échoue (registre changé, commande
   absente), **Then** l'échec est consigné et visible, sans bloquer le
   démarrage du daemon ni les autres reprises.

---

### Edge Cases

- **Arrêt du daemon — deux régimes distincts** (révisé en contre-revue) :
  - *arrêt coopératif* (signal d'arrêt normal) : les équipiers s'arrêtent avec
    lui par le chemin propre (annulation, drain, journal) — zéro orphelin ;
  - *arrêt brutal* (SIGKILL, crash, OOM, coupure) : aucun nettoyage ne peut
    s'exécuter — des descendants peuvent survivre reparentés (npx compris).
    La garantie est alors la **réconciliation au prochain démarrage** : chaque
    équipier vit dans son **groupe de processus** dédié, tracé par un marqueur
    (pgid + heure de naissance + `instance_id`, validation anti-pid-recyclé) ;
    au démarrage, le daemon termine les groupes périmés **avant** toute
    reprise. **Invariant de couverture** (round 3) : aucun enfant n'est
    autorisé à exécuter/continuer **avant la durabilité de son marqueur** —
    la fenêtre spawn OS → écriture du marqueur ne peut pas produire de
    survivant non réconciliable (mécanisme au plan, ex. bootstrap retenu par
    pipe libéré après fsync). Tests de crash aux trois frontières : avant
    marqueur, après marqueur, après libération. Test imposé : SIGKILL du
    daemon → redémarrage → aucun ancien groupe survivant + une seule nouvelle
    instance par persistant.
  C'est la reprise déclarative (US3) qui assure la continuité — pas des
  survivants détachés.
- **Environnement du daemon** : un daemon lancé hors session utilisateur
  (launchd minimal) peut manquer des variables nécessaires aux logins CLI —
  l'hypothèse d'environnement est documentée et vérifiée au spawn (échec
  motivé si l'environnement est manifestement inapte, ex. `HOME` absent).
- **Spawn pendant l'arrêt du daemon** : refusé proprement.
- **Deux ordres de spawn simultanés du même nom** : un seul gagne, l'autre
  reçoit le refus « nom déjà actif ».
- **Équipier wrapper-terminal (mode 007) et équipier daemon-géré coexistent** :
  les deux modes restent disponibles ; l'annuaire les distingue ; aucun
  changement pour les agents tmux.
- **Limite de flotte** : un plafond configurable d'équipiers daemon-gérés
  simultanés (défaut raisonnable), dépassement refusé avec motif.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** : une commande client DOIT permettre de lancer un équipier d'un
  type du registre **par ordre au daemon**, qui en devient le propriétaire du
  cycle de vie, selon une **machine d'états explicite** :
  `Requested → Reserved → Starting → Connected | Failed | Cancelled`. L'ordre
  porte un **`command_id` idempotent** (une relance client après réponse
  perdue retourne l'issue de l'ordre initial, jamais un second équipier) ; le
  nom et le slot de quota sont **réservés atomiquement** (une seule section
  critique) avant tout spawn ; l'ordre est lié à l'enregistrement par
  l'`instance_id` ; un **délai absolu** borne le tout — à expiration, le
  daemon annule, récolte, draine et libère la réservation **avant** de
  répondre, et un `Register` tardif de cette instance est rejeté.
  **Portée de l'idempotence** (round 3) : `command_id` et son issue terminale
  sont conservés dans un **registre borné à rétention fixée** (détail au
  plan) qui **survit au redémarrage du daemon** pour les spawns persistants —
  la mise à jour de l'état désiré et de l'issue est ordonnée
  transactionnellement pour qu'un retry rejoue le même résultat ; pour les
  spawns éphémères, l'idempotence vaut pour la vie du daemon (documenté).
  Tests à barrières : deux spawns simultanés du même nom, timeout juste avant
  `Register`, relance du même `command_id` — y compris après redémarrage pour
  un persistant.
- **FR-002** : un équipier daemon-géré DOIT survivre à la fermeture du
  terminal et de la session shell qui ont donné l'ordre, et rester pleinement
  fonctionnel (livraison, tours, réponses, journal, attach).
- **FR-003** : `bridget stop <nom>` DOIT arrêter proprement un équipier
  daemon-géré depuis n'importe quel client, **descendants compris** : réponse
  synchrone typée (`StopOutcome`) rendue seulement après annulation + grâce +
  arrêt forcé du **groupe de processus** si nécessaire, récolte (`wait`),
  drain terminal, fermeture du journal de lancement, `Unregister` → `stopped`
  — le tout sous délai borné. Le `stop` d'un agent actif **non** daemon-géré
  est refusé avec motif (jamais tuer un wrapper-terminal 007). Tests :
  adaptateur qui ignore l'annulation ; `npx` ayant créé un descendant.
- **FR-004** : l'annuaire DOIT distinguer les équipiers daemon-gérés (marqueur
  de gestion), sans changer les colonnes existantes pour les autres agents.
- **FR-005** : tout échec de lancement DOIT être retourné au client donneur
  d'ordre avec le motif exact (registre, garde de facturation, commande,
  environnement inapte) et ne laisser **aucun état opérationnel résiduel**
  (annuaire, processus, réservations) — le **journal de lancement** (stderr,
  FR-007) est explicitement **exempté** : il est conservé pour le diagnostic,
  sous la politique de rétention documentée (purge par âge, même règle que les
  journaux de session).
- **FR-006** : la mort spontanée d'un équipier daemon-géré DOIT être détectée
  par le daemon et produire les mêmes effets qu'en 007 : échecs motivés des
  demandes en cours, état conforme à la table de transitions, journal cohérent.
- **FR-007** : la sortie d'erreur (stderr) de chaque équipier daemon-géré DOIT
  être conservée dans un fichier par équipier (emplacement documenté, mêmes
  permissions que le journal 0700/0600), consultable après un échec.
- **FR-008** : **parité de garanties mesurée** : une **matrice versionnée des
  garanties 007** (accusés/refus, journal v1, états, relances, garde de
  facturation) est exécutée sur le **même corpus** en mode wrapper-terminal
  puis en mode daemon-géré, et les observables sont comparés (N essais et
  tolérances fixés dans la matrice — pas de « 100 % » sur un échange unique) ;
  pour l'attach 008, la **suite de frames** est comparée sur la même fixture
  dans les deux modes.
- **FR-009** : un spawn DOIT refuser un nom déjà actif (« nom déjà actif »,
  motivé) ; deux ordres simultanés du même nom → un seul gagnant.
- **FR-010** : le marqueur **persistant** est optionnel au spawn ; au
  démarrage, le daemon relance les équipiers persistants depuis un **état
  désiré** aux invariants fixés : source de vérité unique, version de schéma,
  clé stable par équipier, **écriture atomique** (temp + fsync + rename),
  permissions 0700/0600, **ordre de reprise déterministe**, flotte désirée
  supérieure au quota → excédent consigné non relancé, conflit de nom avec un
  wrapper-terminal actif → reprise refusée consignée, **aucun retry infini**
  (échec de reprise consigné, pas répété en boucle). Une **matrice de
  reprise** (succès / échec / conflit / quota) fait partie des tests. `stop`
  retire durablement le marqueur. **Point de linéarisation fixé** (round 2 de
  la contre-revue) : aucune réponse `Connected` au client **avant l'écriture
  durable** de l'état désiré (fsync du fichier temporaire, `rename`, fsync du
  répertoire parent) ; un échec **avant** cette écriture ne laisse aucune
  entrée ; un échec **après** retire durablement l'entrée avant de répondre
  `Failed`. Tests à points de crash : avant le rename, après le rename, avant
  la réponse — reprise et idempotence vérifiées à chaque point. **Propriété
  du fichier** : le daemon est le **seul écrivain** ; le fichier n'est
  éditable à la main que daemon arrêté, il est chargé au démarrage, et toute
  modification à chaud passe par `spawn`/`stop` (pas de rechargement à chaud —
  hors périmètre). (Structures détaillées au plan.)
- **FR-010bis** : **`stop` gagne sur la reprise** : un `stop` pendant qu'un
  spawn de reprise est en vol invalide sa génération (jeton d'annulation ou
  tombstone atomique), retire l'entrée de l'état désiré **avant** de répondre,
  et un `Register` de génération obsolète est rejeté — un équipier arrêté ne
  ressuscite jamais. Test à barrière : `stop` entre le spawn OS et le
  `Register`.
- **FR-011** : à l'**arrêt coopératif** du daemon, les équipiers daemon-gérés
  sont arrêtés par le chemin propre — zéro orphelin ; après un **arrêt
  brutal**, la **réconciliation au démarrage** termine les groupes de
  processus périmés (marqueurs pgid + naissance + instance, anti-pid-recyclé)
  **avant** toute reprise. L'arrêt du daemon n'efface jamais l'état désiré.
- **FR-011bis** : environnement et répertoire de travail **explicites** :
  l'ordre de spawn porte un `cwd` **absolu capturé chez le client** et validé
  (existant, accessible) ; l'environnement de l'enfant est **construit** selon
  une politique documentée à partir de celui du daemon et du registre — jamais
  copié aveuglément du client ; `HOME`, `PATH`, `cwd` et la lisibilité de
  l'auth du CLI sont validés **avant** la réservation finale ; `forbidden_env`
  garde son autorité (FR-005). Cas testés : `cwd` disparu entre l'ordre et le
  spawn ; daemon lancé avec `PATH` minimal.
- **FR-011ter** : abonnements attach (008) et cycle de vie : `stop`, mort ou
  arrêt du daemon produisent un `End` typé vers **chaque** abonnement de
  l'équipier ; après une reprise, les vues se réabonnent et reçoivent un
  **nouveau** `subscription_id` — aucune ancienne génération ne reçoit le
  nouveau flux. Test : attach actif pendant `stop` puis reprise persistante.
- **FR-012** : une **limite configurable** d'équipiers daemon-gérés simultanés
  s'applique (dépassement refusé avec motif) ; le mode wrapper-terminal 007
  reste disponible et inchangé.
- **FR-013** *(réduite deux fois en contre-revue)* : la 009 **réutilise le
  chemin partagé de construction et de lancement de la 007** et ne duplique
  aucun champ d'`AgentDefinition`. Aucune promesse sur les extensions futures
  du registre : la 010 modifiera ce chemin partagé si son champ MCP le
  requiert — c'est elle qui en est propriétaire.

### Key Entities

- **Équipier daemon-géré** : équipier ACP dont le cycle de vie appartient au
  daemon — mêmes attributs d'annuaire qu'en 007 plus le marqueur de gestion et
  l'éventuel marqueur persistant.
- **Ordre de cycle de vie** : spawn / stop, émis par un client, avec réponse
  synchrone motivée.
- **État désiré** : liste déclarative des équipiers persistants (type, nom,
  options), source de la reprise au démarrage.
- **Journal de lancement** : stderr conservée par équipier, complément du
  journal de session v1.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : sur **N = 20 spawns** (adaptateur npx en cache, environnement
  gelé et consigné) : nom retourné et équipier `connected` avec **p95 < 10 s**
  et timeout global fixé ; puis, terminal donneur d'ordre fermé, **20/20**
  échanges complets (demande suivie → réponse → clôture) aboutissent.
- **SC-002** : `stop` → `StopOutcome` synchrone, `stopped` dans l'annuaire et
  **zéro processus résiduel du groupe, descendants compris** (vérifié par
  liste de processus du pgid), sur les scénarios : tour actif en cours,
  adaptateur ignorant l'annulation, descendant `npx` présent.
- **SC-003** : **table fermée de 11 familles** de refus (identique à celle du
  contrat), chacune testée avec son motif typé et l'invariant « **aucun nouvel
  état opérationnel imputable au refus** » (formulation qui reste vraie pour
  les retries idempotents) : type inconnu, commande absente, garde de
  facturation, nom déjà actif, environnement inapte (`HOME`/auth/`PATH`),
  `cwd` disparu, négociation ACP incompatible, timeout avec `Register` tardif
  rejeté, quota saturé, daemon en récupération, idempotence expirée.
- **SC-004** : parité 007 mesurée selon FR-008 : la matrice versionnée de
  garanties passe sur le même corpus dans les deux modes avec les tolérances
  fixées ; pour l'attach 008, la suite de frames est identique sur la même
  fixture dans les deux modes.
- **SC-005** : reprise : sur 3 cycles redémarrage du daemon, les persistants
  reviennent 3/3 sous leur nom, les éphémères 0/3 ; un `stop` avant
  redémarrage exclut de la reprise 3/3.
- **SC-006** : arrêt **coopératif** du daemon avec N équipiers actifs → N
  arrêts propres journalisés, zéro orphelin (liste de processus avant/après) ;
  **SIGKILL** du daemon → au redémarrage, réconciliation : aucun ancien groupe
  survivant, une seule nouvelle instance par persistant.

## Assumptions

- Le daemon tourne dans la session de l'utilisateur avec son environnement de
  login (les CLIs sous-jacents ont besoin de leurs répertoires d'auth) ; le
  fonctionnement sous un daemon système minimal est **hors périmètre** et
  documenté comme tel.
- La « persistance » est une reprise déclarative par le daemon — pas des
  processus détachés survivant au daemon. Garantie exacte : **zéro orphelin
  après un arrêt coopératif ; après un crash, survivants bornés jusqu'à la
  réconciliation du prochain démarrage.**
- Le contexte conversationnel d'un équipier ne survit pas à une reprise (la
  session ACP repart vierge, comme au premier lancement) — la continuité du
  *nom* et de la *configuration* est garantie, celle de la *mémoire de session*
  ne l'est pas ; limite documentée.
- Le mécanisme réutilise l'infrastructure 007 au maximum ; la répartition
  wrapper/daemon exacte (le daemon absorbe-t-il le code du wrapper ou
  lance-t-il le wrapper existant en enfant détaché ?) est une décision du
  plan, pas de la spec.
