# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/research.md](../../../../../../specs/181-questions-mobile/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Décision prioritaire utilisateur : aucune dépendance centrale T3, pas de partage des brouillons en cours. Brouillon local et résolution native partagée.

<a id="extrait-2"></a>

## Extrait 2

### Vérification finale de provenance et notification
T3 mobile rattache PendingUserInputCard au fil qui porte la demande (apps/mobile/src/features/threads/PendingUserInputCard.tsx). Bridget réemploie sa sélection de conversation existante : le lien authentifié reçoit source_agent depuis la présence et le nom de l'annuaire, puis ouvre le formulaire ciblé. Le modèle ne choisit ni l'identité ni la machine. Si la source ne peut être relue, la boîte À traiter garde le guidage d'indisponibilité. Aucun appel T3 n'est requis.

<a id="extrait-3"></a>

## Extrait 3

### Correction audit : mise à jour à capacité pleine
T3 AntigravityAdapter.ts respondToUserInput lit la demande existante par requestId, GrokAdapter.ts conserve ses pendingUserInputs par set/get/delete. Aucun cas de cache borné à32 équivalent dans ces adaptateurs : cette borne appartient à Bridget. La capacité limite seulement les nouvelles clés ; une mise à jour/relecture de clé connue doit rester permise. Test32entrées : échec avant (31au lieu de32), correction contains_key puis relecture, ajout33e refusé et clôture explicite conservée.

<a id="extrait-4"></a>

## Extrait 4

### Correction audit : notifications après la première page
T3 notificationResponseConsumer.ts et notificationNavigation.ts routent une réponse de notification vers le thread attesté ; aucun équivalent de relai FCM avec relecture de toutes les demandes natives Bridget. Réutiliser la pagination mobile181 existante, avec budget15s et maximum2048demandes natif, et publier chaque page au fil de sa lecture. Le modèle ne fournit aucun contenu dans la poussée. Le plafond initial10ne doit pas masquer la11e question.

<a id="extrait-5"></a>

## Extrait 5

### Correction audit : purge idempotente
T3 ProviderSessionRuntime.ts:200 retire importedTranscripts à l’écriture ; pas d’équivalent de tombstone natifBridget purgé à la lecture. La requête181 réécrivait chaque tombstone déjà purgé. Conserver json_remove pour la première purge, ajouter le prédicat questions présent et un index partiel updated_at de même prédicat. Aucun texte ne reste au-delà de la règle précédente, aucune identité de rejeu supprimée. Le test compare total_changes sur deux lectures successives après purge.
