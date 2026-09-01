# Recherche - SPEC-084

## Sources internes examinées

- `crates/bridget-daemon/src/project_policy.rs`: la v1 stocke uniquement `allowed_project_roots` et accepte tout descendant canonique.
- `crates/bridget-daemon/src/project_workspace.rs`: création et import utilisent aujourd'hui la même validation de racine.
- `crates/bridget-daemon/src/control_settings.rs`: le centre expose une seule clé modifiable pour les racines.
- `apps/bridget-desktop/ui/fleet-app.js`: les actions ciblent déjà une source connectée, mais ouvrent directement la source active.
- `apps/bridget-desktop/src-tauri/src/lib.rs`: `panel_open` résout déjà local ou distant sans autorité centrale.
- SPEC-065, SPEC-076, SPEC-080 et SPEC-081-flotte-globale-sources-ui.

## Sources externes primaires

- Git, documentation officielle `git-worktree`: un dépôt peut avoir plusieurs worktrees avec HEAD et index séparés, mais un répertoire Git commun. https://git-scm.com/docs/git-worktree
- Rust standard library, `std::fs::canonicalize`: la canonicalisation résout les liens symboliques et produit un chemin absolu. https://doc.rust-lang.org/std/fs/fn.canonicalize.html

## Conclusions

1. Le problème n'est pas l'absence de multi-serveurs. La flotte et les tunnels existent déjà.
2. Le défaut vient de deux ambiguïtés UI et autorité: la source présélectionnée est cachée et une racine peut signifier projet exact ou parent de création.
3. Un catalogue typé suffit. Une base centrale augmenterait la cohérence distribuée à gérer sans résoudre la validation locale du chemin.
4. La migration la plus sûre ne déduit jamais qu'une ancienne racine est un workspace. Cette intention doit être ajoutée par l'opérateur.
5. Les identités locales peuvent se répéter entre serveurs. Le Desktop doit donc les composer avec la source, sans changer les bases locales.

## Alternatives rejetées

| Alternative | Rejet |
|---|---|
| Utiliser automatiquement le répertoire courant du daemon | Confond source Bridget, stockage runtime et workspace utilisateur. |
| Imposer `/home/moi/projects` | Non portable et contraire à l'administration multi-serveurs. |
| Autoriser un chemin libre dans l'UI | Contourne la politique hôte et augmente le risque de montage arbitraire. |
| Créer une base globale de projets | Duplique les autorités serveur et crée des conflits de synchronisation. |
| Promouvoir toutes les racines v1 en workspace | Élargissement de droits silencieux. |

