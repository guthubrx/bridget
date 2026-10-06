# Contrat 138 — Communication à portée explicite

Ce document décrit le contrat à implémenter. Les options nouvelles ne sont pas
présentées comme déjà disponibles dans le binaire installé.

## Annuaire

Nouvelle variante DirectoryScoped avec scope: same_project | global.
Réponse CommunicationDirectory séparée: ScopedAgentInfo avec AgentInfo aplati,
communication_project optionnel et project_relation same/other/unknown.
AgentInfo et ListAgents de diagnostic restent inchangés.
CommunicationProjectFact est une annonce séparée après Register. Seul le
transport propriétaire vivant peut l'écrire ; les auxiliaires en héritent.
Un client CLI de fond négocié peut établir son propre contexte depuis la racine
explicite du run, validée avec son hôte par le daemon. Aucun Register d'agent ni
identité T3 empruntée. Ce contexte n'écrase jamais celui d'un auxiliaire.
Le projet appelant vient de la connexion attestée. La requête ne reçoit pas
un project_id choisi librement comme autorité. Défaut de communication:
same_project. ListAgents reste un diagnostic global compatible.

CLI: who/agents sans option utilisent le projet ; --global choisit la vue globale.
Sans rattachement attesté ni --project-root explicite, un client CLI autonome
reste inconnu, y compris who/agents : zéro suggestion locale et warning.
Le cwd n'est jamais ajouté implicitement. T3/wrapper transmettent leur contexte.
MCP: who expose scope avec défaut same_project. Les résultats contiennent les
agents retenus et des métadonnées de portée, hors du corps des messages.
Un projet inconnu renvoie une liste locale vide avec project_unknown.

La résolution d'une cible explicite peut consulter le global pour la trouver.
Cela ne constitue pas une suggestion ou un choix interprojets implicite.

## Envoi et fils

BridgetMessage et ThreadRequest acceptent cross_project_reason optionnel.
CLI: --cross-project-reason "Comparer le contrat partagé".
MCP: "cross_project_reason": "Comparer le contrat partagé".
La présence valide exprime le mode volontaire. Aucun deuxième booléen requis.
Valider les 1 à 512 octets UTF-8 trimés avant le canon. null et contrôles sont
des paramètres invalides. Les champs inconnus restent refusés.

| Fait de portée | Motif | Décision pour une opération nouvelle |
|---|---|---|
| Même projet attesté | absent ou valide | Contrôles existants puis dépôt |
| Autre projet attesté | absent | Refus cross_project_reason_required avant dépôt |
| Autre projet attesté | vide/invalide | Refus invalid_cross_project_reason avant dépôt |
| Autre projet attesté | valide ou mandat corrélé | Warning cross_project puis dépôt |
| Au moins un projet inconnu | absent | Compatibilité existante avec warning project_unknown |
| Rejeu exact déjà durable | identique | Résultat antérieur ; aucun nouvel effet |
| Même clé, motif/corps différent | différent | Conflit idempotent existant |

Le warning est établi avant notification et figure dans le résultat caller.
Il ne demande aucune confirmation humaine supplémentaire. Son corps n'est
jamais concaténé au texte envoyé. Un refus ne dépose rien partiellement.

Pour create/post, l'audience est l'ensemble des membres lecteurs. Les règles
s'appliquent avec notify=[] comme avec notify=all. Chaque opération mixte porte
son motif. Le caller peut reprendre celui d'un mandat explicitement configuré
pour cette audience. Aucun consentement persistant n'est ajouté au fil.

Pour une réponse directe, la demande suivie OPEN doit être réellement corrélée et
porter l'autorisation initiale. Vérifier les deux participants inversés. Une
in_reply_to forgée n'autorise rien.

## Compatibilité et négociation

Omettre le motif conserve les octets historiques du canon. Un motif présent
modifie le canon même si le reste du message est identique. Les vieux clients
connus restent soumis à la garde daemon ; les inconnus restent avertis.
Le client exige la capacité138 avant un envoi portant le nouveau motif. Un
serveur qui ne l'annonce pas provoque un refus client sans aucun envoi.
La fédération transmet le motif et le verdict de portée connu. Sans preuve
vérifiable du pair, la portée est inconnue. Un nom de domaine n'est pas une preuve.

Les opérations déjà acceptées ne sont pas resoumises à une nouvelle décision.
Aucune lecture d'historique n'entraîne un envoi ni une conversion de mandat.

## Agent Loop

attach-agent et dispatch vérifient la portée du projet du run. Un mandat explicite
associe motif et agents/rôles attribués. Les rappels worker/coordinator/ROOT
réutilisent ce mandat dans leur envoi structuré. La configuration générale ROOT
sans mandat produit une décision à prendre, pas une notification extérieure.
Le domaine du run n'autorise aucun destinataire. Aucun repli extérieur automatique.
Le lot durable fige racine de contexte, motif et destinataire avec le corps
avant la première tentative. Une reprise n'utilise pas un mandat modifié.

## Contrôle de consigne injectée

ExecutionControlV1, opération SteerCurrent.message, applique la même garde de
portée avant injection. Un projet connu extérieur exige un motif structuré
valide ; l'inconnu conserve sa compatibilité avertie. Le résultat caller porte
project_warnings hors corps, établi avant l'effet sans nouveau dialogue humain.
Ce contrat n'invente aucune option CLI de contrôle.

Le résultat accepté et ses warnings durables sont rendus avant une garde mutable
sur une reprise exacte. Aucun contenu n'est réinjecté. Une seule colonne JSON
project_warnings dans execution_control_commands existante conserve ces warnings,
avec défaut [] et migration compatible. canonical_bytes reste exact et immuable ;
refusal_reason ne porte pas de warnings. Les commandes anciennes rendent [].
Le canon sans motif et Interrupt sans message restent inchangés.
