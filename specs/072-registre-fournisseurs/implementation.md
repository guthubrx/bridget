# Implémentation et preuves - SPEC-072

## Résultat

Le registre reconnaît maintenant explicitement `codex`, `cursor`,
`anthropic`, `glm` et `deepseek`. Le type choisi est conservé jusqu'au
transport et au journal. Ainsi, Claude Code reste le programme de transport
pour Anthropic, GLM et DeepSeek, sans jamais faire passer GLM ou DeepSeek pour
Anthropic.

## Contrat ajouté

- `AgentDefinition` et `ResolvedAgentDefinition` portent `claude_config_dir`.
- Le chemin doit être absolu, appartenir à l'utilisateur courant, être un
  répertoire réel non lié symboliquement et avoir le mode 0700.
- Il est réservé à `claude_stream_json`. Une valeur ambiante
  `CLAUDE_CONFIG_DIR` ne peut pas le remplacer.
- Le chemin non secret participe au digest de définition figée. Les secrets et
  endpoints restent seulement dans les `settings.json` privés hors Git.
- `ClaudeStreamJsonTransport` reçoit le type déclaré et publie donc le
  `provider_kind` exact dans les contextes et bindings.

## Migration de reprise

Le premier démarrage de la version a révélé une incompatibilité de digest : des
définitions figées existantes avaient les capacités L1 mais avaient été écrites
avant `claude_config_dir`. Le nouveau binaire les refusait au redémarrage.

Le service a été immédiatement restauré avec le binaire précédent. La migration
accepte maintenant cette seule forme historique si et seulement si le profil est
absent, tout en conservant les capacités persistées. Les anciennes définitions
plus anciennes, sans capacités, gardent leur migration distincte. Une vérification
hors écriture des 689 définitions réellement persistées a trouvé 0 digest non
reconnu. Le redémarrage suivant a été stable.

## Profils privés du serveur

- Registre : `/home/moi/.config/bridget/agents.json`, mode 0600.
- GLM : `/home/moi/.config/bridget/claude-profiles/glm`, mode 0700, avec un
  `settings.json` mode 0600.
- DeepSeek : `/home/moi/.config/bridget/claude-profiles/deepseek`, mode 0700,
  avec un `settings.json` mode 0600.
- Le profil Anthropic ne reçoit aucun de ces répertoires statiques et interdit
  explicitement les variables d'endpoint compatibles avant son lancement.

Les contenus de ces profils ne sont ni affichés ici ni versionnés.

## Preuves vivantes

| Fournisseur | Preuve | Résultat |
|---|---|---|
| Cursor | Agent géré `cursor` sur ACP, demande sans écriture | Réponse `CURSOR-OK`, tour terminé. |
| GLM | Agent `glm` sur `claude_stream_json`, lecture de `AGENT_HANDOFF.md` | Outil `Read` observé et réponse `GLM-OK`. |
| DeepSeek | Agent `deepseek` sur `claude_stream_json`, demande sans écriture | Refus fournisseur explicite : `API Error: 402 Insufficient Balance`. Aucun repli. |
| Anthropic + GLM | Deux agents démarrés simultanément avec types distincts | GLM répond `GLM-ISOLATION-OK`; Anthropic remonte son quota hebdomadaire réel. Les deux transportent `claude_stream_json` mais gardent leurs identités distinctes. |

Les tests ciblés confirment aussi que l'interruption d'un tour Claude déclenche
la remise suivante sans tuer l'agent et qu'un terminal `api_error` refuse la
livraison de façon explicite.

## Identité visuelle intégrée

La branche est partie de `origin/main` au commit
`04fd9c656ac59c06b2d52e7ee08e233a3ce98005`, qui contient déjà les overlays
et les assets de la SPEC-071. Aucun de ces styles ou comportements n'a été
remplacé.

- `anthropic` est rendu comme Claude Code / Anthropic.
- `glm` est rendu comme GLM / Z.AI avec `/providers/glm.svg`.
- `deepseek` est rendu comme DeepSeek avec `/providers/deepseek.svg`.
- Les deux SVG sont embarqués dans le daemon et servis avec le même ETag et la
  même revalidation que les logos existants.

Les sources et empreintes de chaque SVG sont consignées dans
`crates/bridget-daemon/assets/ui/providers/NOTICE.md`.

## Validation de la reprise

- Les 81 tests JavaScript de l'interface passent.
- Le test HTTP des six logos embarqués passe.
- `cargo check --workspace`, `cargo fmt --all -- --check` et la construction
  release passent.
- Les scénarios ciblés du registre, de l'isolation GLM/Anthropic, de Cursor et
  de la reprise des définitions figées restent validés.
- Trois tests de `managed_parity_test` échouent également à l'identique sur une
  archive propre de `origin/main` :
  `matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`,
  `reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees` et
  `reprise_codex_sans_amorcage_ne_decouvre_pas_mcp`. Ils préexistent à cette
  reprise et ne sont pas modifiés ici.

## Construction prête, non déployée

Le binaire construit depuis la tête commune et les changements de cette SPEC
est `/home/moi/bridget-referent/.worktrees/session-072-registre-fournisseurs/target/release/bridget`.
Son empreinte SHA-256 est
`a083003bc17a0319cfc9df9a592d6f863862b9784a839b486fd2379589c4bb42`.

Cette reprise n'installe pas ce binaire et ne redémarre aucun service. Une
livraison ultérieure devra d'abord intégrer ces changements dans la tête
partagée la plus récente, puis construire et déployer depuis cette seule tête.

## Garde d'intégration avant livraison

Un binaire de production ne doit jamais être construit directement depuis un
worktree de spécification isolé. Avant chaque livraison, les commits de toutes
les livraisons concurrentes doivent être intégrés dans une tête commune, par
merge ou cherry-pick, puis cette tête doit être vérifiée avant la construction.
Le binaire est ensuite construit et déployé uniquement depuis cette branche
d'intégration. Cette règle évite de remplacer des ajouts déjà livrés par un
worktree qui ne les contient pas encore.
