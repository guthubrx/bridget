# Plan140 — Bulles compactes et titre compatible

Statut : Implemented, non déployé. Base T3 : 9905d9cbc2, branche session-140-bridget-compact.
Code T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.
Code et artefacts Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.

## Réutilisation et conception

1. Étendre la logique de présentation dans MessagesTimeline.logic.ts. Reconnaître un en-tête complet au début du texte dans une borne de 1024 caractères. Produire une projection pure, sans attestation d'origine ni mutation du message.
2. Réutiliser UserTimelineRow, CollapsibleUserMessageBody, UserMessageBody et leurs actions. Ajouter une branche de présentation Bridget, pas une seconde liste de messages. La copie conserve le texte source. Les pièces jointes et actions restent sur leurs chemins existants.
3. Importer le logo réel depuis un asset local. Aucun chargement réseau. Réutiliser les couleurs, contrôles et primitives existants ; aucun paquet externe. La ligne entière est activable et son nom accessible reste compréhensible.
4. Conserver un état mémoire par identité de fil et de message. Réutiliser onToggleWorkEntry et les mécanismes de mesure/ancrage. Les changements de fil, le recyclage virtualisé et les mises à jour d'un message ne déplacent pas l'état vers une autre identité.
5. Le corps lisible ne retire qu'un suffixe technique final exactement reconnu. Tout suffixe inconnu ou incomplet reste visible. « Détails techniques » affiche toujours le texte intégral, sans l'interpréter comme HTML.
6. Étendre BridgetMessage par thread_display_title facultatif, avec défaut et omission si absent. Ne pas étendre ThreadNotice, dont la lecture refuse les champs inconnus. Les anciens lecteurs ignorent le champ racine additif.
7. Résoudre le titre dans defer_idempotent_delivery et la remise classique Deliver. Réutiliser Store.thread_show(destinataire, thread_id), donc son contrôle d'appartenance. Ne pas enrichir project_thread_wake ni les octets persistés. Comme from_display_name, le champ appartient seulement à la projection de remise.
8. Le pont T3 garde exactement l'en-tête historique, puis ajoute si présent une ligne « Titre du fil Bridget : \"titre\" » avec la valeur encodée en JSON. Le pont normalise espaces et caractères de contrôle puis borne à 200 caractères. La projection T3 consomme cette ligne. Une ancienne alerte reste « Fil partagé ».

## Sources concernées

T3 : apps/web/src/components/chat/MessagesTimeline.logic.ts et son test, MessagesTimeline.tsx, tests de rendu proches et asset logo local.
Bridget : crates/bridget-core/src/message.rs, crates/bridget-daemon/src/daemon.rs, crates/bridget-daemon/src/t3code.rs et leurs tests existants.
Ces chemins sont relatifs aux deux racines absolues indiquées au début du plan.

## Tests et ordre

Tests RED de projection avant son implémentation. Tests des cinq familles, faux positifs, borne d'en-tête, nom absent, reply, lots et métadonnée JSON. Tests Rust de compatibilité du champ absent, du destinataire non membre, des titres hostiles et du canon/journal inchangés après remise/rejeu. Tests ou recette DOM du vrai composant : clic, Entrée, Espace, copie originale, détails intégraux, messages longs, thèmes, largeur320 et état entre fils/messages. Format, lint, TypeScript et build ciblés sur le frontend ; tests Rust ciblés, jamais un succès global présumé.

## Gates

Audit de réutilisation PASS avant tasks. Analyse puis mise en œuvre. Convergence par exigences et preuves code/test ; aucune case cochée sans preuve. Contre-revue autre fournisseur demandée si joignable, borne courte. Essais isolés, pas de base active, service fournisseur payant ou redémarrage. Les scripts/templates projet absents justifient le protocole manuel documenté, pas la suppression d'une phase.

## Charge future et complexité

Article XIX/XX : projection pure dans la logique existante ; pas de framework de cartes, nouvelle API ou store global. Lecture du titre réutilise la requête et l'autorité existantes à la remise. Reconnaissance de l'en-tête O(1) grâce à la borne1024 ; rendu du brut O(longueur du texte) quand demandé, comme aujourd'hui. Métadonnée titre bornée, aucun lookup navigateur. La reconnaissance visuelle ne devient jamais un contrôle de sécurité.
