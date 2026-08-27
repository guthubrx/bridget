# Déplacer le référent, le service et le registre

Procédure éprouvée le 27/08/2026 dans le sens poste → machine distante.
Le sens inverse n'a **pas** été exécuté : la procédure est symétrique par construction,
mais elle reste **non vérifiée**.

## Ce qui se déplace

Trois choses distinctes, souvent confondues :

- **le service** — le programme permanent qui tient la liste des agents, transporte les
  messages et détient la base. Point de passage obligé ;
- **le registre** — deux fichiers : la base des missions et le catalogue des constats.
  C'est la mémoire, elle ne « sait » rien : c'est la configuration qui la désigne ;
- **le référent** — un agent comme les autres, qui route, arbitre et intègre.

Les agents ne sont **pas** prévenus et n'ont rien à savoir : ils écrivent à un **nom**,
le service route. Le référent doit donc se réenregistrer sous le **même nom**.

## Ordre des opérations

1. **Compiler à la source** sur la machine d'arrivée, depuis la référence commune à jour.
   Installer le binaire *et* l'outil de registre.
2. **Arrêter le service** de départ. Sans cela les deux copies du registre divergent
   pendant la copie, et plus rien ne dit laquelle fait foi.
   Le service peut résister au signal poli tant qu'il porte une version antérieure au
   correctif de terminaison : le signal fort est alors légitime.
3. **Copier les deux fichiers du registre**, puis **vérifier les tailles des deux côtés**.
4. **Vérifier le chemin visé par la configuration d'arrivée.** Piège mesuré : elle peut
   pointer vers une base **vide** préexistante. Copier au chemin configuré, pas au chemin
   habituel.
5. **Migrer le schéma** si l'outil le demande. Sauvegarder avant.
6. **Vérifier le compte** : nombre d'objectifs et de constats non nuls et conformes.
7. **Lancer le service** sur la machine d'arrivée. Les agents s'y reconnectent seuls
   s'ils visent le même point d'entrée local.
8. **Inverser le tunnel** : il publiait le point d'entrée du départ ; il doit désormais
   donner accès depuis le départ vers l'arrivée.
9. **Lancer le référent** sous le **même nom qu'avant**, en mode flux.
10. **Transmettre le contexte** : copier le fichier de session et envoyer un mandat qui
    en donne le chemin, avec l'instruction de le lire **par la fin** — il pèse plus de
    150 Mo et ne tient pas en entier.
11. **Déplacer les tâches périodiques.** La ronde doit tourner là où sont le service et
    le référent.

## Le piège de la ronde

Le minuteur qui lance le constat mécanique ne **traite** rien — sa propre sortie le dit :
« constat seulement : aucune décision ni aucun envoi ». Sans un second geste qui **réveille
le référent**, le tour de garde n'a jamais lieu.

La tâche périodique doit donc faire les deux : lancer le constat, puis envoyer au référent
le mandat de ronde.

## Pièges rencontrés, tous mesurés

- **Le mode flux refuse les arguments d'agent** : il ne peut donc pas reprendre un contexte.
  Flux *ou* mémoire — le contexte se transmet alors par message.
- **Le mode flux échoue à lancer l'agent** s'il manque une entrée dans la configuration des
  agents, là où un autre pilote en a une. L'erreur ne nomme pas la cause.
- **La configuration du registre peut viser une base vide.** Vérifier le chemin, pas le nom.
- **`setsid` n'existe pas sur macOS** ; utiliser l'option de détachement de `ssh`.
- **Le programme local ignore un point d'entrée passé par variable d'environnement** et
  cherche l'emplacement par défaut. Non résolu : passer par un accès distant direct.
