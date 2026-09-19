# ADR 029 — Extraire Bridget communication sans réécrire ses garanties

Date : 2026-09-05. Statut : proposition de réalisation du périmètre approuvé ; gates techniques à exécuter en session 089.

## Contexte

La valeur utilisateur est l'échange fiable entre agents hétérogènes. Le daemon dépend actuellement du plugin le service compagnon et compile avec des fonctions de GUI et de runtime de projet. L'interface ne doit plus imposer son cycle de développement aux communications.

## Décision

Extraire dans un dépôt indépendant avec historique. Préserver les trois crates et les invariants éprouvés ; supprimer les dépendances métier et de présentation par coutures couvertes, non par réécriture. Conserver session native/ACP, identité stable, envoi idempotent, demandes liées, ledger, journal/attach et fédération SSH vers un maître unique.

le service compagnon reste extérieure. Les contrats publics de communication avec les services et les événements attestés subsistent ; les politiques d'objectifs, de greffe et de réassignation ne sont pas une dépendance du noyau. Aucune nouvelle autorité A2A, graphique ou distante n'est introduite. Aucun mécanisme de permission/facturation du fournisseur n'est supprimé pour retirer le sandbox de rendu HTML.

Le nouveau binaire doit disposer d'un espace isolé avant son premier démarrage. Les données, sessions et installations historiques ne migrent pas automatiquement. La recette interserveur est un gate de livraison, pas un suivi facultatif.

## Alternatives rejetées

Réécriture complète (perte des preuves), retour tmux (besoin utilisateur contraire), simples flags masquant toutes les anciennes dépendances (coût maintenu), adoption d'un framework multi-agents (nouvelle orchestration au lieu de découplage). Une passerelle A2A ou une vue T3 seront des décisions ultérieures fondées sur un consommateur réel.

## Conséquences

Comparaison possible au commit source, reprise progressive et réduction mesurée. Coût initial d'inventaire/tests supérieur à une copie des fichiers « évidents », mais chaque garantie doit rester démontrable. Un résultat de livraison ne certifie jamais la réussite sémantique d'une mission.

Référence de réalisation : specs/089-communication-core/plan.md et contracts/communication.md dans cette session.
