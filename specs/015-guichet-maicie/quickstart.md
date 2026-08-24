# Quickstart 015 — Guichet Maicie

Le guichet est livré : Bridget conserve une lettre structurée adressée à
`maicie` quand le compagnon n'est pas lancé ; la prochaine commande Maicie la
relève, la greffe dans son registre privé et répond avec les octets de réponse
persistés. Ce document ne décrit pas une boucle résidente : chaque commande
Maicie reste une invocation courte, pull-only et bornée.

## Préparer les binaires et la configuration

Les chemins ci-dessous sont ceux du worktree de la session. Ils sont absolus
pour que la commande soit copiable sans dépendre du répertoire courant.

```bash
export BRIDGET_ROOT="/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie"
export BRIDGET_BIN="/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget"
export MAICIE_BIN="/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie"
export MAICIE_CONFIG="/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json"
```

```bash
/Users/moi/.cargo/bin/cargo build --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml --release -p bridget-daemon -p maicie
mkdir -p /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie
```

Créer une configuration minimale pour consulter et relever le guichet. La
SQLite Maicie est privée et reste distincte de
`/Users/moi/.cache/bridget/bridget.db`.

```bash
cat > /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json <<'JSON'
{
  "version": 1,
  "bridget_socket": "/Users/moi/.cache/bridget/bridget.sock",
  "database_path": "/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.sqlite3",
  "durations": {
    "short_secs": 30,
    "normal_secs": 300,
    "long_secs": 3600
  },
  "status_capture_budget_ms": 250,
  "profiles": []
}
JSON
```

Dans un terminal dédié, lancer Bridget :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget daemon
```

Puis consulter Maicie. Cette commande négocie la capacité de service et relève
au plus les demandes qui entrent dans son budget ; elle ne garde aucun worker
en arrière-plan après son retour.

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie status --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json --json
```

## Déposer une lettre depuis un équipier réel

`bridget guichet deposer` est une commande producteur : elle doit être lancée
dans le shell d'un wrapper Bridget déjà enregistré. Le nom donné par `--from`
doit donc être l'identité réellement enregistrée de cet équipier ; un terminal
humain hors wrapper est refusé. Les identifiants ci-dessous proviennent de la
délégation à laquelle la lettre est liée.

Pour rendre un dépôt rejouable, conserver **ensemble et sans les modifier** :

- `--id` (`request_id`) ;
- `--issued-at` ;
- `--issuer-scope` ;
- tous les autres champs de l'enveloppe.

La clé est tripartite : `(issuer_scope, service_request, request_id)`. Un même
triplet avec des octets canoniques différents est refusé ; après la rétention,
`idempotency_expired` est terminal et ne doit pas déclencher un nouveau dépôt
implicite.

Dans le shell du wrapper, définir une fois les valeurs stables du retry :

```bash
export G1504_REQUEST_ID="rapport-livraison-001"
export G1504_ISSUED_AT="$(date +%s)"
export G1504_ISSUER_SCOPE="015_scope_0123456789abcdef0123456789abcdef"
```

Les trois formes fermées sont les suivantes.

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget guichet deposer delivery-report --from "$BRIDGET_AGENT_NAME" --objective "$OBJECTIVE_ID" --delegation "$DELEGATION_ID" --hash "$DELIVERY_SHA256" --in-reply-to "$MAICIE_MESSAGE_ID" --id "$G1504_REQUEST_ID" --issued-at "$G1504_ISSUED_AT" --issuer-scope "$G1504_ISSUER_SCOPE"
```

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget guichet deposer mission-status --from "$BRIDGET_AGENT_NAME" --delegation "$DELEGATION_ID" --id "mission-status-001" --issued-at "$(date +%s)" --issuer-scope "$G1504_ISSUER_SCOPE"
```

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget guichet deposer deadline-question --from "$BRIDGET_AGENT_NAME" --delegation "$DELEGATION_ID" --id "deadline-question-001" --issued-at "$(date +%s)" --issuer-scope "$G1504_ISSUER_SCOPE"
```

`delivery-report` exige en plus la référence de demande `--in-reply-to` et un
hash SHA-256 de 64 caractères hexadécimaux. `mission-status` et
`deadline-question` ne prennent que la délégation : Maicie répond à partir de
son registre, sans interpréter du texte libre ni qualifier un retard.

Une sortie `DÉPÔT: queued` atteste la persistance au guichet ;
`DÉPÔT: outcome_unknown` impose de rejouer exactement la même commande. Après
une réponse terminale, le même dépôt affiche `DÉPÔT: accepted` et retourne un
code non nul : c'est le rejeu d'une issue déjà connue, pas un échec à corriger
par un nouveau `request_id`.

## Relever, greffer et constater

Quand Maicie est absente au dépôt, relancer simplement une commande Maicie :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie status --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json --json
```

Cette invocation ouvre une connexion de service avec la capacité négociée
`maicie_guichet`, exécute `GuichetClaimNext` FIFO avec bail, valide la relation
dans la SQLite Maicie et persiste un reçu avec la décision avant de transmettre
les `reply_bytes` associés. La réponse corrélée fait passer la demande Bridget
liée à `answered` et Bridget dépose son `RequestLifecycleEvent` durable. Un
rejeu par la même génération de claim renvoie les mêmes octets ; après une
relève nouvelle, le token et la génération changent et l'ancien détenteur est
refusé `claim_stale`.

Le gate réel G1504 couvre précisément le parcours suivant : dépôt par un
wrapper ACP réel pendant l'absence de Maicie, relève, greffe unique, réponse
corrélée, demande `answered`, événement relevé et rejeu sans doublon. Il a
réussi en **925 ms** dans le commit `69ad00d` :

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget /Users/moi/.cargo/bin/cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml -p maicie --test guichet_gate_integration -- --ignored parcours_reel_g1504_releve_une_lettre_et_ne_la_duplique_pas --nocapture
```

## Frontières de sécurité et limites

La capacité négociée `maicie_guichet`, et non `from: "maicie"`, autorise les
primitives de relève, lookup et réponse. Sans capacité, un tiers qui connaît
le triplet de dépôt reçoit `capability_required` avant toute opération métier.
Dans le modèle local coopératif v1, cette capacité ne prétend toutefois pas
authentifier cryptographiquement un processus hostile du même compte.

Le contrat refuse explicitement texte libre, opération ou champ inconnu,
enveloppe divergente, cible non réservée, capacité absente, bail périmé et
tentative d'approbation distante. Aucune commande Bridget, MCP ou guichet ne
peut proposer, approuver ou consommer une activation de profil :
`maicie profile approve` reste une frappe humaine locale dans un TTY.

Le guichet ne détient que le transport. Maicie demeure seule autorité des
objectifs, délégations, décisions et reçus. Les observations transport restent
séparées et datées ; un `Gap`, `End` ou `unavailable` ne clôt, ne rouvre et ne
transforme jamais un objectif. Une boucle résidente `maicie serve`, une GUI/TUI
et toute interprétation de texte libre restent des évolutions v2, hors de la
session 015.
