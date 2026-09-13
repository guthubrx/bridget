# Recette 092

Utiliser le binaire candidat reconstruit, pas supposer le binaire installé à jour.
Depuis le worktree /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux :

```sh
export PATH="/Users/moi/.cargo/bin:$PATH"
cargo test -p bridget-daemon attach::tests::spec092
cargo test -p bridget-daemon attach::tests
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
# Une seule consolidation finale, pas après chaque modification :
cargo test --workspace
```

Les tests créent leur HOME/socket/TMPDIR isolés. Si les droits de l'équipier
empêchent un test, le pilote l'exécute sans élargir ses droits.

Recette humaine (ne pas la cocher depuis un test d'octets simulés) :
1. Ouvrir attach sur l'UUID attesté de l'équipier ; saisir `première`, Shift+Entrée,
   `seconde`. Vérifier deux lignes sur fond gris et aucune remise avant Entrée.
2. Envoyer une fois ; envoyer ensuite `autre` ; saisir `brouillon`, Haut deux fois,
   Bas deux fois : le brouillon revient. Modifier un rappel puis vérifier l'original.
3. Redimensionner ; statut toujours sous la saisie. Ctrl-C laisse l'agent vivant
   et rend au shell son clavier normal. Réouvrir : historique vide.
4. Si le terminal ne distingue pas Shift+Entrée, essayer Alt+Entrée. La limite
   doit être annoncée, sans modifier les réglages globaux du terminal.

Modèle en direct (déjà livré en 091, pas à réimplémenter ici) : dans attach vers
un Codex géré, `/model gpt-5.6-sol high` est un contrôle explicite. Attendre le reçu
et l'état attesté ; pas de relance ni de nouvelle permission. Une session Codex
interactive humaine utilise son propre `/model` dans la TUI fournisseur.
