# Modèle de données — conserver les autorités, ne pas inventer un nouveau schéma

Ce document décrit les invariants à préserver. Les noms SQL, versions et octets filaires exacts seront épinglés depuis la base Git par T001/T002 ; aucune migration de production n'est définie ni exécutée ici.

| Entité | Identité et faits durables | Autorité |
|---|---|---|
| Agent | Identifiant d'instance stable, nom affiché mutable, génération de processus ; source et instant des faits de présence | Enregistrement validé par le daemon ; jamais le texte d'un prompt |
| Message | ID, émetteur et destinataire, corps exact, reply, in_reply_to et paramètres temporels | Enveloppe canonique acceptée |
| Record d'idempotence | Scope stable + ID, issued_at immuable, bytes canoniques, expiration et issue originale | Transaction du store ; lookup ne réécrit pas le canon |
| Demande suivie | Message source, destinataire, délai déclaré, état et événements attestés | Store Bridget ; une réponse valide ferme la demande, pas une mission métier |
| Ledger | Fait d'émission, identité/corrélation, instant attesté ; phase de remise distincte | Transaction de dispatch pour un envoi idempotent ; ACK atteste séparément sa remise, unicité sous retry |
| Événement durable | event_id, corrélations, source, génération, instant transport et bytes conservés | Producteur transport ; un consommateur n'invente pas un rappel ou une clôture |
| Journal et curseur | Séquence ordonnée, raw/source, génération, état Gap/Unavailable/fraîcheur | Append durable et protocole attach/reprise existant |
| Définition de session | Commande résolue, arguments, protocole, modèle/effort, capacités, droits, digest | Registre déclaratif figé à l'ordre, revalidé sans resonder le fournisseur au replay |
| Service externe | Capacité négociée par connexion ; dépôt, claim owner/token/génération/lease, réponse et événement | Store de communication ; aucune dépendance à l'agrégat ObjectifCoordonné de Maicie |

## Transactions qui ne doivent pas être scindées

1. Réservation : validation du canon puis création unique sous concurrence ; même clé et mêmes bytes retrouvent le même record.
2. Dispatch idempotent : état dispatching, remise, ledger et suivi éventuel sont écrits dans une transaction unique. ACK : remise acked + issue Accepted + réponse liée via le helper partagé dans une transaction unique, sans réécrire le ledger. Réponse de service : replied + answered + événement requis dans la même transaction.
3. Reprise : avancer un curseur seulement après application durable de ce qu'il couvre ; échec au milieu du batch n'autorise pas le saut du reste.
4. Guichet : contrôle de capacité et du détenteur courant ; une portée de déposant n'est pas une portée d'autorisation du service. Claim périmé refusé sans écraser la réponse.

Précision issue de T012 : l'ancienne formulation « ledger à l'ACK » décrivait
un état antérieur du projet, pas la référence Git épinglée. Le code conservé
et le test `ledger_visible_pendant_dispatching_avant_accuse` attestent le
ledger dès `begin_send_delivery`. L'extraction ne déplace pas cette écriture :
une ligne visible n'est jamais une preuve d'injection fournisseur.

## États : ne pas fusionner des dimensions différentes

La progression du transport et celle d'une mission ne partagent pas un enum unique. `in_flight` est distinct d'`outcome_unknown` ; `orphaned` et les refus déterministes ne se transforment pas en attente infinie. `answered` signifie réponse liée enregistrée, pas résultat revu et correct. L'état observable du flux reste distinct de celui du processus.

Un enregistrement persistant n'est pas un événement fournisseur brut. Un événement interne synthétique reste identifié comme tel ; une sérialisation d'un DTO réduit ne peut pas remplacer la ligne brute d'origine.

## Stratégie de migration

Commencer avec un nouveau store vide dans un espace explicitement isolé. Construire les fixtures des schémas historiques retenus et leurs tests de réouverture/concurrence. Ne pas copier les tables Maicie dans le store de communication. Les tables historiques encore nécessaires aux invariants ne sont pas supprimées pour satisfaire un compteur de lignes.

L'import éventuel de données réelles est séparé du démarrage normal : copie préalable, validation de version et des références, comparaison des identifiants/bytes, aucun fallback sur une base inconnue et aucun effacement de l'original. Il ne sera exécuté qu'après autorisation de bascule.
