# Recette 158B — après implémentation seulement

Ces commandes décrivent le contrat candidat. Elles ne sont pas disponibles par le seul reçu 158A. Aucun exemple n'est exécuté ici. ROOT pilote baseline, tests et livraison dans un environnement isolé.

## Réveil local unitaire

```bash
bridget wake AGENT_CONNU --label 'Reprendre la vérification' --id rappel-unique-1
```

Relancer exactement la même commande relit le même reçu ; changer le libellé sous le même ID refuse. Le quota compte seulement les nouveaux IDs unitaires CLI. Aucun modèle hors ligne n'est lancé. La réponse préparée ou différée n'est pas une preuve d'exécution.

## Fil depuis un script headless

Exécuter depuis le contexte d'identité déjà attesté de la CLIthread. Le script hérite BRIDGET_AGENT_ID_FILE et BRIDGET_AGENT_INSTANCE_ID ; le fichier et le credential associé sont ceux existants, pas des valeurs créées pour contourner une garde. Vérifier l'appartenance et les droits de projet avant préparation. Sans ce contexte, l'appel refuse avant réservation.

```bash
bridget wake UUID_DU_FIL --label 'Point de reprise' --id rappel-fil-1
```

Le résultatcommitted signifie contribution+intentions enregistrées. L'auteur est le membre attesté. Les alertes des autresmembres suivent 102 ; le résultat ne promet pas qu'ils ont lu.

## Programmation MCP

bridget_wake schedule prend une cible self typée et une cadence interval de60 sminimum, ou une heure/jours dans le fuseauannoncé. Lister/annuler reste propriétaire seulement. Programmer vers agent/fil est Sends et peut être refusé par le plafondDiscovery.

## Faits manqués

bridget_events action=missed relit les faits propres conservés et les lacunesdatées. La lecture ne supprime rien. Revenir après DND ou se réinscrire peut créer un seul résumé de l'épisode. Son accusé 099 avance le cutoff, pas la simple lecture. Unknown ne prouve pas une lecturemodèle.

Les options MCP de pagination sont `limit` et `after_seq` ; le propriétaire
reste implicite. Schedule crée une nouvelle programmation : après réponse
perdue, relire list. Aucun null explicite ou champ owner/from/issued_at libre.
Les commandes d’administration humaine restent distinctes du déclencheur
headless ; la mutation exige un terminal et ControlStateV1. Cette recette
prévue ne prouve aucun test158B vert ou fonction déjà chargée.

## Vérification prévue

Faire prouver les dix groupes du plan et les trente oracles réemployés. Conserver les reçus, captures du protocole, hashessources et limites. Un test historiquePASS n'est pas un nouveau testB. Aucun secretUS8 ni HTTP. Les scripts de cleanup historiquesSIGKILL ne sont pas réutilisés.

## Fermetures finales du contrat ROOT

**Fuseau observé.** Le daemon relève le fingerprint de la zone effectivement rendue par libc (nom de zone et offset) au tick et avant le gel. Un changement observé recalcule seulement les échéances futures non gelées. FrozenWake et previous_calendar_date restent inchangés. Le résultat d'un calcul hors verrou porte son fingerprint ; le CAS refuse un résultat devenu périmé. Aucune promesse de zone figée ou de changement uniquement au restart. Les changements DST du fingerprint n'autorisent pas une seconde occurrence de la même date.

**Retour du worker.** Le canal de résultat utilise try_send, borne64. Si le canal est plein, aucun résultat sans preuve n'est inventé : le pendingfact durable garde sa provenance et sa deadline initiale immuable. Le tick solde un pending dépassant cette deadline en write_unknown, même si l'écriture a pu réussir ; il ne le qualifie jamais de non-écrit. Aucun once recréé. Si Store ne peut pas écrire ce résultat, garder la ligne pending et l'annonce agrégée de panne disponible, puis réessayer à la maintenance ; jamais annoncer une sauvegarde réussie. Le compteur global existant reste une preuve aggregate limitée, pas un reçu durable par fait.

**Cutoff.** Chaque fait porte un fact_seq SQLite AUTOINCREMENT durable et jamais réutilisé, en plus de son UUID de corrélation. Le cutoff du résumé capture ce fact_seq, plus owner_gap_version et global_gap_version séparés. Un fait arrivé après le gel au même observed_at reste au-delà du cutoff. ACK099 valide avance seulement les trois valeurs gelées ; aucune valeur plus récente n'est consommée. Pas triUUID ou horodatage en secondes seul pour ce seuil.

**Création et annulation.** schedule ne prend aucun command_id et n'est pas idempotent à la création. Chaque admission crée un schedule_id UUID neuf côté daemon. Un retry aveugle peut créer un second schedule ; après réponse perdue, relire list au lieu de promettre une création unique. FR103 protège les occurrences du schedule déjà créé, pas cette création. cancel prend seulement ce schedule_id canonique ; sur la même programmation connue déjà annulée, retourne already_cancelled sans mutation ni nouvelle émission. UUID inconnu refuse. Pas second registre de commandes et aucun command_id administratif gratuit. Les --id CLI unitaires conservent séparément la déduplication099 du contrat.

## Borne des programmations — décision ROOT

Limites physiques :128 lignes au total et16 par propriétaire, y compris les programmations annulées. Une admission peut retirer, dans sa transaction, les plus anciennes lignes annulées sans occurrence en attente ou en vol : d'abord celles du propriétaire si sa borne est atteinte, puis celles de la borne globale. Si aucune ligne n'est retirable, schedule_limit. Les preuves099/102 restent intactes. Cancel rend already_cancelled pour une ligne connue ; après retrait borné, son UUID est inconnu et refuse. Aucun historique perpétuel d'annulation.


## Transitions finales158B R1

Les règles finales de pause, départ099 distinct du commit102, unknown terminal, deadlineworker5s/budgetI/O1s, horizonCLI7jours, attribution du label et entretien des résumés sont définies dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/contracts/wake-158B.md. Elles priment les descriptions préparatoires ci-dessus. Aucun nouveau service, worker, registre ou scénario de confort.
