# Quickstart de validation - SPEC-083

Cette recette s'exécute après SPEC-082. Elle doit être jouée sur Bridget local
et sur une instance Bridget serveur, avec Bridget Desktop macOS.

## Préparation

1. Publier un artefact HTML légitime avec des données locales et une source
   attestée par Bridget.
2. Préparer une fixture HTML hostile qui tente `fetch`, WebSocket, iframe,
   ouverture de fenêtre, formulaire, `top.location`, accès parent et message
   hors contrat.
3. Préparer une page HTTPS de test et un contenu HTML publié localement.

## Parcours P1 - Sandbox

1. Ouvrir l'artefact dans le fil : il s'affiche inline et ne dépasse pas
   1 200 px.
2. Utiliser un filtre local : l'interaction répond sans appel réseau.
3. Demander l'ouverture d'une source : aucune navigation n'arrive avant le
   geste opérateur ; après geste, Browser ouvre la destination validée.
4. Essayer la fixture hostile : chaque sortie est bloquée et un état explicite
   est visible sans crash de conversation.
5. Choisir Enregistrer comme nouvelle version : la version originale reste
   inchangée et la nouvelle porte son lien parent.

## Parcours P1 - Browser et panneau

1. Cliquer le toggle droit : le panneau s'affiche et se masque sans perdre la
   conversation.
2. Cliquer agrandir : l'icône devient restaurer et le Browser conserve sa page.
3. Ouvrir successivement une version d'artefact, un contenu publié, puis une
   URL HTTPS volontaire.
4. Contrôler les onglets Artefacts, Fichiers, Liens et Activité sur le projet
   actif, puis déclencher une recherche globale explicite.
5. Effacer les données Browser depuis Paramètres, redémarrer l'application et
   vérifier que le profil est vide tandis que les artefacts restent accessibles.

## Critères de sortie

- Aucun contenu HTML ne peut joindre le réseau ou l'API Desktop.
- Aucun agent ne voit les données Browser, même après authentification humaine.
- Aucun échec final n'est un placeholder muet.
- Les actions de panneau sont accessibles au clavier et leurs libellés décrivent
  leur état.
- Les traces de collectes, restaurations et créations de version sont dans
  Bridget, pas uniquement dans le navigateur.
