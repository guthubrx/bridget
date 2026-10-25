# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/spec.md](../../../../../../specs/148-identite-delegation/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'utilisateur veut déléguer une mission à un modèle choisi en une opération,
recevoir son résultat et ne plus rencontrer de refus d'identité lorsque T3 partage
un processus fournisseur entre plusieurs conversations. Les échanges et leur
suivi restent explicites dans Bridget. Bridget fournit lui-même le lancement,
le catalogue, les tâches, le résultat et l'annulation. La présence de T3 n'est
jamais une condition. Le connecteur T3 adapte seulement l'identité et l'interface.
La livraison ne redémarre aucune application ni aucun service existant.

<a id="extrait-2"></a>

## Extrait 2

Deux conversations Codex utilisent le même processus T3. Chacune consulte son
annuaire et envoie un message sous sa propre identité. La fermeture d'une session
révoque son accès sans retirer celui de l'autre. Une preuve absente, étrangère,
expirée ou révoquée produit un refus explicite, sans emprunter une autre identité.

<a id="extrait-3"></a>

## Extrait 3

- FR001 : attester la session appelante sans identité choisie dans les arguments métier.
- FR002 : séparer deux conversations actives dans un même processus fournisseur.
- FR003 : refuser une preuve invalide sans repli vers une identité déduite d'un processus.
- FR004 : vérifier la preuve à chaque appel et conserver la révocation T3.
- FR005 : vérifier le rattachement vivant Bridget avant toute opération sous cette identité.
- FR006 : conserver les identités natives, déléguées et les gardes de projet existantes.
- FR007 : exposer une opération de délégation avec mission et sélection explicite.
- FR008 : remettre la mission par Bridget ; une création T3 seule n'est pas une remise Bridget.
- FR009 : conserver les identifiants de tâche, enfant et message dès leur acceptation.
- FR010 : réutiliser les clés et enveloppes pour tout rejeu après coupure.
- FR011 : consulter le catalogue réel du moteur utilisé et ses raisons de refus.
- FR012 : interdire toute substitution de fournisseur/modèle demandé.
- FR013 : borner les droits de l'enfant par ceux de son parent.
- FR014 : préserver les choix humains de désactivation MCP et les configurations explicites.
- FR015 : remettre automatiquement un résultat corrélé au parent une seule fois.
- FR016 : distinguer travail actif, attente des enfants, résultat, échec et annulation.
- FR017 : ne pas confondre fin de tour et fin de mission.
- FR018 : consulter et annuler uniquement une tâche appartenant au parent.
- FR019 : annuler les descendants actifs avec les gardes de permissions existantes.
- FR020 : transmettre la mission, les références choisies et les faits natifs
  d'identité et de Git utiles à la reprise ; jamais la conversation du parent.
- FR021 : ne pas demander d'UUID, de fichier de suivi ou de commande de lancement à l'humain.
- FR022 : borner réseau, tailles, ressources et attentes ; ne jamais journaliser les jetons.
- FR023 : garder visibles les refus et les étapes acceptées d'une opération partielle.
- FR024 : conserver les processus et configurations de production durant le développement.
- FR025 : proposer les mêmes capacités de délégation avec et par le parcours natif Bridget ; aucun
  appel à delegate_task, au catalogue ou au scheduler T3 dans le moteur Bridget.
- FR026 : limiter les adaptations T3 à son connecteur ; une panne T3 ne doit
  pas interrompre les délégations natives ni masquer le catalogue Bridget.
