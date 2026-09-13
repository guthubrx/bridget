# Données 093

Aucun changement persistant ni filaire.

- TurnBlock : clé session/message inchangée, en-tête, réponse brute, détails et
  bornes existantes. Dernier bloc éventuellement retenu pour reflow jusqu'au suivant.
- Vue Markdown : résultat dérivé transitoire ; styles locaux fermés, jamais une
  donnée autoritaire. L'UUID d'adressage reste indépendant du label de présentation.
- Géométrie : largeur/hauteur TTY constatées ; pas d'estimation depuis le texte.
- Présence : nom attesté existant, aucune requête supplémentaire par événement.

Cycle : recevoir → conserver brut borné → projeter à largeur courante → rendre ;
fin réussie = clôture du bloc sans texte technique ; erreur = clôture visible ;
resize = nouvelle projection, jamais nouvel événement ni nouvel envoi.
