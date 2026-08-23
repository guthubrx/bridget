# ADR 003 — Maicie est un compagnon d'orchestration hors processus

- **Statut** : Accepté pour la conception ; implémentation non démarrée
- **Date** : 2026-08-22

## Contexte

Bridget prouve en usage réel l'intérêt de la communication directe entre
agents : présence, messages, demande suivie, réponse et reprise restent
fluides. L'ancien Maicie centralisait au contraire une mission dans un
superviseur, un pipeline et des états de tâches qui enfermaient l'utilisateur
dans une exécution prédéfinie.

Le besoin est maintenant double : pouvoir déléguer une intention complète à
une coordinatrice joignable en permanence, ou parler et intervenir directement
auprès des agents. ACP apporte des faits d'exécution par abonnement public ;
Maicie ne lui attribue pas de statut métier. Aucun protocole ne doit transformer
Bridget en moteur de workflow.

## Décision

Maicie v3 sera un **compagnon hors processus** distribué dans
`plugins/maicie/`, avec son propre exécutable et sa propre base SQLite. Elle
utilisera les interfaces locales publiques de Bridget pour :

- lire l'annuaire, la disponibilité et les demandes suivies ;
- envoyer des demandes et choisir leur délai ;
- lire les faits ACP seulement par l'abonnement public session 008 ;
- rester joignable comme coordinatrice nommée.

Bridget conserve la vérité sur le transport (connexion, présence, livraison,
réponse, timeout). Maicie conserve seulement la vérité de coordination (but,
participants, délégations, attentes, synthèse et décision). Il n'existe ni
base SQLite partagée, ni import de modules internes Bridget, ni accès de
Maicie à `bridget.db`.

Une conversation est le mode par défaut. Une intention ne devient un objectif
coordonné que par une commande explicite de l'utilisateur à Maicie. Les
messages directs restent libres et ne modifient pas silencieusement un objectif
coordonné.

## Conséquences

### Positives

- Le transport reste petit, réutilisable et multifournisseur.
- Maicie peut être arrêtée, mise à jour ou remplacée sans migration de l'état
  de livraison Bridget.
- Un futur GUI et un futur TUI pourront consommer les mêmes commandes et vues
  de coordination sans être eux-mêmes la source de vérité.
- Le périmètre initial ne requiert ni DAG, ni scheduler, ni interface web.

### Négatives et garde-fous

- Deux processus imposent des contrats publics et une corrélation explicite ;
  cette friction est préférable à une base partagée cachée.
- ACP ne fournit pas un statut métier universel « bloqué » : Maicie ne devra
  jamais l'inférer d'une sortie terminal ou proposer un rapport implicite.
- L'activation d'un profil passe par une approbation locale mono-usage et le
  SpawnOrder public session 009 ; Maicie ne gère aucun processus.
- DSH et T3 Code sont des références d'ergonomie ou d'architecture, pas des
  dépendances ni des propriétaires d'état.

## Alternatives écartées

- **Réintégrer Maicie dans le démon Bridget** : couplage fort, cycle de
  publication commun et transport alourdi.
- **Reprendre le superviseur de Maicie historique** : pipeline rigide,
  lifecycle de worktrees et fournisseur Codex imposés.
- **Adopter DSH ou T3 Code comme interface canonique** : ces outils possèdent
  leur propre modèle de sessions et évoluent indépendamment ; ils créeraient
  deux sources de vérité.
