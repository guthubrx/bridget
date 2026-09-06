# 091 — Communication utilisable et lancement explicite

Statut : en cours. Périmètre validé par l'utilisateur le 6 septembre 2026.

## Besoin

L'humain lance un équipier pour développer, lui parle et observe son travail sans tmux. Le profil choisi, les droits effectifs et l'état observé ne doivent pas se contredire. La skill explique les outils existants ; elle ne crée aucune autorité.

## Exigences

- FR-01 : choix explicite et autorisé du profil d'un lancement, figé au rejeu, sans modification de posture globale. Découverte reste le défaut. Développement ne signifie pas bypass général ; refuser les opérations hors droits déclarés.
- FR-02 : l'équipier communique via MCP sans devoir ouvrir la socket depuis son shell restreint. Le message reçu conserve l'émetteur UUID, l'identifiant intégral et la demande de réponse.
- FR-03 : attach rend les événements réels Codex `command` et `approval` avec un contenu neutralisé et borné. Les extensions inconnues restent explicitement inconnues.
- FR-04 : en terminal, une ligne de statut affiche les données de l'annuaire : identité, client/type, modèle, effort et état. Ne pas déduire le fournisseur commercial de Codex/Claude ou du nom du modèle. Source indisponible = inconnue, jamais maintien silencieux d'une valeur périmée.
- FR-05 : attach conserve la saisie (Entrée envoie, Ctrl-C quitte la vue), l'ordre du journal, ses lacunes et ses bornes. Aucun octet de contrôle injecté par les données ; sorties non-TTY stables hors correction explicite des événements.
- FR-06 : skill, aide et README distinguent MCP/CLI, géré/interactif, droits/saisie et persistance. Ajouter cancel en MCP seulement via le contrôle d'identité existant. Ne pas exposer de supervision MCP sans autorisation adéquate ; ne pas retirer Maicie/artefacts sans revue de compatibilité.

## Acceptation

- Agent réel : mission reçue, écriture dans le répertoire autorisé, réponse MCP liée, journal observable. Refus hors droits et posture globale inchangée.
- Rendu de fixtures extraites du journal réel : commande commencée/terminée, autorisation demandée/refusée, extension inconnue ; aucun message générique pour les événements connus.
- Statut : changement modèle/effort, données manquantes, déconnexion, neutralisation des contrôles et terminal étroit ; saisie préservée.
- Tests ciblés pendant le développement ; fmt/clippy et non-régression consolidée avant installation. Aucun redémarrage de production nécessaire aux tests.
