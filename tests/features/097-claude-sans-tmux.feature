# language: fr
Fonctionnalité: Joindre un agent Claude Code sans tmux
  Un agent Claude, géré par le daemon ou lancé par l'humain dans son terminal,
  reçoit réellement les messages Bridget ; aucun agent n'est affiché joignable
  s'il ne l'est pas.

  Scénario: Un Claude géré répond à une demande suivie sur le compte local
    Étant donné un Claude Code connecté au compte de l'humain sur cette machine
    Et un équipier Claude géré lancé par le daemon avec HOME et USER hérités
    Quand un autre agent lui envoie une demande suivie
    Alors la demande devient "answered" avec l'identifiant exact
    Et le journal attachable contient la mission et la réponse
    Et l'arrêt ne laisse aucun processus survivant

  Scénario: Une session interactive lancée dans iTerm reçoit un message
    Étant donné "bridget claude" lancé dans un terminal sans serveur tmux
    Quand un agent Codex lui envoie une demande suivie
    Alors le message apparaît dans la conversation Claude
    Et la réponse liée clôt la demande au ledger

  Scénario: Saisie humaine préservée pendant une remise
    Étant donné une saisie humaine partielle dans le composer de Claude
    Quand un message Bridget arrive
    Alors le message est collé sans détruire la saisie
    Et l'accusé de remise n'est émis qu'après écriture effective

  Scénario: Présence honnête sans voie de remise
    Étant donné un wrapper interactif sans terminal ni pane utilisable
    Quand l'humain tente de le lancer
    Alors Bridget refuse avec un diagnostic et l'alternative "bridget spawn"
    Et l'annuaire ne montre jamais "tmux" pour un agent sans pane

  Scénario: Fermeture et perte du daemon
    Étant donné une session Claude interactive joignable
    Quand Claude se termine ou que le daemon redémarre
    Alors le terminal est restauré et aucune présence fausse ne subsiste
    Et l'identité est identique après reconnexion
