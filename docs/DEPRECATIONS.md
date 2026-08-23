# Registre des dépréciations

Ce registre recense les chemins maintenus temporairement pendant une migration.
Une entrée est supprimée avec son chemin dès que la condition indiquée est
atteinte.

| Chemin | Remplacé par | Supprimable quand |
|---|---|---|
| Liste blanche de binaires dans `crates/bridget-daemon/src/wrapper.rs` | Registre `~/.config/bridget/agents.json` | Supprimée en T703 |
| Affichage stderr sans transport dans `crates/bridget-daemon/src/wrapper.rs` | Livraison structurée ACP ou log applicatif | Supprimé en T705 |
| Micro-banc SC-005 ignoré dans `crates/bridget-transport/src/acp.rs` | Banc à deux vues attach réelles dans `crates/bridget-daemon/tests/sc005_attach_budget.rs` | Après suppression de la couverture historique ACP au prochain cycle de nettoyage des tests |

Relecture T711 : ces deux chemins hérités sont les seuls retirés par la
migration ACP ; ils restent étiquetés ici jusqu'à la suppression de leur
référence historique.

Relecture T911 : la gestion du cycle de vie par le daemon est additive. Le
lancement historique par wrapper-terminal reste supporté et aucun nouveau
chemin n'est déprécié par la session 009.

Relecture T1305 : le rendu compact de `attach` est sélectionné uniquement sur
stdout TTY. Le rendu historique non-TTY reste le chemin de compatibilité et
n'est pas déprécié ; aucune nouvelle entrée n'est donc ajoutée au registre.
