# Quickstart : valider les outils MCP Bridget

Prérequis : session 007 livrée (équipiers ACP, registre), daemon lancé.

## 0. Spike-gate de branchement (D-403 — avant toute implémentation)

Faux serveur MCP jetable (outil `probe`) lancé par chacun des 4 harness
(Claude, Codex, Gemini, équipier ACP via `mcpServers`), par injection
strictement éphémère. Consigner : commande exacte, version, stdout/stderr,
**diff vide des configs utilisateur**, nettoyage. Un échec → révision de spec.

## 1. Envoi par outil

Sur un agent Claude interactif branché : demander « envoie à codex-1 : l'été
s'annonce "chaud", à 100 % — $HOME inclus » via l'outil. Attendu : corps intact
à l'arrivée (vérifier au journal 007), résultat `accepted` avec id, entrée
ledger identique à un envoi binaire (SC-006).

## 2. Refus structurés

Passer codex-1 en DND puis renvoyer. Attendu : résultat métier `dnd` avec
motif et minutes restantes — pas d'`isError`, pas de texte à parser. Répéter
avec un destinataire inconnu (`unknown_recipient`).

## 3. Identité

- `bridget rename` de l'agent entre deux appels → l'expéditeur au ledger suit.
- Lancer `bridget mcp` à la main hors de tout agent → `identity_not_found`.
- Tuer l'agent, réutiliser artificiellement son pid → entrée invalidée
  (naissance différente), erreur explicite.

## 4. Annuaire et ledger

`bridget_who` : liste identique au binaire. `bridget_ledger view=requests` :
la demande suivie ouverte apparaît ; `view=messages` : les messages récents ;
sorties du binaire inchangées au golden test.

## 5. Robustesse

- Daemon arrêté → appel en `isError` explicite immédiat.
- Rafale d'appels au-delà de la limite simultanée → `busy` avant connexion.
- Harness qui abandonne l'appel (`notifications/cancelled`) après écriture →
  `outcome_unknown` au journal, pas de doublon au retry même id.

## 6. Prompt allégé

Comparer les blocs versionnés avant/après (SC-005 ≥ 60 %) et rejouer les
scénarios quickstart 007 §1-§4 avec le prompt réduit : comportement identique.
