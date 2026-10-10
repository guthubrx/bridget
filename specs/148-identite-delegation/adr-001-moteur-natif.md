# Décision148 — moteur Bridget autonome

Statut : retenu pour l'implémentation. Date : 2026-10-10.

## Pourquoi

La délégation doit fonctionner depuis n'importe quel hôte Bridget. Utiliser
l'orchestrateur T3 pour créer les enfants imposerait T3 au produit. L'utilisateur
a explicitement exclu cette dépendance.

## Décision

Le daemon Bridget porte le catalogue, la sélection exacte, les tâches, les
étapes de lancement et remise, les résultats et l'annulation. Il réutilise son
registre, sa flotte gérée, ses remises idempotentes et ses exécutions existantes.
Les références et la définition fournisseur sont conservées avant les effets.

Le connecteur T3 atteste seulement la session courante. Il fournit au MCP Bridget
un credential privé lié au fil. Bridget le vérifie à chaque opération. Une
preuve invalide ou un rattachement daemon révoqué produit un refus fermé.
Les processus partagés ne servent plus à choisir la session de ce connecteur.
Les identités de wrappers natifs conservent leur mécanisme existant.

Les permissions de délégation sont des permissions natives Bridget. Un parent
géré peut transmettre ses limites prouvées. Un parent externe avec projet attesté
peut demander une inspection dans ce même arbre. Pour accorder plus de droits,
le contrôle humain Bridget conserve une politique liée à son instance et au
dossier autorisé. Une révocation explicite ferme aussi l'héritage automatique.
La commande humaine ne devient pas un outil d'agent. Un choix explicite ne peut
ni inventer un modèle ni élargir les permissions du parent. Le catalogue affiche
les postures réellement utilisables et les raisons d'un refus.

Les revues et tests GLM lancés par T3 sont un choix de validation de cette
session. Ils ne constituent pas une dépendance d'exécution de Bridget.

## Conséquences à vérifier

- La recette du moteur natif se déroule avec T3 absent.
- Un rejeu conserve l'enfant et la mission initiaux, même après coupure.
- Une fin de tour avec enfants actifs ne publie pas le résultat final.
- La révocation ferme les nouveaux accès ; lire un résultat ne relance rien.
- Aucun jeton T3 ne rejoint le prompt, la ligne de commande ou les journaux.
- Aucun processus de production n'est redémarré pendant cette session.
