# Audit de réutilisation — SPEC140

Statut : PASS de conception, contrôle final validé par le principal avant tasks.
Date : 2026-10-07. Ce PASS ne représente aucune implémentation ni test exécuté.

## Inventaire et décisions

| Item prévu | Décision et existant | Preuve / justification |
|---|---|---|
| Reconnaissance et projection | REUTILISER MessagesTimeline.logic.ts | /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts:73 ; extension pure du prédicat existant |
| Bulle droite et actions | REUTILISER UserTimelineRow | /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx:2113 ; alignement existant ; copie originale à2249 |
| Repli et corps | REUTILISER CollapsibleUserMessageBody et UserMessageBody | /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx:3937 et4005 ; branche locale, pas une liste parallèle |
| Défilement et mesure | REUTILISER onToggleWorkEntry et rowSize | /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx:633,1320,1328 ; identité fil/message locale |
| Logo Bridget | CREER uniquement l'asset T3 importé | Logo source /Users/moi/Nextcloud/10.Scripts/64.bridget/assets/branding/bridget-logo.svg ; recherche d'assets T3 sans logo Bridget ; aucun service ou URL externe |
| Nom humain | REUTILISER le texte d'enveloppe | Aucun contrat de provenance des messages T3 ; ne pas créer de lookup/prop destinataire sans source |
| Titre du fil | REUTILISER BridgetMessage et Store.thread_show | Champ racine facultatif additif ; l'autorité d'appartenance de la requête existante est conservée ; pas d'API nouvelle |
| Métadonnée remise | REUTILISER defer_idempotent_delivery et Deliver | Résolution éphémère à la remise comme from_display_name ; journal, canon et project_thread_wake non modifiés |
| Enveloppe T3 | REUTILISER le constructeur dans t3code.rs | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3090 ; en-tête historique inchangé, ligne JSON titre bornée additive |
| Tests | REUTILISER tests de logique, composants et Rust existants | Pas de harnais global nouveau, de dépendance, service ou migration |

## Existant potentiellement pertinent

Les publications de fil action/blocker/decision/history existent dans le contrat de transport, mais une sollicitation ne transporte pas leur nature. Ne pas les déduire du corps. ThreadNotice refuse les champs inconnus : l'étendre casserait la compatibilité. BridgetMessage accepte le champ racine additif. Les titres des anciennes enveloppes ne sont pas retrouvés par une interrogation du navigateur.

## Duplications évidentes

Aucune après sélection de la branche existante de CollapsibleUserMessageBody, de la logique existante et de Store.thread_show. Un composant local n'est acceptable que s'il rend cette branche lisible sans créer de seconde responsabilité métier. Aucun store global de cartes ou registre de noms.

## Règles et specs applicables

AGENTS140 et constitution : isolation, zéro commit automatique, anti-doublon, interfaces simples, tests observables. SPEC134 pour les noms ; SPEC136 pour le silence et le canon historique ; SPEC138 pour l'autorité des destinataires ; SPEC139 pour la reconnaissance d'enveloppe. Les standards Cartae sur Next/i18n/backend Python ne remplacent pas la structure React/Vite et Rust de ces dépôts.

## Journal de recherche

Le principal et l'inventaire ont recherché noms et responsabilités de hasBridgetEnvelopeHeading, UserTimelineRow, CollapsibleUserMessageBody, UserMessageBody, onToggleWorkEntry, rowSize et les assets T3. Ils ont lu les constructeurs d'enveloppe, BridgetMessage, ThreadNotice, project_thread_wake, defer_idempotent_delivery, Deliver et Store.thread_show. Les preuves ci-dessus sont les points d'entrée ; les numéros des modifications finales devront être consignés à la convergence.

## Arbitrages

| Sujet | Décision | Motif |
|---|---|---|
| Titre | Champ racine facultatif à la remise | Extension autorisée par utilisateur ; pas de changement du canon ni du contrat strict ThreadNotice |
| Ancien titre absent | Fil partagé | Pas de titre inventé, pas de backfill ni requête à l'exécution |
| Destinataire | Ne pas afficher de nom absent | Le modèle T3 de message ne fournit pas cette provenance ; position droite conservée |
| Logo | Asset local du logo officiel | Seul fichier de présentation nouveau nécessaire ; recherche préalable sans équivalent T3 |
| Corps lisible | Retrait du seul suffixe final exact connu | Texte inconnu conservé ; brut intégral accessible |

## Gate avant tasks

- [x] Items du plan extraits et équivalents recherchés.
- [x] Réutilisations privilégiées et créations limitées justifiées.
- [x] Aucune duplication évidente non arbitrée.
- [x] Extension de titre autorisée et compatibilité traitée.
- [x] Aucun nouveau service, API, stockage, migration ou dépendance.

Le principal a lu spec, plan, rapport et checklist puis validé le gate PASS avant génération de tasks.md. Ce rapport ne permet pas à lui seul de déclarer les tests ou la livraison terminés.

## Arbitrages effectivement appliqués pendant l'implémentation

- REUTILISER projectBridgetEnvelope/hasBridgetEnvelopeHeading dans /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.logic.ts:100 et109. La projection transporte uniquement des faits de présentation ; aucune couche d'identité.
- REUTILISER CollapsibleUserMessageBody, avec un composant local BridgetMessageBody dans le même fichier /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/components/chat/MessagesTimeline.tsx:3973 et4034. L'extraction porte la règle réelle de compatibilité disclosure/ancres/brut ; pas un wrapper vide ni une seconde liste. Recherche préalable des primitives et responsabilités consignée dans l'inventaire initial.
- CREER seulement l'asset local /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web/src/assets/bridget-logo.svg. Aucun équivalent T3 trouvé avant écriture ; cmp confirme l'identité du logo officiel source. Le bouton natif et les tokens existants évitent une dépendance.
- REUTILISER le contrat racine et les chemins de remise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-core/src/message.rs:95 et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/crates/bridget-daemon/src/daemon.rs:3964,7515,14348. Les titres clients sont retirés avant les octets durables, pas admis comme autorité. L'extension de communication.rs est un test du canon existant, pas un nouveau canon.
- REUTILISER les tests de logique et UI existants, les modules Rust et la recette Browser T3 isolée. Aucun framework de test nouveau. Le petit helper local de retrait de suffixe porte une compatibilité exacte sur cinq familles réelles, pas une abstraction générale de parsing.

Correction audit QUAL-001 : reconnaissance limitée au suffixe fermé via sous-agent codex/claude/cursor avec référence16hex dans la projection existante. Les formats inconnus restent verbatim.12 tests RED/GREEN et une copie UI exacte vérifient le bénéfice ; aucun identifiant de routage n'est supprimé du brut.
