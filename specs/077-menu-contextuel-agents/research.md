# Recherche - SPEC-077 Menu contextuel des agents

Date: 2026-08-30

## D1 - Conserver deux déclencheurs visibles et experts

Décision: conserver les trois points comme signe visible et ajouter le clic droit comme accélérateur expert vers le même menu.

Rationale: un clic droit seul est peu découvrable. Nielsen Norman Group recommande un signe visible pour les menus contextuels et demande que les options restent identiques quelle que soit la voie d'ouverture. Microsoft Research décrit également l'intérêt d'une transition entre un menu visible pour les novices et un geste accéléré pour les utilisateurs experts.

Alternatives considérées:
- clic droit seul: rejeté car invisible;
- trois points seuls: rejeté car moins rapide pour un usage bureau intensif;
- deux menus séparés: rejeté car source de divergence.

Impact mainteneur: une seule fonction et un seul nœud DOM portent les deux interactions.

Sources:
- https://www.nngroup.com/articles/contextual-menus/
- https://www.nngroup.com/articles/contextual-menus-guidelines/
- https://www.microsoft.com/en-us/research/publication/an-empirical-evaluation-of-marking-menus/

## D2 - Employer le pattern ARIA menu button

Décision: role menu, aria-haspopup menu, aria-expanded et menuitem, avec focus déplacé à l'ouverture et navigation par flèches.

Rationale: W3C APG documente ce contrat et précise qu'ARIA seul ne fournit pas le comportement clavier. Le script doit donc gérer explicitement le focus.

Alternatives considérées:
- conserver role dialog: rejeté car le contenu devient principalement une liste de commandes;
- simple div avec boutons tabulables: rejeté car la sémantique et les attentes clavier seraient imprécises.

Impact mainteneur: les règles de clavier sont regroupées dans le gestionnaire existant et couvertes par tests.

Source:
- https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/
- https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/

## D3 - Transformer la fiche existante

Décision: réutiliser la fiche globale SPEC-071/073 et la rendre plus compacte.

Rationale: elle résout déjà le clipping, le positionnement, la fermeture extérieure, le redimensionnement et la conservation pendant rafraîchissement.

Alternatives considérées:
- créer un second context-menu: rejeté comme duplication;
- utiliser le menu natif du navigateur: rejeté car il ne peut pas exposer les commandes métier;
- ajouter une bibliothèque: rejeté car aucune dépendance n'est nécessaire.

Impact mainteneur: moins de surfaces concurrentes et suppression du comportement de mini-dialogue.

## D4 - Persister seulement des préférences locales

Décision: une structure locale versionnée pour épinglage, masquage et curseurs de lecture.

Rationale: ces préférences sont propres au poste et ne changent aucun fait de flotte. Un endpoint serveur créerait une responsabilité et une migration injustifiées.

Alternatives considérées:
- mémoire de session seule: rejetée car les préférences disparaîtraient au rechargement;
- stockage serveur: rejeté hors besoin multi-postes;
- une clé par action: rejetée car validation et migration seraient dispersées.

Impact mainteneur: un schéma unique et un fallback unique.

## D5 - Rendre le masquage réversible

Décision: une section repliée Agents masqués avec le même rendu de ligne et le même menu.

Rationale: retirer totalement la ligne rendrait l'action difficilement réversible. La section repliée conserve la sobriété tout en empêchant un état sans issue.

Alternatives considérées:
- bouton global de remise à zéro: trop large et destructif;
- restauration par stockage navigateur: non acceptable pour l'utilisateur;
- masquage temporaire: ne répond pas à la demande durable.

Impact mainteneur: réemploi du pattern Agents arrêtés, sans page de réglages.

## D6 - Réutiliser entièrement SPEC-075

Décision: aucun nouveau backend et aucune nouvelle commande de cycle de vie.

Rationale: les routes, validations, confirmations et verdicts existent et sont testés. Le menu ne fait qu'exposer plus clairement cette capacité.

Impact mainteneur: aucune duplication de protocole ou de règle d'éligibilité.
