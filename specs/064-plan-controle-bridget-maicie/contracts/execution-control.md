# Contrat conceptuel du plan de contrôle

## Objet

Définir les commandes et événements neutres entre clients Bridget, daemon et
adaptateurs sans imposer la forme d'un fournisseur particulier.

## Commandes

### SubmitWork

Préconditions :

- identité et portée de l'émetteur validées ;
- destinataire logique connu ;
- intention reconnue ;
- identifiant et octets canoniques stables.

Résultat :

- `Accepted` avec `submission_id` et décision d'admission ;
- `Rejected` avec raison structurée ;
- `OutcomeUnknown` rejouable avec la même identité.

### ControlExecution

Actions :

- `SteerCurrent` ;
- `Interrupt` ;
- `PauseQueue` ;
- `ResumeQueue` ;
- `CancelQueued`.

Préconditions : exécution, génération, autorité et capacité corrélées.

### ResumeExecution

Modes demandables : `NativeResume`, `NativeFork`, `Reconstruct`.

Le résultat indique toujours le mode effectivement utilisé. Aucun repli n'est
silencieux.

### WaitExecutions

Attend un ensemble borné d'exécutions et se réveille sur changement d'état,
message ou échéance. L'attente ne réalise aucun polling fournisseur caché.

## Événements

- `SubmissionAdmitted`
- `DeliveryPhaseChanged`
- `MessageVisible`
- `ExecutionStateChanged`
- `ApprovalRequested`
- `ApprovalResolved`
- `ApprovalLoopDetected`
- `ProviderCapabilityObserved`
- `AgentLinkChanged`
- `UsageUpdated`
- `ProjectionGapDetected`

Chaque événement contient :

- identifiant stable ;
- séquence ou curseur ;
- source et génération ;
- identifiants de corrélation disponibles ;
- horodatage ;
- raison structurée ;
- preuve brute référencée ou incorporée selon la politique.

## Idempotence

- Rejouer la même commande avec la même identité et les mêmes octets retourne
  la même décision publique.
- La même identité avec des octets différents est refusée.
- Une expiration d'idempotence est terminale et ne crée pas une nouvelle
  identité implicite.
- Un événement déjà appliqué au même curseur est un no-op observable.

## Compatibilité

- Les champs ajoutés pendant migration sont optionnels pour les anciens
  producteurs et explicitement `unknown` dans les nouvelles projections.
- Une version inconnue est refusée en écriture.
- Aucun consommateur ne déduit une valeur absente à partir d'un autre champ
  seulement parce que deux fixtures historiques utilisaient la même valeur.
