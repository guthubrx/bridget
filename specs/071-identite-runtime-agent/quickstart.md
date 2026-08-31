# Quickstart de validation - SPEC-071

## Préconditions

- Worktree : `/home/moi/bridget-referent/.worktrees/session-071-identite-runtime-agent`
- Node et Cargo disponibles sur `cartae.app`.
- Aucun redémarrage du relais UI nécessaire pour les tests statiques.

## Tests rapides

```bash
cd /home/moi/bridget-referent/.worktrees/session-071-identite-runtime-agent
node crates/bridget-daemon/assets/ui/app.js
cargo test -p bridget-daemon ui::
cargo test -p bridget-daemon --test ui_relay_test
```

## Vérification des SVG

```bash
cd /home/moi/bridget-referent/.worktrees/session-071-identite-runtime-agent
rg -ni '<script|on[a-z-]+[[:space:]]*=|(?:href|xlink:href)[[:space:]]*=|data:(?:image|text|application)' crates/bridget-daemon/assets/ui/providers/*.svg
```

Résultat attendu : aucune occurrence active. Les déclarations d'espace de noms
XML restent autorisées car elles ne chargent aucune ressource.

## Parcours manuel après compilation explicite

1. Ouvrir la fiche d'un agent Codex, Claude Code, Cursor et Gemini CLI.
2. Vérifier le produit, l'éditeur, le mode, le modèle et le transport attendus.
3. Vérifier un type inconnu et une présence sans mode.
4. Passer le pointeur de la ligne à la fiche, puis revenir.
5. Ouvrir au clavier et fermer avec Échap sans changer la sélection.
6. Tester la première et la dernière ligne à 1280x720 puis à 200 % de zoom.
7. Couper le réseau public et rouvrir les fiches : les logos restent visibles.

## Critère de sortie

La feature est acceptable lorsque toutes les suites passent et que le parcours
manuel ne révèle ni coupure, ni identité inventée, ni gêne de navigation.
