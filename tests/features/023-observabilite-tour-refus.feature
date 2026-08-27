# language: fr
Fonctionnalité: Expliquer l'inactivité et les refus

  Scénario: Un tour terminé sans reprise ne paraît plus au travail
    Étant donné une délégation ouverte et une présence vivante
    Et un dernier tour terminé sans ouverture ultérieure
    Quand la ronde classe l'agent
    Alors elle nomme la fin sans reprise
    Et elle ne le classe pas occupé

  Scénario: Un long tour ouvert reste légitime
    Étant donné une délégation ouverte depuis plus d'une heure
    Et un transport qui consigne aussi les bornes terminales
    Et un dernier tour ouvert sans borne terminale
    Quand la ronde classe l'agent
    Alors elle le classe occupé

  Scénario: Une source qui ne ferme jamais ses tours ne prouve rien
    Étant donné un transport interactif qui ne consigne que les ouvertures
    Et un état daemon qui ne vaut pas busy
    Quand la ronde classe l'agent
    Alors elle le classe indéterminé avec la condition de source

  Scénario: Un refus de cible nomme sa condition
    Étant donné une cible dont l'état vaut busy
    Quand la délégation est refusée
    Alors le message rendu nomme state=busy
    Et il ne cite pas le domaine comme cause
