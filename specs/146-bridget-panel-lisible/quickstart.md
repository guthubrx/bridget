# Recette isolée — SPEC146

Date : 2026-10-08. Guide de contrôle, pas preuve d'exécution.

## Préparation

Utiliser les worktrees146 et le socle145 identifié par le manifeste d'import. Ne pas démarrer un serveur contre la base T3 réelle. Ne pas relancer l'application installée ou Bridget actif.

Préparer des données de test : plus d'une page de fils ; un ancien fil au dernier échange le plus récent ; un fil vide ; des activités égales ; trois pages de messages ; des auteurs distincts ; les quatre types ; un remplacement ; des corps courts et longs avec lignes blanches et Unicode.

## Vérifications fonctionnelles

1. Ouvrir le panneau. Le fil au dernier échange le plus récent apparaît d'abord. Les dates courtes correspondent à l'ordre. Le fil vide indique sa création.
2. Ouvrir le fil multipage. La première séquence est la dernière réelle. Charger les messages plus anciens ; ils suivent en ordre décroissant sans trou ni doublon.
3. Ajouter un message à la fixture entre deux pages. L'instantané existant reste inchangé. Rafraîchir pour voir le nouveau message d'abord.
4. Observer un corps long. Son aperçu occupe au plus quatre lignes. Déplier puis replier. La copie restitue toujours l'original complet.
5. Chercher un mot uniquement présent dans la portion repliée chargée. Le message est retrouvé sans appel distant.
6. Ouvrir les détails. Type français, séquence et remplacement correspondent à la fixture. Les auteurs sont visibles sans ouvrir ces détails.
7. Tester largeur étroite et navigation clavier. Tous les contrôles restent utilisables, sans chevauchement ni focus perdu.
8. Tester changement A → B → A, fermeture, révocation, daemon absent et version ancienne. Aucun ancien contenu ne réapparaît ; le refus ne devient pas un accès élargi.
9. Comparer les données métier et les appels observés. Zéro ACK, émission, réveil et modèle. Aucune avance de curseur agent ni modification de mission.

## Preuves à consigner

Enregistrer les commandes effectivement exécutées, leurs sorties, la source testée et les limites. Conserver les constats RED puis GREEN sans les remplacer par des succès historiques145. Une capture d'aperçu peut montrer le rendu ; elle ne prouve pas l'installation en production.

La recette utilise les composants réels dans un aperçu isolé et des données de test. Aucun résultat de redémarrage, commit, fusion, push ou installation n'est revendiqué par ce guide.
