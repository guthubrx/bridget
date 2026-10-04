# Recherche 133 — Relais de sous-agents

## Décision 1 — Le parent reste le principal Bridget

**Décision** : un sous-agent interne utilise une capacité déléguée du parent. Il
ne reçoit pas une identité Bridget durable. Les réponses reviennent au parent.

**Raison** : Anthropic décrit les systèmes multi-agents comme un orchestrateur
qui délègue puis synthétise les résultats. OpenAI distingue aussi l'agent utilisé
comme outil, où le responsable garde la conversation, du transfert complet de
conversation. Ce besoin relève du premier modèle.

**Sources** :
- https://www.anthropic.com/engineering/multi-agent-research-system
- https://developers.openai.com/cookbook/examples/agents_sdk/migrate-from-claude-agent-sdk/readme

**Alternatives** : identité durable par enfant ; transfert complet vers l'enfant ;
relais textuel manuel par le parent. La première ajoute un cycle de vie et des
orphelins. La seconde change le propriétaire de la conversation. La troisième
conserve le défaut vécu.

**Impact mainteneur** : une seule identité routable et une preuve enfant locale.

## Décision 2 — Capacité fermée à `who` et `send`

**Décision** : l'enfant consulte l'annuaire et envoie. Tous les autres outils
sont refusés avant exécution.

**Raison** : les recommandations OpenAI placent outils, secrets, validations et
approbations dans un runtime de confiance. Le guide des garde-fous recommande une
validation au point d'outil et une approbation pour les effets sensibles.

**Source** : https://developers.openai.com/api/docs/guides/agents/guardrails-approvals

**Alternatives** : hériter de tous les droits du parent ; liste configurable ;
aucun outil. L'héritage est trop large. La configuration ajoute un système de
politique sans usage prouvé. L'interdiction totale ne répond pas au besoin.

**Impact mainteneur** : une liste fermée de deux noms, testée au répartiteur.

## Décision 3 — Preuve locale distincte et non transmissible

**Décision** : le pont T3 publie un marqueur enfant privé distinct des marqueurs
d'identité principale. La CLI refuse ce marqueur. Seule la façade MCP le comprend.

**Raison** : le contrat MCP décrit une identité de client/session et exige une
autorisation explicite. Ses recommandations de sécurité refusent le passage
implicite de jetons et le « confused deputy ». Une preuve limitée à une seule
façade réduit ce risque.

**Source** : https://go.sdk.modelcontextprotocol.io/protocol/

**Alternatives** : réutiliser directement le marqueur principal ; passer un
secret dans le prompt ; déduire le parent du nom ou du dossier courant. La
réutilisation donne trop de droits. Le secret dans le prompt peut fuiter. Les
heuristiques ne prouvent pas le parent.

**Impact mainteneur** : deux répertoires aux rôles explicites, même mécanique de
fichiers privés et même inventaire OS.

## Décision 4 — Provenance structurée, référence enfant opaque

**Décision** : chaque message enfant porte fournisseur et empreinte opaque. Le
format n'expose pas l'identifiant de session natif.

**Raison** : le destinataire doit distinguer le parent direct d'un sous-agent.
La provenance structurée évite de dépendre d'un texte libre et garde les règles
d'affichage centralisées.

**Alternatives** : préfixe libre dans le corps ; aucune provenance ; identité de
routage enfant. Le préfixe libre est fragile. L'absence trompe le destinataire.
L'identité enfant recrée une boîte de réception et un cycle de vie.

**Impact mainteneur** : un champ optionnel rétrocompatible et deux valeurs bornées.

## Charge et limites

Anthropic rapporte une consommation de jetons nettement supérieure pour les
systèmes multi-agents et souligne que le codage offre moins de tâches réellement
parallèles que la recherche. La fonction ne lance donc aucun agent et ne crée
aucune boucle. Elle autorise seulement un relais explicite quand l'hôte a déjà
créé un enfant.
