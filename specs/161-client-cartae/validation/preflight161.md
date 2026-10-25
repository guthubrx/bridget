# Préflight SPEC-161 — T001

**Date :** 2026-10-13. **Périmètre :** état initial et préparation documentaire. **Résultat T001 :** vérifié. **Livraison :** non exécutée. Les observations distantes ci-dessous ont été effectuées par ROOT entre08:05 et08:22 CEST ; elles ne sont pas présentées comme de nouvelles mesures de cet agent.

## Sources privées lues et empreintes recalculées

| Preuve | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery161-20261013/initial-local.json | d1cf01365b4378530bda4f88157dfb809d73d992faa1a77a6ef2ed0fdc834b9e |
| /Users/moi/.cache/bridget-delivery161-20261013/cache-inventory.json | 83cac7bc7acf98deaffbe86a89af35085ddddaacd3c9aa3444efc4dc0579b047 |
| /Users/moi/.cache/bridget-delivery161-20261013/cache-supply-candidates.json | 89c47dfc18667c28ab03cf6a56e725c170075a8eda10cdc3092428836cb16fd0 |
| /Users/moi/.cache/bridget-delivery161-20261013/cache-index-inventory.json | 5842c86ecbd9439fbcd96d787145131f14cfa3cbce1d364cc740c8b4cc38300e |

Les quatre JSON sont lisibles et valides. Aucun secret de connexion n'est recopié dans ce reçu.

## État Git et propriété

Checkout durable : /Users/moi/Nextcloud/10.Scripts/64.bridget, main, HEAD1663cb61301e69ccf7294a567f01266be44558e7. initial-local.json daté08:17:31 confirme git status vide sur ce checkout. Le worktree161 existe alors sur session-161-client-cartae, mêmeHEADinitial. Identité relevée parROOT : guthubrx / maintainer@example.invalid, inchangée. Remote autorisé : github, https://github.com/guthubrx/bridget.git.

Source à compiler distincte : e6812e3e7b2d051d630192857cc3f440180d51fa. Le runner161 sera identifié séparément après commit ; aucune identité de runner final n'est encore prouvée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/161-client-cartae/validation/preflight161.md#extrait-1).

## Processus protégés — état initial08:17:31

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/161-client-cartae/validation/preflight161.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/161-client-cartae/validation/preflight161.md#extrait-3).

## Cartae et liaison existante — observations ROOT

- Machine : cartae. /home/moi/.local/bin/bridget pointe vers /home/moi/.local/lib/bridget-communication/096/bridget. La cible et son empreinte binaire finale devront être relevées à nouveau juste avant bascule ; ne pas confondre ce préflight avec cette garde tardive.
- /home/moi/.local/lib/bridget-communication appartient à moi:moi, mode700. /home/moi/.local, /home/moi/.local/bin et /home/moi/.local/lib appartiennent à moi:moi, mode775. Aucun chmod global de ces parents partagés.
- Préfixe /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d absent au relevé ; vérifier de nouveau avant réservation. Espace disponible observé265G, pas une taille réservée.
- Python3.12.3, lsof, rsync, cc et nice présents. Cargo et rustc1.92 présents sous /home/moi/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/.
- Configurations Cargo config/config.toml cherchées parROOT dans les parents /, /home, /home/moi, /home/moi/.local, /home/moi/.local/lib et /home/moi/.local/lib/bridget-communication : absentes. Aucun environnementCARGO/RUST dans la sessionSSH observée. Ne pas déduire de cette liste une inspection d'un autre chemin non relevé ; le runner impose ses bornes explicites.
- Status local et distant RC0 : maître monordinateur, BuildIDe6812e3e7b2d, socketcore existante. Le vieuxclient096 émet un avertissement erroné « daemon périmé » ; aucune relance ne doit en être déduite.
- Tentative status --json refusée pour argument invalide, puis status simple réussi. Conserver cette distinction ; pas d'invention de sortieJSON native.

## Cache et décision d'approvisionnement

Inventaire :113paquetsregistry du verrou160,28archives/sources absentes surCartae. Les28archives existent surMac,4 009 872octets au total ; ROOT a attesté leurs empreintes contreCargo.lock160. Sept entrées d'index n'ont pas les versions requises : data-encoding2.11.1, minreq2.14.1, pulldown-cmark0.13.4, sha10.10.7, sha1_smol1.0.1, sha2-asm0.6.4, tungstenite0.26.2. Une dépendanceWindows absente ne prouve pas un échecLinux.

Copie contrôlée Mac→Cartae autorisée par décisionROOT dans le périmètre de livraison : fichiers déjà présents, SHA verrouillés, aucun téléchargementtiers, aucun changementRust/Cargo.lock. Index sauvegardé avecSHA et comparaison avant remplacement ; divergence étrangère refusée. L'agent de copie travaille encore : aucune copie terminée, aucun index activé et aucune résolution offline réussie ne sont attestés ici. Si préparation impossible, conserver096 et bloquer le build.

## Ressources à suivre

Préfixe161 futur, éventuels suffixesr2, checkoutsource160 futur, stages privés et preuves restent à inventorier lors de leur création. Préserver le client096 pour rollback, la cible161 quand elle sera active, les bases, la liaisonSSH, les caches partagés et les travaux étrangers. Utiliser build.py avec ses gardes pour les caches éligibles ; aucune purge générale. Ce préflight ne prouve ni un build, ni une bascule, ni un push, ni un nettoyage.
