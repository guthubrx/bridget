# Correction 285 : guidage du champ version

Défaut confirmé par deux appels KS du 12 novembre à 09:22:18 et 09:22:20 Paris :
`version` vaut la chaîne `"1"`, avec l’action `search`. Le refus générique parle
pourtant de l’action `catalogue`. Aucune commande n’a été transmise au daemon.

- [x] Lire les arguments réels et le parseur.
- [x] Nommer le type du champ version et réserver le conseil catalogue à cette action.
- [x] Qualifier les refus et le chemin valide sur Mac et Linux.
- [x] Publier et activer les services autorisés, vérifier les images et le lien natif.
- [ ] Recharger le guidage des anciennes façades MCP : différé, T3 reste ouvert par décision utilisateur.
- [x] Commit, push et ménage gardé, avec conservations documentées.

Identité Git : absente du dépôt Parix KS et du worktree fe7321af. Le parent a
committé les fichiers, l’arbre est propre. Aucune configuration user.name ou
user.email n’est modifiée, conformément à la règle de François.
