# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/archive-r4-recettes/native-real-recipes-sonnet-r4.md](../../../../../../../../specs/149-sous-agents-lineage/validation/archive-r4-recettes/native-real-recipes-sonnet-r4.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Receipt `native149-debug-receipt.json` r4 vérifié : empreinte source complète `e6f45086…5408` (196 fichiers) et empreinte production `e0d86f01…f3a7f215` (111 fichiers) recalculées, identiques. `wrapper.rs` `b599f8bb…b455e65`, `native_delegation.rs` `fc62f33d…72dc0d9`.
- Binaire : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-823e8a5fab8a`, SHA256 `823e8a5fab8abe38fcb439d8fda98d929622dae3751ff8f629d306744379328b` (debug, 53 993 512 octets).
- Lanceur `gclaude` : `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e` (constante contrat). CLI claude 2.1.296 : `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937` (240 664 432 octets).
- Fixture privée 0700 : `/Users/moi/.cache/bridget149-recipe-r2.r4` (home, registre, logs, état). Aucune lecture/écriture de `/Users/moi/.cache/bridget-core` en écriture ; registre prod lu seulement pour le clone. Aucun vecteur T3 dans l'environnement.
- Daemons fixture : PID 42877, 33457, 15425 (arrêtés, SIGTERM un par un). Aucun processus résiduel du fixture à la fin.
- Registre fixture : `glm` = `glm-5.3-flash` exact (aucun repli) ; `codex` = clone de `codex-pro` (protocole `codex_app_server`).
