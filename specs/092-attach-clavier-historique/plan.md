# Plan 092

## Complément US5 — raccourcis classiques et Shift+Entrée iTerm

Suite autorisée du même clavier : US5 corrige la confusion CR/LF attestée par la
configuration iTerm et l'envoi constaté par l'humain. Aucun changement de réglage
global. RawTerminal conserve CR/LF distincts uniquement en double-TTY ; le décodeur
ou le handler conserve cette distinction jusqu'à la politique TUI/non-TUI.
Réutiliser InputBuffer, snapshot et bornes de mot US4 pour toutes les commandes.
Un seul registre volatile de dernier fragment supprimé, sans presse-papiers externe.
Les lignes logiques sont délimitées par LF, jamais les lignes de wrapping renderer.

Ownership inchangé : Agent23 code attach.rs/tests + README FR/EN + skill du worktree ;
pilote spec/plan/tasks/recette, review/tests/release/install. Aucun autre fichier
de production, dépendance, protocole ou redémarrage. Resize093 reste figé.
Tests nouveaux rouges d'abord sur la garde réelle avec ICRNL actif, puis matrice
US5 au handler. Grille réelle puis fmt/clippy et consolidation finale unique.

## Complément US4 — édition au curseur, 06/09

Continuation du clavier 092 validée par l'humain. Même worktree autorisé, branche
de travail cumulative 093 conservée pour ne pas déplacer les WIP ; aucun merge
ni commit automatique. Resize 093 figé, pas de CPR/plein écran dans ce lot.

Réutilisation : InputDecoder, InputBuffer, input_snapshot, RendererSender,
BlockRenderer et PseudoTerminal sont les coutures uniques. Étendre le curseur
logique dans InputBuffer et son brouillon de navigation ; snapshot texte/position
sous le même verrou. Le renderer projette ce curseur sur les rangées de saisie,
pas sur le Markdown du journal. Fenêtre de saisie autour du curseur, statut dessous.
Complexité attendue : O(n) insertion/effacement Vec et déplacement par mot, O(n)
projection en cellules, aucune recherche imbriquée. Séquences bornées32 préservées.

Propriétaire code : agent23, attach.rs/tests + README FR/EN + skill active.
Pilote : artefacts, revue, exécution des tests autorisés, build et installation
après preuve, sans restart daemon ou agents. Préserver WIP des autres fichiers.
Point d'intégration : le candidat de resize rejeté ne doit pas être installé
implicitement avec ce clavier ; isoler le delta avant le build final.

Tests ciblés après les tâches puis consolidation en fin de lot ; oracle rouge
pour les nouvelles touches avant correction. Gate réel : émulateur existant en
PTY, texte final ET position physique du curseur. Pas de nouvelle infrastructure.

## Contexte technique et isolation

Rust 1.92.0, libc/Unicode existants, entrée TTY et renderer dans attach.rs.
Branche session-092-attach-clavier-historique depuis ea52cd04 ; worktree dédié
réutilisé : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux.
Le chemin reste 091 pour conserver les droits du processus déjà lancé par l'humain.
091 est mergée et propre ; son fichier de preuve non suivi est préservé.
Pas de partage du fichier de production avec le pilote. Pas de permissions élargies.

## Architecture choisie

1. Étendre l'entrée locale existante avec un petit décodeur incrémental borné
   (32 octets stockés au maximum par séquence). Réutiliser handle_input_byte et
   InputBuffer, supprimer alt_prefix lorsqu'il est remplacé, ne pas doubler la voie.
2. Activer une pile de mode clavier compatible Kitty (désambiguïsation seulement)
   sur TTY non dumb ; restaurer exactement la pile ouverte, jamais les préférences
   iTerm2. La garde locale possède cette ressource comme termios. Aucune attente
   de réponse terminal avant de rendre la main. Protocoles détaillés au contrat.
3. Historique dans InputBuffer (VecDeque déjà importé), index de navigation et
   brouillon sauvegardé. À l'édition, quitter la navigation. Enregistrer seulement
   après la même écriture réussie Send/SelectRuntime que 091, sans second envoi.
   Créer cet état à la durée de vie de run_with_input, non à chaque connexion
   drive_interactive (1712 au socle). Réutiliser l'état local après reconnexion,
   jamais pending_send. Activer la navigation uniquement stdin ET stdout TTY ;
   consommer les flèches sans effet ni ANSI en sortie redirigée.
4. Le renderer existant reçoit input_changed après nouvelle ligne/rappel/édition.
   Pas de nouveau renderer, de voie réseau ni de persistance.

## Phases et ownership

- Pilote : spec, recherche, réutilisation, tâches, analyse puis revue/convergence.
- Équipier a4d12c75-5994-4c02-9acc-2db07ba817af : attach.rs et ses tests internes,
  README.md, README.en.md, skills/bridget/SKILL.md, implementation.md 092.
- Les autres fichiers Rust/protocole/wrapper/manifestes sont hors couloir.
- Phase 1 : tests discriminants et décodeur ; phase 2 : mode clavier/restauration ;
  phase 3 : historique ; phase 4 : couture/recette/docs ; phase 5 : revue et audit.

## Stratégie de validation

Fixtures au niveau handle_input_byte sur UnixStream::pair, puis PseudoTerminal
déjà présent pour la couture terminal. Attentes sur état/bytes, pas sleep arbitraire.
Échec attendu avant correctif pour Shift+Entrée et flèches ; modifier le décodage,
la restauration du brouillon ou le point de mémorisation doit casser l'oracle.
Tests pertinents après chaque lot ; consolidation unique à la fin : attach, fmt,
clippy workspace et tests workspace. En cas de rouge extérieur, donner sa sortie
et son périmètre, ne pas corriger opportunément. Pas de doublage en boucle de la suite.

## Minimalisme et sécurité

Complexité : décodeur O(1) par octet (borne fixe), navigation O(taille du corps
rappelé), stockage/éviction O(taille ajoutée + évincée), pas de recherche N×N.
Séquences de contrôle inconnues consommées sans injection de commande ; sorties
terminal constantes, texte toujours via renderer sécurisé. Historique non durable
pour ne pas créer une nouvelle copie de données privées. Aucune permission ajoutée.
Deux petites données privées portent des invariants nécessaires, pas un framework.

## Gates et outillage

Sync utilisateur exécutée. Templates/scripts officiels absents dans ce checkout :
protocole SpecKit appliqué manuellement, sans bootstrap ou génération du runtime.
reuse-audit PASS obligatoire avant tasks ; analyse croisée avant code.
Revue fournisseur différent impossible avec l'annuaire présent (tous Codex) :
ne pas la déclarer faite. Revue du pilote non auteur + tests explicites.
Converge relit FR/SC ; en cas de lacune, ajoute seulement des tâches puis redélègue.
Aucun commit automatique, conformément à my-specify-all fourni par l'utilisateur.
