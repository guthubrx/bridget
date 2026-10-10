# Revue de durcissement T3 149 — ronde 3 (C01, C02, F1, F2)

Date : 2026-10-10. Sous-agent de revue indépendant (GLM 5.3 Flash). Ronde ciblée.
Périmètre : les quatre findings seulement, et leurs impacts directs. Le reste
des axes reste couvert par r1 (`t3-permissions-code-review-r1.md`, APPROVE) et
r2 (`t3-lineage-code-review-r2.md`, APPROVE). Aucune re-exploration générale.
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

Verdict : **APPROVE**.

Les quatre findings sont fermés. Chaque correction tient l'oracle d'origine.
Aucun chemin positif antérieur ne régresse. Aucun bug nouveau trouvé.

---

## Méthode et limites de mission

- Lecture seule. Lectures ciblées des sections corrigées, des appelants amont
  et des sources d'oracle. Aucun build, lint, typecheck, test, Git, cochage,
  config, restart. Aucun modèle réel.
- Une seule écriture : ce rapport.
- Les tests indépendants sont en cours chez d'autres agents. Je ne les lance
  pas et ne les exige pas ici.

---

## C01 — Fermé. Allowlist fermée de `claudeSanitarySettings`

Preuve : `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts`

- Ligne 540 : allowlist fermée de six clés exactement : `permissions`,
  `alwaysThinkingEnabled`, `fastMode`, `ultracode`, `autoCompactWindow`,
  `showThinkingSummaries`. C'est la réponse annoncée par Sol, à la clé près.
- Ligne 541 : toute autre clé de premier niveau lève
  `permission_source_unavailable`. La denylist silencieuse a disparu.
  `allowedMcpServers`, `enableAllProjectMcpServers` et `processWrapper`
  (oracle de déclenchement r1) sont désormais refusés.
- Lignes 855-856 et 891-892 : les deux seuls appels capturent le `throw` et
  passent `permissionsRepresented = false`. Ligne 902 : aucun fait publié.
  Le refus est nommé, conservateur, sans repli v1.
- Lignes 544-545 : allowlist de second niveau sur `permissions` :
  `allow, ask, deny, defaultMode, additionalDirectories,
  disableBypassPermissionsMode`. C'est exactement l'objet
  `settings_overrides.permissions` du contrat `permissions.md` (l.152-159).
  Une sous-clé inconnue est refusée.

Non-régression du chemin positif, preuve amont :

- `apps/server/src/claudeModelOptions.ts:45-49` : les settings compilés ne
  produisent que `alwaysThinkingEnabled`, `fastMode`, `ultracode`.
- `ClaudeAdapterV2.ts:1146` ajoute `autoCompactWindow`; ligne 1178 ajoute
  `showThinkingSummaries`. Les cinq clés sont dans l'allowlist.
- `apps/server/src/provider/Drivers/ClaudeMcp.ts:49-54` :
  `claudeBridgetReadOnlySettings` ne retourne que `{ permissions }`.
  La propagation de sous-clés inconnues (spread ligne 51) est revalidée par
  la ligne 545. Une sous-clé inconnue ferme le fait. Direction sûre.
- `sdkSettings` n'est jamais fourni : unique appel du builder à
  `ClaudeAdapterV2.ts:7650`, aucune affectation `sdkSettings:` dans le dépôt
  serveur. Aucune clé settings hors allowlist n'est atteignable aujourd'hui.
