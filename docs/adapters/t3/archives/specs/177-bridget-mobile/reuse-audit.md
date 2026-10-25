# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/reuse-audit.md](../../../../../../specs/177-bridget-mobile/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Autorité humaine mobile | human_principal_actor et human_authority_guard | referent_control.rs:31 ; daemon.rs:18115 | Créer une identité d'appareil ; réutiliser les opérations mais ne pas usurper le libellé ou la conversation |
| Migration du maître | reattest dépend du runtime d'origine | t3code_mcp.rs:39 ; native_delegation.rs:4017 | Prouver une adaptation liée à la machine d'origine avant bascule ; aucun relâchement de validation |

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
|---|---|---|
| rg mobile / gateway / http / android | crates, scripts, docs et manifestes | Aucun accès mobile ni module Android |
| rg HumanPrincipal / human_authority_guard / control_client_actor | daemon.rs, protocol.rs | Deux gardes humaines existantes, responsabilités différentes |
| rg reattest / loopback / validate_endpoint | t3code_mcp.rs, native_delegation.rs | Validation locale du runtime et du port |
| rg handle_idempotent_send / human_from_bare_client | daemon.rs | Envoi humain natif réutilisable sans identité d'agent |
| rg human_cancel / resolve / lookup | delegation.rs, human_inbox.rs, idempotency.rs | Puits existants à réutiliser |
| lecture federation-services.md | docs | Un maître, pas de réplication |
| inventaire PATH et emplacements Android | Cartae | Outils Android absents des emplacements usuels |

<a id="extrait-3"></a>

## Extrait 3

Lecture courte de T3 : apps/server/src/project/T3ProjectFileLoader.ts charge le projet sur la machine qui possède son workspace ; apps/server/src/vcs/GitVcsDriverCore.ts transmet explicitement cwd au lecteur Git. Aucun mécanisme de projet fédéré équivalent dans ces fichiers. Bridget réutilise son resolve_communication_project existant sur le worker provisionné, avec le canal OriginProof déjà introduit dans ce lot. Il ne recrée pas une règle lexicale de projet et ne transforme pas le chemin Mac en chemin Linux. La garde humaine conserve le binding T3 et l'égalité du projet validé.

<a id="extrait-4"></a>

## Extrait 4

Pour le relais SSH duplex, les mêmes sources T3 ne couvrent pas socat/MSG_PEEK. Bridget conserve son runner natif et son contrat de flux SSH ; le correctif évite de consommer puis réinjecter le premier octet. La preuve macOS utilise deux échanges espacés, pas seulement un écho immédiat.

<a id="extrait-5"></a>

## Extrait 5

Recette Pixel : T3 ChatView.tsx, scrollToEnd et bouton de retour au direct (4663+, 8008+), maintient le suivi de fin jusqu’à un geste de lecture antérieure. Le client Compose reprend ce comportement sans réutiliser son code web : en-tête compact, boutons secondaires dans un menu et saisie ancrée. Pour l’historique vide, le lecteur Attach natif de Bridget connaissait déjà SnapshotCaughtUp même sans through_seq ; le relais mobile perdait ce fait en testant seulement is_some. Un booléen interne non sérialisé conserve maintenant l’événement explicite. Le protocole et les limites restent inchangés. T3 ne possède pas ce relais Unix distant ; le lecteur natif existant est la référence appropriée.

<a id="extrait-6"></a>

## Extrait 6

### Reassociation and expired QR
The user reused the consumed first QR after forgetting local access. The server refused correctly; Android incorrectly showed a generic unconfirmed response. Dedicated EN/FR guidance now explains expiry, one-use pairing and the explicit new-association action. No pending request or saved work is discarded automatically. T3 ConnectionsSettings.tsx computes expiry from expiresAt and displays an expiration label; the temporary private QR page follows the same expiry principle without copying its implementation. Lost-response reconciliation is unchanged.

<a id="extrait-7"></a>

## Extrait 7

### Recent history, project labels and keyboard (physical feedback)
T3 ChatView.tsx retains follow-to-end until a real user scroll, and observes composer resize. Android uses its existing LazyListState with viewport-size changes and stable message keys. T3 ProjectIconPickerDialog uses a compact project icon; Bridget uses a local monogram and a short name, with host/parent/digest only for homonyms. Native host/root identities remain unchanged. Recent history reuses Attach Tail(1) to locate the end, then the existing bounded forward page. The optional tail flag defaults to false for previous native requests; workers need no new operation. Older messages are loaded explicitly without moving the live cursor.
