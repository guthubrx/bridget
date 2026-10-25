# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/adr-001-moteur-natif.md](../../../../../../specs/148-identite-delegation/adr-001-moteur-natif.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La délégation doit fonctionner depuis n'importe quel hôte Bridget. Utiliser
l'orchestrateur T3 pour créer les enfants imposerait T3 au produit. L'utilisateur
a explicitement exclu cette dépendance.

<a id="extrait-2"></a>

## Extrait 2

Le connecteur T3 atteste seulement la session courante. Il fournit au MCP Bridget
un credential privé lié au fil. Bridget le vérifie à chaque opération. Une
preuve invalide ou un rattachement daemon révoqué produit un refus fermé.
Les processus partagés ne servent plus à choisir la session de ce connecteur.
Les identités de wrappers natifs conservent leur mécanisme existant.

<a id="extrait-3"></a>

## Extrait 3

Les revues et tests GLM lancés par T3 sont un choix de validation de cette
session. Ils ne constituent pas une dépendance d'exécution de Bridget.

<a id="extrait-4"></a>

## Extrait 4

- La recette du moteur natif se déroule avec dans un environnement natif isolé.
- Un rejeu conserve l'enfant et la mission initiaux, même après coupure.
- Une fin de tour avec enfants actifs ne publie pas le résultat final.
- La révocation ferme les nouveaux accès ; lire un résultat ne relance rien.
- Aucun jeton T3 ne rejoint le prompt, la ligne de commande ou les journaux.
- Aucun processus de production n'est redémarré pendant cette session.
