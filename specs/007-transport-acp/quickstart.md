# Quickstart : valider le transport ACP à la main

Scénario de validation de bout en bout pour les développeurs. Prérequis : daemon
compilé et lancé, CLI Codex loggé (abonnement), `npx` disponible.

## 0. Spike préalable (avant toute implémentation)

Valider l'adaptateur seul, hors Bridget :

```bash
npx @zed-industries/codex-acp@0.16.0
# puis coller sur stdin, ligne par ligne :
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}
```

Attendu : une réponse `initialize` avec les capacités de l'agent, sans demande
de clé API. Si ce spike échoue, s'arrêter et documenter — le reste de la
session repose dessus.

## 1. Lancer un équipier Codex

```bash
bridget codex --equipier          # nom de flag définitif fixé en tasks
bridget who                       # l'équipier apparaît, transport « acp »
```

Vérifier : aucun bloc « Règles ABSOLUES » n'existe nulle part (l'équipier n'a
reçu aucun prompt d'endoctrinement).

## 2. Échange complet avec demande suivie

Depuis un autre terminal :

```bash
bridget send --to codex-1 --reply "Explique en une phrase le rôle du fichier crates/bridget-core/src/router.rs"
bridget ledger                    # la demande apparaît, puis se clôt à la réponse
```

Attendu : la réponse arrive à l'émetteur ; la demande est close sans qu'aucune
commande n'ait été exécutée par l'équipier.

## 3. Intégrité du corps de message

```bash
bridget send --to codex-1 --reply "Répète exactement, entre balises <echo> : l'apostrophe d'usage, \"guillemets\", \$VAR, \`backticks\`,
et ce saut de ligne."
```

Attendu : le contenu revient intact dans la réponse (SC-002).

## 4. File d'attente pendant un tour

Envoyer une tâche longue puis, immédiatement, un second message. Attendu : le
second est livré après la fin du premier tour, dans l'ordre, sans perte ; le
ledger ne montre aucune relance pendant le tour en cours (SC-004).

## 5. Garde de facturation

```bash
OPENAI_API_KEY=test bridget codex --equipier
```

Attendu : refus immédiat, message en français nommant la variable et le
contournement `BRIDGET_ALLOW_API_KEY=1` (SC-005).

## 6. Non-régression tmux et fédération

- Lancer un agent en mode actuel (sans flag équipier) dans tmux : le
  comportement `💬` est inchangé (SC-006).
- Si une machine fédérée est disponible : dérouler l'étape 2 vers un équipier
  distant (FR-014).

## 7. Journal

```bash
cat ~/.cache/bridget/sessions/codex-1/*.jsonl | tail -20
```

Attendu : `turn_start` / `update` / `turn_end` horodatés pour les échanges
ci-dessus.