- Une settings chaîne non JSON (chemin de fichier) retourne `undefined`
  (ligne 538). Ce fichier reste couvert par `permission_sources` revisionné
  (`captureClaudeLaunchContext`, lignes 517-519). Cohérent avec le contrat
  (l.200-204 : source fichier héritée par l'input complet vérifié).

Conclusion : C01 fermé. L'attestation ne peut plus omettre un élargissement.

---

## C02 — Fermé. Les quatre champs de médiation exclus

Preuve : `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:851-854`

- Ligne 852 : `permissionPromptToolName === undefined`.
- Ligne 853 : `permissionPrompts === undefined` et
  `spawnClaudeCodeProcess === undefined`.
- Ligne 854 : `hooks === undefined`.
- Les conditions d'origine restent : `canUseTool !== undefined`,
  `sandbox === undefined`, `managedSettings === undefined`,
  `claudePermissionFlagsRepresented` (l.851, 854). Aucune condition retirée.
- Noms de champs vérifiés dans le SDK réel `@anthropic-ai/claude-agent-sdk@
  0.3.276`, `sdk.d.ts` : `hooks` l.1658/4221, `permissionPromptToolName`
  l.1915, `permissionPrompts` l.1924, `spawnClaudeCodeProcess` l.2342.

Non-régression : un balayage de `apps/server/src` montre que seules les
gardes référencent ces quatre champs. Aucun chemin T3 actif ne les pose.
`makeClaudeQueryOptions` (l.1148-1201) ne les définit pas. La garde ne peut
donc faire perdre un fait qu'à un chemin futur qui changerait la médiation —
exactement l'objectif du finding. Direction sûre.

Conclusion : C02 fermé. Un changement de médiation invalide le fait.

---

## F1 — Fermé. Wire 24 KiB, contenu 16 KiB, sur read et follow

Preuve : `apps/server/src/bridget/BridgetReader.ts`

- Lignes 419-420 : `journalContentBytes = 16 * 1024` (contenu) et
  `journalEnvelopeBytes = 24 * 1024` (transport). Deux bornes séparées,
  commentées l.417-418.
- Lignes 428-429 : la borne de 16 KiB porte sur la somme UTF-8 des événements
  JSON sérialisés : `Buffer.byteLength(JSON.stringify(event), "utf8")`.
  Dépassement → `output_limit`. La borne du contrat `lineage.md` l.137
  (« au plus 100 événements et 16 KiB de contenu ») est appliquée sur le
  contenu, pas sur l'encodage NDJSON.
- Mesure alignée sur l'oracle natif, vérifié à la source :
  `crates/bridget-transport/src/journal.rs:698` :
  `let bytes = serde_json::to_vec(&entry)?;` — `entry.bytes.len()` est bien
  l'entrée JSON sérialisée complète, pas le seul texte de message. L'événement
  T3 (`packages/contracts/src/bridgetLineage.ts:73-77`) est l'objet structuré
  complet ; le reader re-sérialise ce même objet. Les deux mesures désignent
  la même chose.
- Wire 24 KiB sur les deux voies : lecture non-follow l.440
  (`runLineage(args, journalEnvelopeBytes)` pour l'action journal), follow
  l.513 (`maxLine = journalEnvelopeBytes`, tampon l.467, refus au-delà
  l.478). `validateJournal` s'applique aux deux voies : l.451 et l.515.
- Marge d'enveloppe suffisante : 8 KiB pour 100 événements. Les clés
  d'enveloppe du journal (`version`, `status`, `task_id`, `next_seq`,
  `caught_up`, `gap`) pèsent quelques centaines d'octets. Une page conforme
  au contrat ne peut plus être refusée par le transport. Une page
  non conforme est refusée par la ligne 429. Les deux directions sont sûres.
- Branches voisines intactes : show toujours 24 KiB pour une fenêtre 16 KiB
  (l.440) avec recontrôle d'octets l.443-447 ; cancel 4096 (l.455) ;
  list 128 KiB (l.399). Aucun changement hors journal.

Limite notée : à la frontière exacte (contenu très proche de 16384), une
divergence de re-sérialisation entre `serde_json` et `JSON.stringify` est
théoriquement possible. Son seul effet est un refus de disponibilité sur une
page à la limite exacte. Direction sûre, non bloquant.

Conclusion : F1 fermé. Les pages maximales passent le transport ; la borne
contractuelle reste appliquée sur le contenu.

---

## F2 — Fermé. Retrait de disponibilité des tâches disparues du snapshot

Preuve : `apps/server/src/orchestration-v2/Orchestrator.ts:10245-10251`

- Position : la boucle de retrait vient APRÈS la validation complète du
  snapshot — déduplication et atomicité (l.10178-10180), ownership parent et
  ancestré profondeur ≤ 8 (l.10181-10195), garde anti cross-root par tâche
  (l.10207-10208). Un snapshot incomplet (`next_cursor` non nul) est refusé
  avant toute émission (l.10179-10180). Le commentaire l.10245-10246
  documente l'invariant.
- La boucle est vivante, pas morte :
  `getBridgetTaskThreads(root.id, ids)` retourne tous les fils du root,
  pas seulement les ids du snapshot. Preuve
  `apps/server/src/orchestration-v2/ProjectionStore.ts:4259-4263` :
  `WHERE bridgetTaskRef.rootThreadId = rootThreadId OR thread_id IN ids`.
  Une tâche retirée garde son marqueur, donc elle reste retournée par la
  clause du root. L'implémentation replay reflète la même sémantique
  (`ProjectionStore.ts:6046-6050`).
- Garde par fil, l.10249 : ignorer sans marqueur, ignorer un `rootThreadId`
  d'un autre root, ignorer les tâches présentes (`tasks.has`), ignorer les
  fils déjà retirés. Les autres racines ne sont jamais touchées.
- Ligne 10250 : le payload étend le fil précédent et ne remplace que
  `bridgetLineage.available = false`. Le marqueur `bridgetTaskRef`, le corps
  `bridgetTask` et l'historique restent. L'UI garde le dernier état vérifié,
  plus présenté comme courant. C'est l'option (a) du fix proposé en r2.
- Aucune mutation sur panne ou incomplet : la voie de panne
  (`bridget.lineage.unavailable`, l.10255-10269) est inchangée ; les events
  s'accumulent dans le `Ref` et ne s'appliquent que sur le succès, avec la
  sémantique transactionnelle déjà approuvée en r2. La dernière chute
  possible est dans la boucle d'upsert, avant la boucle de retrait.
- Pas de tempête d'événements : un fil déjà `available: false` est sauté à
  chaque sync suivante (l.10249). L'estampille porte la nouvelle
  génération/séquence (l.10199, 10250) — honnête, le retrait est causé par
  ce snapshot.

Conclusion : F2 fermé. La projection reflète le retrait natif, sans perte
d'historique et sans mutation en cas d'échec.

---

## Les six INFO de r2 ne deviennent pas bloquants

- I-1, I-2, I-4, I-6 : non touchés par les quatre correctifs. Statuts r2
  inchangés.
- I-3 (longueurs UTF-16 vs octets dans le schéma) : la nouvelle mesure du
  journal est en octets UTF-8 (l.428). Elle va dans le sens noté par r2.
- I-5 (`output.timedOut` code mort, l.383) : toujours présent, cosmétique,
  hors périmètre de cette ronde.

---

## Bugs nouveaux

Aucun. Aucun déclencheur attendu/réel à signaler. Aucun fix demandé à Sol
sur ce lot.

---

## Limites de cette revue

1. Aucun `tsc`, build, lint ou test lancé. La validité à la compilation du
   correctif C02 n'est pas prouvée par compilation ; les noms de champs ont
   été vérifiés à la main dans `sdk.d.ts` 0.3.276.
2. Rust natif : lecture ciblée de la seule définition `bytes` du journal
   (`journal.rs:698`). Le reste du pont natif et l'application daemon de la
   borne 16 KiB restent du volet Sol, non relus ici.
3. Aucune preuve runtime. Les preuves d'exécution appartiennent aux agents
   de tests en cours ; je n'ai doublé aucun de leurs tests.
4. Divergence théorique de re-sérialisation à la frontière exacte des
   16 KiB (F1). Effet limité à un refus de disponibilité. Noté, non bloquant.
5. Les rapports r1 et r2 restent la référence de tous les autres axes. Cette
   ronde ne les re-vérifie pas et ne baisse aucun oracle.

---

## Conclusion

Les quatre durcissements sont implantés conformément aux réponses de Sol.
C01 : allowlist fermée à six clés, refus nommé de toute autre clé, sous-clés
de `permissions` bornées au contrat. C02 : les quatre champs de médiation
exclus par des gardes `undefined`. F1 : wire 24 KiB et contenu 16 KiB
séparés, mesure alignée sur `entry.bytes.len` natif, read et follow couverts.
F2 : retrait de disponibilité après snapshot complet validé, marqueur et
historique conservés, autres racines intactes, aucune mutation en cas de
panne. Aucune régression du chemin positif d'origine sur les quatre points.

**Verdict : APPROVE.**
