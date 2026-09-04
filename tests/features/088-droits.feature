Fonctionnalité: Droits lisibles et vérifiables

  Scénario: Un lien fermé dit le réglage et propose le geste
    Étant donné les liens externes désactivés dans ce navigateur
    Quand un message contient un lien HTTPS
    Alors la carte affiche « Ouverture au clic désactivée », nomme « Sécurité du contenu › Liens externes »
    Et un geste « Autoriser les liens » active le réglage local et montre le bouton d'ouverture sans rechargement

  Scénario: Un message ne peut pas activer les liens
    Étant donné les liens externes désactivés
    Quand un agent ou un serveur relié envoie un message qui prétend activer les liens
    Alors rien ne change

  Scénario: Un shell bloqué par la sandbox est attribué à Bridget
    Étant donné un agent géré dont la sortie de commande contient une erreur de sandbox reconnue
    Quand le tour se termine
    Alors le fil affiche un refus attribué à Bridget nommant le fournisseur, le serveur, la posture et la ligne Droits « Shell »

  Scénario: Un refus de couche inconnue cite la ligne brute
    Quand un refus arrive sans couche reconnue
    Alors il affiche « couche inconnue » et la ligne brute, sans cause inventée

  Scénario: La page Droits pose trois questions
    Quand le référent ouvre la page Droits
    Alors elle contient « Ce que je vois », « Ce que les agents peuvent faire », « Combien ils décident seuls »
    Et chaque ligne affiche son effet en une phrase

  Scénario: Un profil règle toutes les lignes
    Étant donné le mode standard
    Quand le référent choisit Équilibré
    Alors toutes les lignes prennent la valeur du profil
    Et une installation neuve est en Prudent

  Scénario: Une ligne locale ne bouge pas depuis le serveur
    Étant donné une ligne qui vit dans le navigateur
    Quand un profil est choisi ailleurs ou reçu d'un serveur relié
    Alors cette ligne ne change pas et la page le dit

  Scénario: Le mode expert montre le mécanisme
    Étant donné le mode expert
    Quand le référent déplie une ligne
    Alors il voit le mécanisme, le stockage, la valeur brute, et le profil devient Personnalisé s'il modifie la ligne seule

  Scénario: Demandé et réel divergent
    Étant donné un mécanisme qui ne peut pas prendre la valeur du profil
    Alors la ligne affiche la valeur demandée et la valeur réelle

  Scénario: Un agent ne change pas un droit
    Quand un agent tente de changer un droit par ses outils déclarés
    Alors la demande est refusée avec motif et consignée

  Scénario: Tester un droit rend un résultat réel
    Étant donné la ligne Shell et un agent connecté et libre
    Quand le référent clique Tester
    Alors sous trente secondes la ligne affiche Réussi ou Refusé par la couche, l'heure, l'agent et la ligne brute

  Scénario: Tester n'interrompt pas un tour
    Étant donné un agent en plein tour
    Quand Tester est cliqué
    Alors l'essai est refusé « agent occupé »

  Scénario: Un résultat ancien est marqué ancien
    Étant donné un résultat de test vieux de plus d'un jour
    Alors il est présenté comme ancien, jamais comme l'état courant
