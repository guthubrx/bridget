# Contrat d'affichage 093

- TTY conversation : en-tête compact distinct, corps sans UUID-indent, Markdown
  interprété ; raisonnements masqués et fins réussies silencieuses.
- Échecs/interruption/inconnu : visibles. Commandes, outils, refus, permissions,
  Gap/Unavailable/End : visibles. Aucune suppression des bytes journal.
- Non-TTY : projection diagnostique existante conservée, sans séquences ANSI.
- Couleur désactivée : contenu lisible, aucun SGR ; aucune perte de contenu.
- Corps externe : pas d'ANSI/OSC actif, pas de lien terminal exécutable, pas d'HTML
  exécuté, pas de ressource distante. Seuls styles constants du renderer autorisés.
- Resize : source brute conservée pour bloc actuellement géré, repli en cellules
  visibles ; nouveaux messages à la largeur nouvelle. Scrollback externalisé non
  garanti ; ne pas cloner tout le journal dans un second historique.
- Les noms de présentation ne servent jamais à une décision de routage.
