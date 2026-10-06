# language: fr
Fonctionnalité: Conserver l'histoire et transmettre les consignes utiles

  @spec136_history_silent_and_exact
  Scénario: Soixante-quatre comptes rendus ne réveillent personne
    Étant donné un fil partagé entre A et B dans un daemon isolé
    Quand A publie 64 comptes rendus de classe historique sans cible
    Alors aucun membre n'est sollicité
    Et la lecture de travail ne recopie aucun compte rendu
    Et l'historique permet de récupérer les 64 corps exacts

  @spec136_supersession_keeps_only_current_bodies
  Scénario: Les anciennes consignes restent des preuves, pas des ordres
    Étant donné une action de A pour B et un blocage indépendant
    Quand A remplace explicitement son action dix fois pour B
    Alors la lecture ne transmet que le dernier corps d'action et le blocage
    Et les dix anciennes actions restent des références relisibles

  @spec136_supersession_refusals_are_atomic
  Scénario: Une correction illégitime ne masque aucun travail
    Étant donné une action courante de A pour B
    Quand C tente de la remplacer ou A change l'audience
    Alors aucun dépôt ni confirmation ni sollicitation n'est modifié

  @spec136_receipt_snapshot_survives_later_replacement
  Scénario: La lecture rejouée ne saute pas une correction concurrente
    Étant donné une page non confirmée de B
    Quand A remplace son action après cette lecture
    Alors le rejeu de la page est identique
    Et après confirmation la correction reste nouvelle pour B

  @spec136_v1_schema_migration_preserves_legacy_and_is_repeatable
  @spec136_structured_history_survives_daemon_restart
  Scénario: Anciennes preuves et nouvelles relations survivent à la reprise
    Étant donné une base ancienne et un dépôt sans classe
    Quand le daemon migre puis redémarre après une correction structurée
    Alors les anciens corps restent exacts
    Et la correction et son lien restent vérifiables
