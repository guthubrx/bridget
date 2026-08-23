# Contrat local — commandes Maicie et frontière Bridget

## Principes

- Les exemples décrivent le contrat cible ; ils ne constituent pas une
  implémentation disponible.
- Le texte humain et la sortie JSON décrivent la même opération.
- Maicie passe par un client Bridget public ; elle ne lit jamais un fichier,
  socket ou crate interne de Bridget.
- Chaque réponse structurée comporte `objective_id` et, s'il existe,
  `message_id` généré par Maicie avant l'appel Bridget.

## Commandes Maicie MVP

### Créer ou déléguer un objectif

```text
maicie delegate --goal <texte> [--to <agent-ou-profil>] [--duration courte|normale|longue] [--json]
```

Effets : crée un `ObjectifCoordonné` et, si une cible est explicitement donnée
ou est l'unique égalité de tags disponible, une `Délégation`. Sinon la sortie
propose les candidats sans choisir. Une activation de profil crée une
approbation, jamais un processus. La demande Bridget porte le timeout de la
classe de durée et un `message_id` déjà durablement persisté.

Réponse JSON minimale :

```json
{
  "objective_id": "uuid",
  "state": "en_coordination",
  "delegations": [{"id": "uuid", "participant": "prospective", "message_id": "uuid", "coordination_state": "prepared"}]
}
```

### Intervenir explicitement

```text
maicie objective <objective-id> add-participant <agent>
maicie objective <objective-id> remove-participant <agent> --reason <texte>
maicie objective <objective-id> summarize
maicie objective <objective-id> close --reason <texte>
```

Effets : produit une décision Maicie auditée. Aucune commande Bridget générale
ne devient une intervention Maicie par effet de bord.

### Consulter

```text
maicie status [<objective-id>] [--json]
maicie profiles [--json]
maicie approve <approval-id>
maicie reject <decision-id> --reason <texte>
```

`status --json` sépare au minimum : `transport_snapshot` (Bridget), `runtime`
(abonnement ACP 008), `coordination`, `freshness` et `unknown`/`stream_gap`.

## Contrat Maicie → Bridget

| Besoin Maicie | Contrat Bridget requis | Règle |
|---|---|---|
| choisir un participant | annuaire JSON avec nom, type, disponibilité, transport, domaine, modèle/effort s'ils sont connus | lecture seule ; absence = `inconnu` |
| déléguer | envoi suivi avec cible, corps, timeout et `message_id` fourni par client ; déduplication par ce même id | l'outbox Maicie est persistée avant I/O |
| réconcilier | lecture d'issue/liste de demandes par `message_id` | nulle lecture SQLite directe ; même id au retry |
| cesser d'attendre | annulation suivie par identifiant et motif | Bridget décide l'issue de livraison |
| constater activité | `Subscribe` session 008 : version/capacités, `subscription_id`, `seq`, `Gap`, `End` | gate 008 obligatoire ; snapshot dérivé, jamais autorité métier |
| réveiller profil | `SpawnOrder` public session 009 | gate 009 obligatoire ; Maicie ne lance pas de processus |

La première version peut adapter les commandes JSON existantes de Bridget si —
et seulement si — elles acceptent l'identifiant client idempotent et l'issue
consultable par identifiant. Sinon, une extension Bridget versionnée, négociée
et testée est un prérequis au code Maicie ; il est interdit de diminuer
SC-001 par une réémission non corrélée.

## Séquence outbox obligatoire

1. Transaction SQLite Maicie : créer `Délégation` et `OutboxDélégation` en
   `prepared`, avec UUID `message_id`, target, bytes exacts du corps, `reply`,
   timeout/échéance et hash de l'enveloppe.
2. Appeler Bridget avec ce même `message_id`.
3. Sur Ack : transaction vers `accepted`. Sur coupure/ack perdu : transaction
   vers `outcome_unknown`.
4. Au redémarrage, toute ligne non terminale — y compris `prepared` — consulte
   Bridget par `message_id`. Si aucune issue n'est connue, réémettre les bytes
   exacts persistés ; Bridget déduplique. Une rétention de tombstone au moins
   égale à `retry_until` est obligatoire ; sinon `idempotency_expired` est
   retourné et le replay est interdit.

Les tests de contrat couvrent : Ack normal, Ack perdu après acceptation,
redémarrage, retry identique et tentative de corps différent sous même id.

## Approbation d'activation et SpawnOrder

`maicie approve` est une commande locale structurée. Une transaction écrit
`ActivationOutbox(command_id, spawn_order_bytes, dispatching)` et passe
l'approbation à `dispatching`, avec `objective_id`, `profile_id`,
`profile_hash`, `context_hash`, périmètre, paramètres, `actor=local_human` et
expiration. Juste avant l'émission, Maicie revalide ces hashes puis émet le
`SpawnOrder` public de session 009 avec le même `command_id`. Après crash ou
Ack perdu, toute ligne non terminale fait lookup/replay exact ; l'approbation
ne devient consommée qu'après issue durable. Maicie ne crée ni `Child`, ni
commande OS, ni logique d'arrêt/reconnexion. Le contexte après connexion est
livré par `OutboxDélégation`, avec les bytes/hash approuvés.

`actor=local_human` est un attribut du modèle de confiance local mono-utilisateur
— il ne prouve pas cryptographiquement qu'un humain physique a saisi la
commande. `approve` n'est jamais exposé comme outil Bridget ou MCP.

## Permissions ACP

Le journal ACP 007 décrit une permission et la réponse automatique déjà choisie
par le registre. Maicie peut afficher ce fait avec son issue et sa séquence,
mais ne peut pas le présenter comme une permission humaine en attente ni
modifier le cycle ACP.
