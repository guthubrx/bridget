# Contrat164 — Projection de lecture du journal natif

## Frontière

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/contracts/presentation.md#extrait-1).

## Blocs

- **Texte.** Contenu exact de fragments reconnus, concaténés sans séparateur ajouté. Plage de séquences, clé du premier événement, session et références sources. Présentation Markdown en lecture seule.
- **Activité.** Commande/outil reconnu depuis une forme effectivement observée. Entrée/sortie/code retour/durée uniquement quand explicitement présents et valides. Sans corrélation explicite, afficher des activités distinctes. Détails repliables, sans exécution.
- **Technique.** Événement non reconnu, historique, anomalie ou données originales. Texte JSON échappé/texte brut, consultable à la demande. Conserver tous les événements.

La représentation TypeScript concrète reste au développeur. Elle doit être plus petite que les modèles fournisseur existants et ne pas fabriquer leurs identifiants ou leurs rôles. Elle ne requiert ni nouveau type public ni modification de BridgetLineage.

## Frontières du texte

Regrouper seulement dans la même tâche et la même session, sans événement intermédiaire incompatible. Deux identifiants explicites de livraison différents séparent les blocs. `message_id` identifie la livraison Bridget et non un message fournisseur ; un identifiant commun ne suffit donc pas à créer un message fournisseur. Si l'un manque, conserver un bloc de texte contigu seulement et ne pas revendiquer une identité fournisseur. Un événement non textuel, une lacune déclarée, une éviction ou une rupture de session ferme le segment.

Un saut numérique de séquence seul ne prouve pas une perte : le journal peut être filtré. Le début d'une page/fenêtre sans preuve de début est un extrait. Chaque page historique est projetée seule. Aucun bloc n'est assemblé à travers deux pages séparées ; le curseur atteste la lecture, pas une identité de message fournisseur. Le résultat natif `show` reste séparé et paginé ; ne pas l'utiliser pour compléter arbitrairement une progression.

## Fidélité et rejeu

Entrées compatibles de test : fragments `"workflow"`, `" Spec"`, `"Kit"`, `" et"` → texte exactement `"workflow SpecKit et"`. Conserver retours à la ligne, Unicode, espaces et délimiteurs Markdown. Rejouer une séquence inchangée ne crée aucun nouveau texte. Une séquence conflictuelle est détectée par mergeJournalEvents avant déduplication silencieuse des conteneurs. Le premier événement reste la source normale, un témoin différent par séquence chargée reste consultable et des conflits supplémentaires sont signalés sans copies illimitées. Aucun remplacement silencieux.

Clé stable du bloc fondée sur son événement initial, indépendante de la longueur du texte. Les détails, la sélection et le défilement ne doivent pas sauter lors d'un ajout. Le lecteur qui remonte ne subit aucun retour forcé au bas.

## Rendu sûr

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/contracts/presentation.md#extrait-2).

## Bornes

La projection transforme uniquement la fenêtre/page déjà chargée. Pas de requête supplémentaire par événement. Fenêtre de 4 096 événements, page de 100 événements, résultat par fenêtres de 16 KiB et plafond de 256 KiB inchangés. Accumuler les fragments dans un tableau puis `join` ; complexité O(n+c) pour n événements et c caractères. Les références sources ne nécessitent pas une seconde copie profonde du payload. Aucun historique persistant. Le réducteur existant peut conserver au plus 64 lacunes éphémères identifiées par bornes/raison, sans répéter un gap sticky. Pruner seulement hors fenêtre ; si une frontière pertinente doit être retirée sous pression, conserver une indication de coupe et interdire la fusion dans la zone incertaine. Signaler également toute coupe à 4 096 événements réalisée après merge local. Les blocs clos gardent leur identité et sont mémoïsés ; les événements bruts ne sont montés qu’à l’ouverture.

## États et actions

`caught_up` signifie que le lecteur a rattrapé cette page/ce flux. Cela ne signifie pas que la mission est finie. Disponibilité Lineage, contact distant et erreur de lecture restent séparés. « Résultat disponible » dépend seulement de l'état natif et du résultat exposé. Arrêt, parent, reconnexion, pages et permissions utilisent les commandes actuelles. Pas de composeur.

## Formes positives des oracles

Reprendre le tableau du plan et le reçu ROOT `/Users/moi/.cache/bridget-delivery164-20261013/journal-shapes164-preparation.json`. Les payloads update/text/content sont des segments. Les update/command/item-started sans item_id et update/command/item-completed avec item_id sont distincts tant qu’aucune corrélation partagée n’existe. Le completed porte lui-même les preuves de résultat. Claude outil appel seul, ACP tool_call_id et refus attributed_to Bridget ne sont pas transformés en parole/réussite de l’agent. Les formes inconnues restent des détails.
