# Archives techniques de l’adaptateur T3

Document d’origine : [specs/155-demandes-enfant/research.md](../../../../../../specs/155-demandes-enfant/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Source : lecture du checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`), lecture seule, et du
code T3 `origin/main` par `git show`. Ce checkpoint historique supposait150 livrée ; les acquis150–154 sont désormais livrés. Le passageR16 ci-dessous remplace les hypothèses fournisseur. Les
lignes citées sont celles du checkout principal.

<a id="extrait-2"></a>

## Extrait 2

- T3 : `t3_pending_request_list/read/respond` ne couvre que `user_input`. Les
  approbations restent à l'humain.
- Bridget 155 : questions au parent, approbations à l'humain dans une limite absolue
  vérifiée, issue explicite. Un point T3 reste meilleur : son affichage dans une
  interface. Il est hors périmètre ici.
