# Quickstart : valider les équipiers daemon-gérés

Prérequis : 007 livrée (équipiers, registre), 008 livrée (attach), daemon lancé.

## 1. Spawn et survie au terminal

```bash
bridget spawn codex          # retourne le nom (ex. codex-1) une fois connected
bridget who                  # codex-1 visible, marqueur « géré »
exit                         # fermer CE terminal
# depuis un AUTRE terminal :
bridget send --to codex-1 --reply "Réponds exactement : SPAWN_OK"
bridget requests             # demande close, réponse reçue
```

## 2. Stop universel et propre

```bash
bridget stop codex-1         # StopOutcome=Stopped, who → stopped
pgrep -g <pgid>              # aucun processus du groupe (descendants compris)
bridget stop <agent-tmux>    # refus NotManaged — jamais tuer un wrapper 007
```

## 3. Échecs motivés (table SC-003 — 11 familles)

La couverture **exhaustive** des 11 familles est automatisée (matrice de tests
SC-003) ; ce quickstart en exerce manuellement un échantillon : type inconnu,
commande absente, `OPENAI_API_KEY` posée (garde), nom déjà actif, cwd supprimé
avant spawn, quota atteint, spawn pendant `Recovering`. Les quatre restantes —
`EnvUnfit` (HOME/auth/PATH), `NegotiationFailed`, `SpawnTimeout` avec
`Register` tardif rejeté, `IdempotencyExpired` — sont vérifiées par la matrice
automatisée. Observable commun : motif typé au client + aucun nouvel état
opérationnel imputable au refus.

## 4. Persistance et réconciliation

```bash
bridget spawn codex --persistent   # + un éphémère
# arrêt coopératif du daemon puis relance :
bridget who                        # le persistant revient (même nom), l'éphémère non
# SIGKILL du daemon puis relance :
ps                                 # aucun ancien groupe ; une seule nouvelle instance
bridget stop <persistant> ; relance daemon → il ne revient pas
```

## 5. Attach sur daemon-géré

`bridget attach <nom>` : comportement identique au mode wrapper-terminal
(comparaison de frames, matrice FR-008) ; pendant un `stop`, la vue reçoit
`End` typé. Précision : après un `stop`, « reprise » = un **nouveau
`SpawnOrder`** du même nom (nouvelle génération — le `stop` a durablement
retiré l'entrée de `fleet.json`) ; la vue se réabonne alors avec un nouveau
`subscription_id`.

## 6. Parité FR-008

Banc : corpus commun (quickstart 007 §1-§5 automatisés) exécuté dans les deux
modes, observables comparés, N et tolérances de la matrice versionnée.
