# Recherche 093

2026-09-06 — décision : conserver le renderer existant et ne remplacer que la
présentation des blocs. Capture humaine comme preuve du défaut de rendu.

Existant : TurnBlock.response conserve déjà le Markdown brut borné ; .lines()
appelle render_prefixed_with_limit, responsable de l'indentation de continuation
selon la longueur du préfixe. Ce n'est pas nécessairement une faute LF brute :
write_terminal_lines utilise déjà CRLF. Le test devra discriminer les deux.

Markdown : aucune dépendance présente dans Cargo.toml/Cargo.lock.
- pulldown-cmark : parser CommonMark en événements, approprié au bloc borné,
  n'impose ni terminal ni canal. https://docs.rs/crate/pulldown-cmark/latest
- termimad : parsing/style/largeur intégrés, mais amène crossterm et une mécanique
  de disposition voisine du renderer actuel. Non retenu pour éviter deux autorités
  de géométrie. https://docs.rs/termimad/latest/termimad/struct.MadSkin.html
- Parseur maison : refusé, maintenance inutile de fences/emphase/fragmentation.
- Syntect : non retenu à ce stade ; le besoin présent est Markdown et code distinct
  visuellement, pas une grammaire de coloration lexicale de tous langages.

Résolution Cargo effective : pulldown-cmark 0.13.4, default-features=false ;
seule autre entrée nouvelle du lockfile : unicase 2.9.0. bitflags et memchr
étaient déjà présents ; aucune montée de version des dépendances existantes.
Le texte doit être neutralisé aussi APRÈS décodage des entités CommonMark :
assainir uniquement le Markdown brut laisserait un contrôle recréé par le parser.

Charge cognitive : baseline utilisateur consultée, aucune métrique de productivité
inventée. L'oracle est la capture et les critères visuels précis de la spec.
Tests : réemploi PseudoTerminal existant, aucun framework supplémentaire.

Resize : refresh_geometry existe mais n'est pas actuellement un redessin autonome
garanti sans commande. Le texte engagé au scrollback ne peut être réédité par
Bridget sans changer d'architecture ; limite assumée. Le bloc actif, puis un unique
dernier bloc terminé tant qu'il reste dans la zone gérée, sont retenus pour reflow.
Une frontière de débordement ne peut pas être un index de lignes physiques dépendant
de la largeur : les tests doivent couvrir aussi un bloc dépassant un écran court.

Flux : O(n) par parsing d'un bloc borné ne signifie pas O(n) pour toute la réponse.
Avec F fragments, un parsing systématique coûte O(F × n). La livraison doit donc
soit borner explicitement la cadence des parses/redessins en conservant un flush
terminal immédiat, soit produire une mesure discriminante sur des milliers de
fragments montrant le respect du budget. apply_batch seul ne prouve pas cette borne.
