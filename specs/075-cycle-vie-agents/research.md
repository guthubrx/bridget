# Recherche et décisions - SPEC-075

## Question

Comment exposer un cycle de vie d'agent compréhensible et durable sans créer un
second superviseur ni confondre interruption d'un travail, arrêt d'un processus,
reprise et retrait de l'inventaire?

## Sources internes

- SPEC-009 fournit le superviseur, les groupes de processus, `SpawnOrder`,
  `StopOrder`, la génération, la définition figée et `fleet.json`.
- SPEC-071 fournit l'identité runtime affichée dans la fiche.
- SPEC-073 fournit le bouton à trois points, les confirmations, le relais HTTP
  local et les tests d'absence de succès optimiste.
- Le code actuel retire l'entrée persistante et le roster dans
  `FleetSupervisor::invalidate_for_stop`. Ce fait explique l'impossibilité de
  relancer et la disparition après redémarrage.
- Les tables SQLite de spawn conservent les générations et les enveloppes, mais
  ne constituent pas l'inventaire courant. Les parcourir sans borne ferait
  réapparaître de vieux agents historiques.

## Sources externes vérifiées le 2026-08-30

1. OpenAI Codex app-server sépare explicitement `turn/interrupt`,
   `thread/resume`, `thread/archive`, `thread/unarchive` et `thread/delete`.
   Cette séparation confirme qu'une interruption, une reprise, un archivage et
   une suppression ne doivent pas partager un libellé ou une issue.
   Source officielle:
   https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md
2. Anthropic recommande des primitives simples et composables, avec contrôle
   humain et états observables, plutôt qu'un framework autonome supplémentaire.
   Sources officielles:
   https://www.anthropic.com/engineering/building-effective-agents
   https://www.anthropic.com/research/trustworthy-agents
3. Anthropic observe que les utilisateurs expérimentés supervisent et
   interrompent davantage au lieu d'approuver chaque action. L'interface doit
   donc rendre l'état et l'intervention fiables, sans confirmations redondantes
   pour les actions non destructrices.
   Source officielle:
   https://www.anthropic.com/research/measuring-agent-autonomy

## Décisions

### D1 - Faire évoluer `fleet.json`

Décision: `fleet.json` devient l'inventaire durable des agents gérés, et plus
seulement la liste des agents persistants à reprendre. Son schéma 4 ajoute
`persistent` et `lifecycle_state`.

Pourquoi: le même écrivain atomique, le même verrou et la même source de vérité
peuvent porter la reprise et l'affichage des agents arrêtés. Une nouvelle base
ou un second registre introduirait une divergence à réconcilier.

### D2 - Conserver deux dimensions distinctes

- `lifecycle_state`: `running`, `stopped` ou `decommissioned`.
- `persistent`: reprise automatique d'une entrée `running` au démarrage.

Pourquoi: « arrêté » et « non persistant » ne signifient pas la même chose. Un
agent non persistant actif devient arrêté au redémarrage, mais reste connu.

### D3 - Relancer depuis la définition figée

Décision: une relance consomme la dernière définition runtime durable et crée
une nouvelle génération sous le même nom.

Pourquoi: relire le registre mutable transformerait silencieusement une relance
en changement de provider, modèle, protocole ou permissions.

### D4 - Composer le décommissionnement avec l'arrêt

Décision: un agent actif est d'abord arrêté par le superviseur existant. Il ne
passe en `decommissioned` qu'après un verdict d'arrêt positif. Cette entrée
reste cachée, réserve le nom et sépare l'ancien historique d'un futur spawn. En
cas d'échec, l'entrée arrêtée est conservée et l'opération peut être retentée.

Pourquoi: supprimer avant de connaître le sort du processus créerait un
processus potentiellement vivant que Bridget ne reconnaît plus comme géré.

### D4bis - Ne pas réactiver un mandat clos

Décision: une relance administrative conserve le nom, le runtime, le projet et
la politique de persistance, mais crée une génération sans ancien parent ou
mandat actif. La filiation précédente reste dans l'historique.

Pourquoi: l'arrêt ferme le lien de délégation. Le réutiliser automatiquement
pourrait relancer un objectif terminé ou rattacher l'agent à une instance parent
qui n'existe plus.

### D5 - Ne pas reconstruire tout l'historique

Décision: la migration héritée n'adopte que les agents arrêtés explicitement
présents dans l'instantané pré-mise-à-niveau et dont une génération gérée est
vérifiable. Elle ne transforme pas toutes les anciennes commandes SQLite en
agents visibles.

Pourquoi: l'historique contient des générations anciennes et des noms réutilisés.
Une reconstruction globale polluerait la flotte et inventerait un état courant.

### D6 - Conserver des commandes explicites

Décision: maintenir `StopOrder` et ajouter `RelaunchOrder` et
`DecommissionOrder`, chacun avec son résultat fermé.

Pourquoi: des variantes explicites rendent le protocole et les journaux
compréhensibles et évitent qu'un champ d'action générique accepte une valeur
inconnue ou soit mal routé.

## Alternatives rejetées

- Déduire les agents arrêtés de toute la table `spawn_commands`: trop de faux
  positifs historiques.
- Conserver l'arrêt seulement en mémoire: disparition au redémarrage.
- Utiliser `persistent=false` comme état arrêté: mélange politique de reprise et
  état du processus.
- Décommissionner en supprimant d'abord puis en tuant: perte d'autorité en cas
  d'échec partiel.
- Ajouter un service de cycle de vie: duplication du superviseur Rust existant.

## Dégradation d'outillage

- La commande `mem` n'est pas installée sur le serveur. Aucun contexte DevKMS
  n'a donc pu être chargé ou capturé.
- Le synchroniseur SpecKit a été lancé, mais le CLI `specify` requis n'est pas
  installé. Les artefacts sont produits directement dans le dossier de spec,
  conformément au mode dégradé de la skill.
