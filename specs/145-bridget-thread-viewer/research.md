# Recherche — SPEC145

Date : 2026-10-07. Constat de planification, pas preuve d'exécution produit.

## Sources locales et constats

L'exploration directe confirme les lectures de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/threads.rs` : list458, show485, history754. `show` expose actuellement `own_acked_seq` et `own_wake`. Sa sortie brute ne convient pas à une surface humaine. `history` utilise `thread_history` sans appel `read` ni ACK.

Le stockage `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` vérifie l'appartenance dans list1179, show1216 et history1385. La liste utilise UUID ASC. L'historique utilise une séquence croissante et une borne snapshot. Les limites présentes sont20/100 pour la liste,50/200 pour l'historique,60 Kio par page et12 Kio de réserve de métadonnées.

Le daemon `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs` refuse actuellement `ThreadRequest` au rôle Client vers11410. La branche agent11638 dépend d'une identité de connexion vivante. Cette voie n'est pas une API humaine réutilisable telle quelle.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/research.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/research.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/research.md#extrait-3).

## Validation externe transmise par le principal

Le principal a effectué trois requêtes de validation primaire frontend, tests et sécurité. Les sources suivantes guident la conception ; cette note ne prétend pas une seconde collecte indépendante.

- W3C, modèle d'onglets accessible : https://www.w3.org/WAI/ARIA/apg/patterns/tabs/ . Utiliser les interactions natives et la sélection accessible des onglets. La connaissance de ce modèle ne remplace pas une preuve de conformité du panneau final.
- web.dev, coût du contenu hors écran : https://web.dev/articles/content-visibility . Ne pas confondre compacité visuelle et limitation de charge ; la pagination bornée doit contrôler la quantité de données et de DOM. Ne pas ajouter cette propriété sans besoin observé.
- Testing Library, choix des requêtes : https://testing-library.com/docs/queries/about/ . Tester les commandes visibles et leurs noms accessibles, pas seulement les détails internes du composant.
- Playwright, pratiques de test : https://playwright.dev/docs/best-practices . Isoler l'environnement et vérifier les comportements observables. Un navigateur n'est pas autorisé dans la phase actuelle ; ces recommandations ne prouvent aucune recette.
- OWASP, modèles d'autorisation : https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Patterns_Cheat_Sheet.html . Vérifier l'autorisation à chaque lecture et refuser par défaut.
- PortSwigger, contrôle d'accès : https://portswigger.net/web-security/access-control . Couvrir les identifiants forgés, la portée projet et l'accès direct aux objets.

## Décisions

I004 : lecture source145 pendant implémentation, sans edit code par l'agent documentaire. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` ligne624 restitue l'objet notify historique ; ligne1035 stocke mode/targets. NotifySpec::mode fournit none/targets/all. Les dates ThreadRow84–85 et entry_json628 sont i64 Unix secondes, confirmées par unix_now_secs/as_secs dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`. Le principal approuve l'alignement Rust/TypeScript sur ces sorties réelles. Aucun changement de besoin ni nouveau type Post.

Réutiliser les lectures et règles de stockage, avec une projection humaine distincte. Ne pas exposer les curseurs ni réouvrir toutes les commandes agent au rôle Client. Ajouter une capacité versionnée fermée et un fait primaire de liaison en mémoire. Aucun changement de schéma DB, nouvelle dépendance ou API générale n'est requis.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/research.md#extrait-4).
