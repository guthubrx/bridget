Fonctionnalité: Projet système Bridget et dogfooding expert

  Scénario: Un seul projet système est admis
    Étant donné un projet système Bridget déjà déclaré
    Quand un second projet demande ce rôle
    Alors le registre refuse sans modifier les deux liaisons

  Scénario: Le mode expert est désactivé par défaut
    Étant donné le projet système déclaré
    Quand aucun réglage expert n'a été confirmé
    Alors aucun agent Docker ne peut écrire dans un worktree Bridget

  Scénario: Le checkout principal reste toujours lecture seule
    Étant donné le mode expert activé
    Quand un agent rejoint le runtime du projet système
    Alors le checkout principal Bridget est monté en lecture seule

  Scénario: Un worktree non-main attribué est le seul chemin écrivable
    Étant donné un worktree lié sur une branche autre que main
    Quand une lease est attribuée à un agent système
    Alors ce worktree est monté en écriture pour cet agent uniquement

  Scénario: Une attribution concurrente est refusée
    Étant donné une lease active sur un worktree
    Quand un second agent demande ce même worktree
    Alors le daemon refuse sans démarrer le second agent

  Scénario: Le mode expert ne livre rien automatiquement
    Étant donné un agent système autorisé à modifier son worktree
    Quand son travail se termine
    Alors aucun merge, push, installation, redémarrage ou déploiement n'est déclenché
