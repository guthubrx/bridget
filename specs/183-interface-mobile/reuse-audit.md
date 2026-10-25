# Audit de reutilisation de l'existant — interface-mobile183

## Decision

Statut: PASS
Date: 2026-10-25
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/183-interface-mobile/specs/183-interface-mobile

Conclusion courte: le plan étend les composants natifs existants. Aucun nouveau moteur de recherche, annuaire, stockage serveur ou service réseau. Les ajouts portent des règles de présentation, d’identité ou d’isolation nécessaires au parcours.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 12 |
| Items audites | 12 |
| Reutilisations deja prevues | 12 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Volet/annuaire | AgentDirectory et ProjectAvatar | android/app/src/main/java/app/cartae/bridget/MainActivity.kt:215 | Réutiliser liste et icônes dans le tiroir, pas une copie |
| Favoris/filtres | favorite,visibleAgents | android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt:261 | Réutiliser préférences, indexer avant rendu |
| Repères d’attention | preferencesKey,historyKey et observations | android/app/src/main/java/app/cartae/bridget/LocalStore.kt:98 ; MobileViewModel.kt:455 | Extension bornée et par association ; aucune présence assimilée au succès |
| Recherche dans le fil | groupes messages et fonctions texte | android/app/src/main/java/app/cartae/bridget/MainActivity.kt:319 ; ConversationText.kt | Ajouter comportement de recherche dans l’existant |
| Dictée composeur | RecognizerIntent | android/app/src/main/java/app/cartae/bridget/UserInputForm.kt:103 | Même mécanisme système, identité figée, résultat éditable |
| Préparation fichier | importFile/enqueueFile | android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt:550 ; LocalStore.kt | Réutiliser commande durable et fichier chiffré, différer réseau |
| File d’envoi | MessageQueue | android/app/src/main/java/app/cartae/bridget/MessageQueue.kt:24 | Aucun nouveau contrat de mutation |
| Code copiable | RichMarkdown/HighlightedCode | android/app/src/main/java/app/cartae/bridget/RichContent.kt:78 | CommonMark et coloration conservés ; toolbar locale |
| HTML isolé | HtmlPreview/htmlPreviewDocument | android/app/src/main/java/app/cartae/bridget/RichContent.kt:143 | Étendre avec hôte distinct et sandbox, pas une seconde filière de téléchargement |
| Compactage natif | journal/record_active_act | crates/bridget-transport/src/codex_app_server.rs:2105 | Sous-type du kind tool existant, aucune table |
| Cycle de vie adaptateur | parse_detail/activity_payloads | crates/bridget-daemon/src/t3code_contract_v2.rs:206 ; t3code.rs:4113 | Normaliser uniquement les items structurés et leur identité |
| Tests et preuves | JVM, instrumentation et générateur | android/app/src/test/java/app/cartae/bridget/RichContentTest.kt ; android/app/src/androidTest/java/app/cartae/bridget/UserInputVoiceTest.kt | Étendre les lots existants ; nouvelles fixtures synthétiques seulement |

## Existant potentiellement pertinent non mentionne

Aucun après exploration. Les projections execution et reconnect_count de l’annuaire peuvent compléter les faits mobiles existants ; elles ne remplacent pas un événement terminal absent.

## Duplications evidentes

Aucune. Doublon de table, endpoint de recherche ou annuaire exclu avant rédaction du plan.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Autorité produit | Contrats génériques ; notes des intégrations dans leur adaptateur |
| ConstitutionXVI | Worktree dédié | Serveur, branche183, base figée |
| ConstitutionXVIII | Complexité | Index O(n), limites existantes ; aucun appel réseau par entrée |
| ConstitutionXIX/XX | Réutilisation/maintenance | Aucune nouvelle dépendance ; comportements testables sans contexte caché |
| research/07 | Accessibilité | Cibles48dp, libellés, ancienneté lisible |
| research/06 | Contenu non fiable | Zéro pont privilégié, tests hostiles, moteur corrigé |
| Preuves-correction | Parcours réels | Diaporama autonome dans le commit, originaux privés |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
|177|Identité et portée mobile|Réutiliser et tester sans élargissement|
|178|Moteur et dépendances corrigés|Conserver les gardes et les tests|
|180|File et révisions|Conserver édition/annulation/promotion|
|181|Questions et dictée|Même modèle d’identité et réponse libre|
|182|Produit autonome|Pas de branding externe dans les guides généraux|

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg AgentDirectory/ProjectAvatar/favorite/visibleAgents | Android | Composants et état présents |
| rg importFile/beginFileSelection/enqueueFile | Android | Import actuel avec effet réseau ; staging à séparer |
| rg RichMarkdown/HtmlPreview/RecognizerIntent | Android | Rendu et reconnaissance présents |
| rg contextCompaction/compacted dans schéma généré | moteur installé | Débuts/fins et identités attestés |
| rg turnItems/tool.progress/activities | adaptateur | Parseur et projection existants |
| rg MobileOperation/agents_page/ScopedAgentInfo | Rust | Annuaire déjà filtré avant pagination |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Annuaire/stockage/fichier/code | réutiliser | Responsabilités existantes1:1 |2026-10-25|
| Vue de sécurité HTML | étendre l’existant | Une instance dédiée et enveloppe sandbox portent une frontière de sécurité, pas un wrapper vide |2026-10-25|
| Repère et recherche | étendre l’existant | Pas de moteur global ni de synchronisation de brouillons |2026-10-25|

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md prévoit la réutilisation et les divergences sont justifiées
