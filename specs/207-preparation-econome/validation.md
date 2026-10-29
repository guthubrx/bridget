# Validation207 — en cours

Source initiale c79bbc60 : 5 tests ciblés et26 tests Git PASS. Source3ff8be65 :6 tests207 PASS. Revue7d2f4890 : trois P2 corrigés dans3ff8be65 (full-tree, manifeste, codes E/S). Revuea9757f7d confirme les trois corrections. Son P2 sur1Mio est infirmé par le puits capture : stdout de ls-tree et ls-files dispose déjà de8Mio, autres commandes1Mio ; une vraie extraction native a passé sur le dépôt Opus2D de près de20k fichiers. Le plafond reste borné, aucune sortie tronquée acceptée.

La suite élargie sur3ff8be65 a trouvé une régression : lire un marqueur absent avec config --worktree échoue sur un ancien dépôt sans extension worktreeConfig. La lecture utilise maintenant config --no-includes --get, qui consulte aussi la configuration du worktree lorsqu'elle est activée et accepte l'absence sur un ancien dépôt. Les réglages ne sont pas modifiés. Le contrôle de drapeaux passe avant l'absence de manifeste ; un ancien assume-unchanged peut désormais produire rapport indisponible plutôt qu'un faux propre. Aucun fichier de ces dossiers n'est converti.

Recette manuelle native sur un clone privé Opus2D sous le cache : SHA ab9e77f0c57d428aa31029cce58c09cbced1c01d,350243993octets sélectionnés sur9923953231 ;387444Kio alloués,5,87s. Aucun Task, modèle ou service métier. Les deux premiers essais privés ont été refusés avant création car le clone sans checkout n'avait pas la règle d'exclusion locale .worktrees ; la recette a ensuite fourni cette règle privée, sans changer le projet original.

Aucun chargement207 à ce stade. Binairesc79/3ff sont candidats, pas une preuve de runtime. La mesure finale, les hachages des binaires chargés, la configuration et le nettoyage seront ajoutés après vérification.
