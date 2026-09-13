# Recette 093

Après installation seulement, quitter l'ancienne vue attach avec Ctrl-C puis :

```sh
bridget attach a4d12c75-5994-4c02-9acc-2db07ba817af
```

Demander une réponse avec titre, liste, emphase, citation et bloc Rust. Vérifier
le vrai rendu (pas uniquement la présence des bytes Markdown), aucune ligne
raisonnement/completed normale, source identifiée par label compact. Le code ne
doit pas être exécuté. Rétrécir/agrandir, y compris après fin de réponse et sans
frapper ; vérifier dernier bloc géré, brouillon, fond gris et statut dessous.

Une erreur et une lacune synthétiques sont vérifiées par PTY isolé, pas injectées
dans le daemon utilisateur. Non-TTY reste diagnostic. Ctrl-C quitte la vue seule.
Pas besoin de relancer daemon, agent ni session Codex pour adopter ce renderer.

L'historique ancien déjà confié au terminal suit son propre reflow ; Bridget ne
le redessine pas. L'édition au curseur, MCP rename/DND et SSH persistant ne sont
pas déclarés livrés avec cette recette.
