# 272 : annuler avant la création d’une tâche

Statut : actif dans les services 0.8.38 / 272. Le catalogue des sessions déjà ouvertes attend leur remontage natif ; François a choisi de garder T3 ouvert.

Une admission en cours doit pouvoir être annulée quand son brief est périmé.
Le champ kind=admission de bridget_cancel est explicite ; l’absence de kind
garde strictement la demande de message suivie historique. Aucun aiguillage
automatique selon une collision de clé.

La mutation native CancelAdmission exige la capacité admission_cancel_v1 du
maître. Le propriétaire vient de la connexion attestée. Le reçu Pending,
DecisionStarted ou Chosen devient Refused avec admission_cancelled, dans le CAS
SQLite existant. La création de Task et l’annulation sont sérialisées : si la
création a gagné, rendre son task_id et demander bridget_task_cancel sans
annulation implicite de cette tâche. Sinon le choix ou la préparation déjà
en cours peut finir, mais ne peut plus admettre de tâche pour cette clé.

Une clé non enregistrée ne crée pas de barrière contre un appel encore en
précontrôle. Le refus le dit explicitement ; ne jamais annoncer cancelled dans
ce cas. Une autre identité ne peut lire ni annuler ce reçu. Les contrôles de
lecture seule et les états terminaux restent prioritaires.

Aucun nouveau format de stockage, aucun effacement de dossier, aucune tâche
métier relancée. Les anciens pairs refusent la nouvelle opération avant envoi.
