# Plan - SPEC-070

## Décision de conception

Etendre les deux composants existants : le transport Codex pour séparer
l'échéance fournisseur du suivi de réponse, puis la projection du journal dans
`app.js` pour rendre le travail actif visible. Les notifications restent un
adaptateur de navigateur local. Aucun service, endpoint, stockage ou dépendance
n'est créé.

## Architecture cible

```text
événement fournisseur réel
        |
        v
journal EventSource existant
        |
        v
projection de tour active dans app.js
        |                         \
        v                          v
bouille + ligne atténuée      réponse ou erreur terminale
                                   |
                                   v
                           notification opt-in + cible message
```

## Lots réversibles

1. Séparer les deux délais dans `crates/bridget-daemon/src/daemon.rs` et
   protéger le terminal transport dans
   `crates/bridget-transport/src/codex_app_server.rs`.
2. Dériver le dernier acte réel d'un tour non terminal dans
   `crates/bridget-daemon/assets/ui/app.js`, puis l'afficher avec l'avatar
   existant au-dessus du compositeur via `index.html` et `theme.css`.
3. Relier un échec terminal à son message, afficher un détail sûr et permettre
   de cibler un message du fil sans modifier le défilement hors du bas.
4. Ajouter une permission utilisateur et une notification navigateur locale
   pour les seuls terminaux réels.
5. Couvrir les invariants par les tests Node existants et les tests Rust du
   daemon et du transport.

## Fichiers touchés

| Fichier | Rôle |
|---|---|
| `crates/bridget-daemon/src/daemon.rs` | ne plus transformer `reply_timeout` en échéance transport courte |
| `crates/bridget-transport/src/codex_app_server.rs` | interrompre et terminer proprement à la véritable échéance fournisseur |
| `crates/bridget-daemon/assets/ui/index.html` | conteneur d'activité et contrôle de permission |
| `crates/bridget-daemon/assets/ui/theme.css` | présence compacte et lisible sans animation décorative |
| `crates/bridget-daemon/assets/ui/app.js` | projection active, erreur corrélée, cible de message, notification et tests Node |

## Garde-fous

- `prompt_dispatched` ne suffit jamais à afficher que l'agent travaille.
- Le délai court reste réservé à une intention de contrôle explicite.
- Les événements existants restent la source de vérité ; aucun état serveur
  redondant n'est introduit.
- La notification est déclenchée uniquement après une réponse ou un échec
  terminal, uniquement avec permission et uniquement hors premier plan.
- Le clic natif est meilleur effort ; la cible interne est toujours disponible.
- La projection reste O(n) sur le journal du fil et n'ajoute pas de parcours
  imbriqué.

## Validation

- Faux Codex : tour avec réponse attendue actif au-delà de 60 secondes et
  terminal normal avant l'échéance fournisseur.
- Faux Codex : vraie échéance fournisseur entraîne un `turn/interrupt` borné
  et un seul terminal.
- UI Node : aucun état actif sans événement fournisseur, dernier acte affiché
  durant le tour, fin et erreur retirent l'état actif.
- UI Node : notification refusée ou inactive ne produit rien ; notification
  autorisée vise le bon agent et le bon message.
- UI Node : cible de message ne déplace pas un lecteur hors du bas avant son
  clic volontaire.
