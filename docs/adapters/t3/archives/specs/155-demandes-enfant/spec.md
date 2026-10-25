# Archives techniques de l’adaptateur T3

Document d’origine : [specs/155-demandes-enfant/spec.md](../../../../../../specs/155-demandes-enfant/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Réponse du parent à une approbation (question 2, défaut : non).
- Enfants Claude (flux `claude_stream_json`, y compris GLM) : leurs invites sont
  désactivées au lancement (`--permission-prompts none`,
  `crates/bridget-daemon/src/native_permissions.rs:313`). Session suivante
  (question 3).
- Enfants ACP (Cursor) et agents hors moteur natif.
- Réponse humaine en texte libre. La boîte humaine ne porte que des choix fermés.
- Interface graphique des demandes dans T3 ou dans le panneau Bridget.
- Décisions durables (« toujours autoriser », « pour la session », amendements de
  règles) : jamais proposées, jamais relayées.
- Méthode item/permissions/requestApproval (octroi additionnel à scope turn/session, hors périmètre v1) et toute autre méthode `item/*/requestApproval` inconnue :
  refus automatique `approval_method_unsupported`.
- Questions à secret (`isSecret`) : refus `secret_not_forwardable`. Élicitations en
  mode URL : refus `url_mode_unsupported`.
- Approbations d'écriture de fichier sous un parent `workspaceWrite` : refusées en v1
  (`file_confinement_unproven`). Elles reviendront avec une preuve de confinement à
  l'exécution, dans une session future.
- Parent dont `approvals_reviewer` n'est pas `user` (relecteur automatique Codex) :
  refus à l'admission en mode `forward`.
- Transfert d'une demande au grand-parent quand le parent se tait : un seul saut.
- Gel des délais pendant l'attente (session152) et budgets (session158).
- Réutilisation d'un enfant (session153).
