# Livraison du transfert différentiel — 22 octobre 2026

## Résultat actif

Source du binaire : b8406f6c2d2c6abf5e09493a81d1fabaa8104fb1. Build actif du maître : 172-b8406f6c, PID 64762, confirmé par bridget_status. Version du paquet : 0.4.1.

SHA-256 du binaire signé : e5c325591d447ce8e71e41a1f4a52bcc417e7ae5b22b8b8588013055b076af66.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/bundles-divergents-20261022.md#extrait-1).

## Défaut et correction

Lors d'une divergence, le HEAD distant n'est pas ancêtre de la base source. L'ancienne préparation abandonnait l'exclusion et pouvait exporter tout l'historique. La correction calcule une base commune, fait confirmer sa présence par l'inspection distante existante, puis exporte seulement la partie postérieure. Le distant vérifie le bundle avant import. Les droits, hooks, mappings et HEAD existants restent protégés.

Références Git : https://git-scm.com/docs/git-bundle et https://git-scm.com/docs/git-merge-base. Les prérequis d'un bundle sont vérifiés par Git avant son import.

## Preuves

- 24 tests Python ciblés PASS sur Cartae, source finale vérifiée par empreintes dans le worktree auteur et le worktree de contre-revue. Pas de suite complète.
- RED initial : 3 nouveaux tests discriminants échouent avant correction (2 FAIL et 1 ERROR).
- RED supplémentaire : le vrai wrapper Git convertissait le délai en refus nettoyable. Le test injecté dans subprocess.run prouve que le reçu disparaissait. Le garde final préserve ready=false ; le rejeu reste fermé.
- Dépôts de test réels : branches divergentes, historique commun de 2 Mio, bundle inférieur à 32 Kio, import réel et HEAD distant inchangé. Nouveau projet et contrôles de chemins/reçus/hooks toujours couverts.
- Les sept codes de délai sont testés à l'égalité exacte : git, bundle_base, bundle_export, bundle_verify, bundle_import, ssh_inspect et ssh_prepare. Aucun argument de commande privé n'est transmis.
- Revue r1 daab8723-2447-4de9-8c58-1d818a91a81d : APPROVE. Contre-revue finale 494dec67-0d21-4676-94cf-3e2c65937306 : APPROVE. Résultats complets lus, acquittés et tâches clôturées. Les relecteurs ont lu le code ; ils n'ont pas exécuté les tests.
- Compilation macOS finale : 68,23 s, RC 0, signature vérifiée. Aucun fichier Rust de production modifié.

## Qualification réelle Notre-Dame sans nouvelle mission

Cible : fd4cad3b5e3be49bc56f866ece6c695ed1b2d3c5. Base commune confirmée sur Cartae : e89b8430c17fc7f5ba2917346a7bbf36b43b3446. Le delta contient 43 commits sur les 6 914 commits de l'historique cible.

Export mesuré : 7 610 389 octets en 0,504 s. SHA-256 : ae5fde76384de3592fd74081670fef4b5dba8d75df1ad183eef0baae693468e7. Ce même paquet a été copié dans un dossier privé Cartae ; git bundle verify y réussit en 0,018 s et HEAD reste inchangé. L'export a été mesuré sur 10eaa0e7 ; la fonction export_bundle est byte-identique dans la source finale.

Aucun import dans Opus2D, lancement de modèle ou rejeu de monuments-notre-dame-couverture-projection-20261022-01 n'a été effectué par cette qualification. Ce test qualifie le paquet et ses prérequis réels ; il ne qualifie pas la remise d'une mission au fournisseur.

## Limites et reprise

Le timeout Notre-Dame initial n'a pas de phase conservée. Sa cause n'est pas rétrospectivement attribuée à l'export. Les futurs délais portent un code de phase fermé. L'échéance SSH peut expirer avant la réponse du helper distant ; ssh_prepare_timeout désigne alors l'attente, pas une preuve de panne réseau.

Si le HEAD distant est inconnu de la source ou si aucune base commune ne peut être calculée, l'export complet préexistant reste possible. Le gain mesuré ne s'étend pas à tous les dépôts.

Un timeout de création garde son reçu incomplet. Avant toute remise en état ciblée, vérifier la fin du processus et de ses descendants, l'absence d'écriture tardive, le dossier cible et l'enregistrement git worktree list. Toute incertitude conserve le reçu. Cette livraison ne supprime aucun reçu de préparation bloquée.

La première revue et la contre-revue ne remplacent pas une recette métier. Les limites non bloquantes de couverture et le fallback générique préexistant sont conservés hors du déblocage mesuré.

## Information et nettoyage

Résultat envoyé à monuments, demande d'origine mcp-85809-6ad9f598-3, message bridget172-bundle-differentiel-actif-monuments-20261022-r1. Reçu de transport in_flight : cela ne prouve pas sa lecture. Aucun rejeu métier demandé dans cet envoi.

Le snapshot Mac de compilation, le premier binaire jamais activé et le dépôt d'export temporaire ont été retirés après un inventaire lsof complet sans fichier ouvert. Le binaire final, le rollback 172-76b682d6, les preuves et le cache Cargo utile sont conservés. Les worktrees Cartae du lot sont conservés ; l'inventaire global Linux reste incomplet et aucune suppression forcée n'a été faite.

Preuves privées : /Users/moi/.cache/bridget-bundle172 et /home/moi/.cache/bridget-bundle172.
