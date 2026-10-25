# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/plan.md](../../../../../../specs/140-bulles-compactes/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut : Implemented, non déployé. Base T3 : 9905d9cbc2, branche session-140-bridget-compact.
Code T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.
Code et artefacts Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.

<a id="extrait-2"></a>

## Extrait 2

1. Étendre la logique de présentation dans MessagesTimeline.logic.ts. Reconnaître un en-tête complet au début du texte dans une borne de 1024 caractères. Produire une projection pure, sans attestation d'origine ni mutation du message.
2. Réutiliser UserTimelineRow, CollapsibleUserMessageBody, UserMessageBody et leurs actions. Ajouter une branche de présentation Bridget, pas une seconde liste de messages. La copie conserve le texte source. Les pièces jointes et actions restent sur leurs chemins existants.
3. Importer le logo réel depuis un asset local. Aucun chargement réseau. Réutiliser les couleurs, contrôles et primitives existants ; aucun paquet externe. La ligne entière est activable et son nom accessible reste compréhensible.
4. Conserver un état mémoire par identité de fil et de message. Réutiliser onToggleWorkEntry et les mécanismes de mesure/ancrage. Les changements de fil, le recyclage virtualisé et les mises à jour d'un message ne déplacent pas l'état vers une autre identité.
5. Le corps lisible ne retire qu'un suffixe technique final exactement reconnu. Tout suffixe inconnu ou incomplet reste visible. « Détails techniques » affiche toujours le texte intégral, sans l'interpréter comme HTML.
6. Étendre BridgetMessage par thread_display_title facultatif, avec défaut et omission si absent. Ne pas étendre ThreadNotice, dont la lecture refuse les champs inconnus. Les anciens lecteurs ignorent le champ racine additif.
7. Résoudre le titre dans defer_idempotent_delivery et la remise classique Deliver. Réutiliser Store.thread_show(destinataire, thread_id), donc son contrôle d'appartenance. Ne pas enrichir project_thread_wake ni les octets persistés. Comme from_display_name, le champ appartient seulement à la projection de remise.
8. Le pont T3 garde exactement l'en-tête historique, puis ajoute si présent une ligne « Titre du fil Bridget : \"titre\" » avec la valeur encodée en JSON. Le pont normalise espaces et caractères de contrôle puis borne à 200 caractères. La projection T3 consomme cette ligne. Une ancienne alerte reste « Fil partagé ».

<a id="extrait-3"></a>

## Extrait 3

T3 : apps/web/src/components/chat/MessagesTimeline.logic.ts et son test, MessagesTimeline.tsx, tests de rendu proches et asset logo local.
Bridget : crates/bridget-core/src/message.rs, crates/bridget-daemon/src/daemon.rs, crates/bridget-daemon/src/t3code.rs et leurs tests existants.
Ces chemins sont relatifs aux deux racines absolues indiquées au début du plan.
