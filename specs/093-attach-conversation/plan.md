# Plan 093 — Renderer conversationnel

## Contexte technique

Rust workspace, bridget-daemon, renderer unique dans crates/bridget-daemon/src/attach.rs.
Réutiliser TurnBlock, BlockRenderer, RendererCommand, RawTerminal, InputBuffer,
terminal_geometry, unicode-width et PseudoTerminal. Aucune modification du daemon,
des sessions Codex, de la socket ni du journal transport.

## Décisions et réemploi

1. Étendre les blocs existants ; séparer présentation conversation TTY et rendu
   non-TTY déjà utile comme diagnostic. Ne pas filtrer à l'ingestion.
2. Markdown : parser CommonMark reconnu `pulldown-cmark`, fonctions inutiles
   désactivées, plutôt qu'un parseur maison. Renderer local mince pour styles
   fermés et disposition ; pas de termimad/crossterm à côté du renderer existant.
   Version et dépendances transitives vérifiées avant adoption définitive.
3. Corps en lignes sous en-tête, plus d'indentation par taille d'UUID ; nom attesté
   via Presence déjà disponible, repli compact sans modifier agent_id.
4. Tout style est généré localement après neutralisation du texte. Le calcul de
   cellules ne doit pas compter les SGR. Réinitialiser le style à chaque ligne.
5. Reflow sur géométrie actualisée dans la boucle renderer existante. Garder au
   plus le bloc actuel/dernier bloc géré, borné ; aucun historique bis. L'ancien
   scrollback déjà engagé reste propriété du terminal.
6. Réponse fragmentée : rendre le buffer borné du bloc, puis rerendre à sa mise à
   jour. Complexité O(n) par rendu avec n borné par MAX_TURN_BLOCK_BYTES ; traitement
   regroupé par apply_batch, mais coût total potentiellement O(F × n) pour F fragments.
   Borner la cadence des parses/redessins avec flush terminal immédiat, ou mesurer
   explicitement un budget acceptable sur des milliers de fragments ; apply_batch
   seul n'est pas une preuve de borne. Pas de parse à chaque octet clavier.

## Fichiers et ownership

- Agent 23 Sol high : crates/bridget-daemon/src/attach.rs, sous-module privé
  attach_renderer.rs pour isoler/tester la projection Markdown pure, ses tests,
  crates/bridget-daemon/Cargo.toml et Cargo.lock pour la dépendance approuvée.
- Pilote : specs/093-attach-conversation/*, README FR/EN et skill après stabilisation,
  revue, tests hors sandbox, build et installation atomique.
- Ne pas modifier les hunks 090/091/092 étrangers. Aucun stage/commit automatique.

Le module privé est chargé depuis attach.rs (pas de nouvelle API publique). Il
porte parsing, styles fermés et cellules visibles, trois responsabilités de
présentation cohérentes. Les noms des autres émetteurs peuvent venir du même
résultat ListAgents déjà lu, en table bornée et sans nouvel appel.

## Phases

Contrat et recherche → reuse-audit PASS → tâches → analyze → tests de régression et
implémentation → revue adverse → convergence → gate workspace → release et installation.
Tests ciblés pendant le travail ; gate complet en fin, conformément au choix utilisateur.

## Constitution et risques

XIX/XX : une dépendance de parsing remplace une complexité syntaxique non triviale ;
pas de nouvelle boucle service ni deuxième renderer d'écran. Isolation : branche
session-093-attach-conversation dans le worktree physique091 existant, pour garder
le cwd réellement autorisé de l'équipier vivant, avec WIP092 validé conservé.
Modifications de production déléguées, plafond Sol high. Scripts/templates SpecKit
absents : création manuelle équivalente des artefacts, pas de modification du runtime
SpecKit. Sync utilisateur exécutée : aucun changement nécessaire.

## Vérification

Unitaires du parser + vrais points d'entrée BlockRenderer et PseudoTerminal,
resize à TIOCSWINSZ, batch fragmenté, erreurs visibles, contrôle ANSI hostile,
comparaison sans couleur. Gate release via binaire installé et démonstration attach.
Pas d'autre fournisseur connecté pour contre-revue ; relecture indépendante Codex
si disponible, sans la présenter comme revue inter-fournisseurs.
