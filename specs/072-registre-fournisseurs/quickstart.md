# Vérification opérateur - SPEC-072

## Lire la provenance

Dans l interface Bridget, sélectionner un agent déjà actif. Son identité runtime est distincte du fournisseur sélectionné : un agent GLM doit indiquer GLM comme provenance et Claude Code comme runtime lorsqu il est affiché par la SPEC-071.

## Tester Cursor

1. Choisir l agent Cursor géré.
2. Envoyer une question sans écriture, par exemple donne une phrase de test.
3. Observer la remise, le début d activité, les actions puis la réponse.
4. Ouvrir le détail d activité et vérifier cursor et acp.

## Tester GLM et DeepSeek

1. Choisir successivement les agents GLM puis DeepSeek.
2. Leur demander de lire un fichier sans le modifier puis de résumer la lecture.
3. Vérifier que chaque retour conserve son fournisseur et que les outils sont journalisés en temps réel.
4. Vérifier dans une autre conversation que l agent Anthropic répond toujours via Anthropic et non via l endpoint précédent.

## Tester un échec explicite

Un profil absent, illisible ou sans identifiant ne doit ni démarrer un tour ni répondre comme s il utilisait Anthropic. L interface doit montrer le motif de configuration fourni par Bridget.

## Limites assumées

Un runtime Cursor peut employer un modèle édité par une autre entreprise. Le fournisseur affiché est alors Cursor, car c est le produit et le compte que Bridget a réellement lancé. GLM et DeepSeek sont différents : ils sont des upstreams sélectionnés explicitement pour Claude Code.
