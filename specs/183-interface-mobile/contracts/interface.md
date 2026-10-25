# Contrats183 — Présentation et activité native

Le client réutilise annuaire, journal paginé/suivi, questions, commandes et reçus mobiles. Les ajouts éventuels à l’annuaire sont des projections de faits existants, optionnelles et rétrocompatibles. Les autorisations restent antérieures à la pagination.

## Activité de compactage

Événement journal : event=update, payload.kind=tool et tool_type=context_compaction, payload.status=running|completed, payload.turn_id et payload.item_id opaques et bornés. La session/agent et la séquence proviennent du journal authentifié. Aucun contenu du contexte compacté n’est publié. Les adaptateurs ne produisent cet événement qu’à partir d’un signal structurel exact. Un client ancien peut ignorer le kind.

## Présentation

Volet depuis le fil ; tous/favoris, recherche, pagination. Marqueur de question lié à une demande active ; ouvrir le bon fil/formulaire. Marqueur à consulter lié à une nouveauté/activité terminale attestée, jamais à un simple heartbeat. L’ancienneté et l’absence de capacité restent visibles.

Envoi normal inchangé ; interruption seulement depuis l’action explicite de la file. Une page HTML ne peut appeler aucune commande. Une réponse dictée est seulement un brouillon éditable. Recherche limitée explicitement aux messages chargés.

## Isolation HTML

Contenu non fiable dans une iframe sandbox avec scripts locaux seulement, origine opaque, aucun pont natif, réseau/fichier/contenu/popup/téléchargement/permission refusés. Document hôte de confiance distinct. Mode code et statique disponibles. Moteur WebView non attesté ou ancien : pas de rendu HTML actif.
