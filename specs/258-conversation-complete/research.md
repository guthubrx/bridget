# Recherche 258

## Décisions établies

- Le paquet T3 chargé date du 3 novembre ; le paquet pages du 8 novembre n'est
  pas installé. Réutiliser son code intégré en 75c7dafe51, sans seconde UI.
- ChatView appelle ThreadTimeline via projectBridgetTimeline pour les enfants,
  mais useTimelineFollow reçoit encore les entrées de projection ordinaires.
  Le suivi du journal doit être éprouvé et raccordé à ces faits.
- ChatView interdit canOperateThread sur bridgetTaskRef et ne monte pas
  ChatComposer. Le serveur refuse aussi les commandes ordinaires. Garder ce
  refus ; ajouter le parcours humain natif dans les ports existants.
- HumanLineage expose list/show/journal/watch/cancel. SelectRuntime existe dans
  attach et reçoit une confirmation du wrapper. Étendre plutôt que contourner.
- BridgetMessage transporte le texte et des références ; il ne faut pas
  transformer une image en référence locale fictive. Capacités réelles requises.

## Références consultées

Code primaire : ChatView.tsx, ChatComposer.tsx, BridgetTaskJournal.tsx,
BridgetTimeline.logic.ts, useTimelineFollow.ts, BridgetReader.ts,
BridgetLineage.ts ; protocol/lineage_protocol.rs, daemon/native_lineage.rs,
select_runtime_from_attach dans daemon.rs, attach.rs.
Baseline : /Users/moi/.speckit/research/07-frontend-design-systems.md.
Clavier des sélecteurs : https://www.w3.org/WAI/ARIA/apg/patterns/combobox/ .
Virtualisation : conserver la liste et les mécanismes de mesure existants,
sans introduire un second moteur de liste ou une dépendance supplémentaire.

## Choix de raccord après étude native

Le protocole humain est étendu par une opération de boîte fermée, protégée par
une capacité dédiée. Les variantes de lecture, envoi, sélection et interruption
partagent le contexte racine et la tâche ; elles ne changent pas les anciennes
réponses. Le contrôle modèle doit passer jusqu'au wrapper exact ; si une route
distante manque, elle est implémentée et testée, pas remplacée par le parent.
L'envoi humain réutilise la remise et ses reçus ; les primitives de contrôle
réutilisent l'identité d'exécution dérivée par le daemon. Les capacités sont
produites côté cible et vérifiées à chaque commande. Le catalogue n'est pas
présumé depuis celui du parent. Ces choix couvrent les inconnues techniques,
dont les détails sont éprouvés par les tâches de contrat et leurs tests.

L'étude native 848cba70 a confirmé les gardes et puits existants et signalé les
routes attach spécifiques comme points à factoriser. Ses limites (catalogue et
route distante non relus) ne sont pas présentées comme des preuves d'absence.
L'exploration T3 déléguée a été refusée cwd_outside_parent_project sans tâche ;
lecture directe autorisée poursuivie. Aucun changement de route pour la contourner.

IncidentOAuth10nov : les journaux corrélés des deux tours montrent exactement Failed to authenticate: OAuth session expired and could not be refreshed, puis terminal api_error/is_errortrue. Le SDK local types SDKAssistantMessageError inclut authentication_failed (apps/server/node_modules/@anthropic-ai/claude-agent-sdk/sdk.d.ts:3800 du worktreeT3). Le parseur accepte ce code ou le marqueur API dédié, avec diagnostic exact ; une citation ordinaire reste hors classement. La CLI installée expose claude auth login (compteabonnementexistant par défaut) ; aucun login/logout ni changement de compte lancé.
