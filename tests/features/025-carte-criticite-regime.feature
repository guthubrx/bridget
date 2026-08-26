# language: fr
Fonctionnalité: Carte de criticité auto-élue et régime de revue
  Afin que le régime de revue ne dépende plus d'une liste choisie à la main
  Maicie doit mesurer le lot, proposer un régime et consigner toute décision

  Scénario: Une citation complète élit un seul homonyme
    Étant donné deux fichiers nommés "store.rs" dans le commit mesuré
    Et un Bloquant citant "plugins/maicie/src/store.rs"
    Quand la carte est calculée
    Alors elle contient exactement "plugins/maicie/src/store.rs"
    Et elle ne contient pas l'autre fichier "store.rs"

  Scénario: Une citation nue ambiguë n'élit aucun homonyme
    Étant donné deux fichiers nommés "main.rs" dans le commit mesuré
    Et un Bloquant citant "main.rs"
    Quand la carte est calculée
    Alors aucun fichier "main.rs" n'est élu par ce constat
    Et une ambiguïté durable liste les deux chemins complets

  Scénario: Le numéro de ligne départage un nom nu
    Étant donné deux fichiers nommés "store.rs" de longueurs différentes
    Et un Bloquant citant une ligne présente dans un seul candidat
    Quand la carte est calculée
    Alors un seul chemin complet est élu

  Plan du scénario: Chaque germe déclenche le jury sans registre
    Étant donné un registre vide
    Et un diff contenant une modification de type "<type>"
    Quand le régime est proposé
    Alors le régime proposé est "jury_1_plus_1"

    Exemples:
      | type                             |
      | schema_persistence               |
      | protocol_message                 |
      | authentication_permission        |
      | external_input_durable_write     |

  Scénario: Le worktree ne remplace pas les commits soumis
    Étant donné une base et une tête Git gelées
    Et un worktree contenant des modifications différentes
    Quand le lot est soumis
    Alors la carte correspond uniquement au diff entre la base et la tête

  Scénario: Le référent doit décider sans texte libre
    Étant donné une proposition durable "jury_1_plus_1"
    Quand le référent retient "revue_simple"
    Alors un écart "lightened" est enregistré sans champ de motif
    Et l'élection des relecteurs reste explicitement indisponible

  Scénario: Un refus est visible et compté
    Étant donné une référence Git courte
    Quand le lot est soumis
    Alors la soumission est refusée avec "branch_ref_not_full"
    Et une ligne durable augmente exactement ce compteur de refus

  Scénario: Les données sources ne fuient pas dans la greffe
    Étant donné un secret synthétique présent seulement dans le diff et un constat
    Quand la carte, la soumission et les métriques sont persistées
    Alors le secret est absent de la base et de toutes les sorties

  Scénario: Une fausse version précédente ne permet pas la migration v20
    Étant donné une base marquée v19 sans le DDL v19 attendu
    Quand la migration v20 est demandée
    Alors la migration est refusée
    Et l'empreinte bit à bit de la base reste identique
