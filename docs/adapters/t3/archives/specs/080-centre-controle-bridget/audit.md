# Archives techniques de l’adaptateur T3

Document d’origine : [specs/080-centre-controle-bridget/audit.md](../../../../../../specs/080-centre-controle-bridget/audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- PASS : l'accès se fait par une roue seule en bas de la barre d'agents et ouvre un overlay, pas un panneau de détails de conversation.
- PASS : la navigation et la recherche couvrent les huit rubriques annoncées.
- PASS : les préférences de présentation sont locales, normalisées et bornées. Les tests couvrent le stockage corrompu, le fuseau IANA, les polices et tailles invalides.
- PASS : la page serveur conserve le catalogue fermé, la prévisualisation, la confirmation et la génération attendue de la politique de racines.
- PASS : la page Usage distingue les jetons observés d'une estimation API indisponible.
- PASS : les champs de la page Typographie utilisent la grille, les dimensions, les libellés, les sous-titres et les états de focus portés de T3 Code, tout en conservant le fond Bridget et l'overlay Bridget.
