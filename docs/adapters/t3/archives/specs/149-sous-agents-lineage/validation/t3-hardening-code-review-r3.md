# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-hardening-code-review-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-hardening-code-review-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10. Sous-agent de revue indépendant (GLM 5.3 Flash). Ronde ciblée.
Périmètre : les quatre findings seulement, et leurs impacts directs. Le reste
des axes reste couvert par r1 (`t3-permissions-code-review-r1.md`, APPROVE) et
r2 (`t3-lineage-code-review-r2.md`, APPROVE). Aucune re-exploration générale.
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

Non-régression : un balayage de `apps/server/src` montre que seules les
gardes référencent ces quatre champs. Aucun chemin T3 actif ne les pose.
`makeClaudeQueryOptions` (l.1148-1201) ne les définit pas. La garde ne peut
donc faire perdre un fait qu'à un chemin futur qui changerait la médiation —
exactement l'objectif du finding. Direction sûre.

<a id="extrait-3"></a>

## Extrait 3

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
