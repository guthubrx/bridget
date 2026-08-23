# Spécification de fonctionnalité : Renommer un agent Bridget

**Branche de fonctionnalité** : `session-01-renommage-agent-bridget`  
**Créée le** : 2026-08-14  
**Statut** : Brouillon  
**Entrée** : « Je voudrais que le protocole permette de renommer un agent »

## Scénarios utilisateur et tests *(obligatoire)*

### Histoire utilisateur 1 — Renommer son agent actif (Priorité : P1)

Un opérateur renomme un agent déjà connecté sans interrompre sa session ni perdre sa capacité à échanger avec les autres agents.

**Pourquoi cette priorité** : le nom est l’adresse humaine d’un agent ; pouvoir le corriger ou le rendre explicite évite de recréer une session pour une simple évolution d’identité.

**Test indépendant** : connecter un agent, demander son renommage, puis vérifier que son nouveau nom est visible et qu’il peut encore envoyer et recevoir des messages.

**Scénarios d’acceptation** :

1. **Étant donné** un agent connecté sous un nom disponible, **quand** il choisit un nouveau nom valide et disponible, **alors** il reste connecté sous ce nouveau nom sans redémarrage.
2. **Étant donné** un agent renommé, **quand** un autre agent lui envoie un message avec son nouveau nom, **alors** le message lui est livré.
3. **Étant donné** un agent renommé, **quand** un autre agent cible son ancien nom, **alors** le système indique que cet agent est introuvable.

---

### Histoire utilisateur 2 — Préserver une identité renommée (Priorité : P2)

Un opérateur retrouve le nom choisi lors de la reprise ultérieure de la même session d’agent.

**Pourquoi cette priorité** : un renommage doit être durable pour éviter une divergence entre le nom affiché pendant la session et celui repris ensuite.

**Test indépendant** : renommer un agent, le déconnecter puis le reprendre selon le mécanisme de reprise habituel et constater que le nouveau nom est proposé.

**Scénario d’acceptation** :

1. **Étant donné** un agent renommé avec succès, **quand** il est repris dans le même contexte, **alors** il réutilise son nouveau nom sauf choix explicite d’un autre nom.

### Cas limites

- Le nouveau nom est déjà attribué à un agent connecté : le renommage est refusé et le nom actuel ne change pas.
- Le nouveau nom est vide ou invalide : le renommage est refusé et l’erreur explique la cause.
- Deux agents demandent simultanément le même nom : un seul renommage aboutit ; l’autre est refusé sans état intermédiaire visible.
- Un agent renomme son identité alors qu’un message est en cours d’acheminement : les messages restent associés à une identité cohérente avant ou après l’opération.

## Exigences *(obligatoire)*

### Exigences fonctionnelles

- **FR-001** : Le système DOIT permettre à un agent connecté de demander le remplacement de son nom par un nouveau nom.
- **FR-002** : Le système DOIT appliquer le changement sans déconnecter l’agent concerné.
- **FR-003** : Le système DOIT refuser un nom déjà utilisé, vide ou non conforme et conserver le nom courant dans ce cas.
- **FR-004** : Le système DOIT rendre le nouveau nom immédiatement utilisable pour l’affichage, l’annuaire et l’acheminement des messages.
- **FR-005** : Le système DOIT cesser de résoudre l’ancien nom dès qu’un renommage réussit.
- **FR-006** : Le système DOIT garantir qu’un renommage concurrent ne peut attribuer le même nom qu’à un seul agent.
- **FR-007** : Le système DOIT conserver le nouveau nom pour une reprise normale de la même session d’agent.
- **FR-008** : Le système DOIT fournir à l’agent demandeur une confirmation de succès ou une erreur exploitable.

### Entités clés

- **Identité d’agent** : nom unique permettant de présenter, adresser et reprendre un agent Bridget.
- **Demande de renommage** : intention d’un agent connecté de remplacer son identité courante par une identité cible.

## Critères de succès *(obligatoire)*

### Résultats mesurables

- **CS-001** : Dans un essai avec deux agents connectés, un agent renommé reçoit un message adressé à son nouveau nom sans reconnexion.
- **CS-002** : Dans 100 tentatives de renommage vers un nom déjà attribué, 100 sont refusées et aucune ne modifie l’identité active.
- **CS-003** : Après un renommage réussi, l’ancien nom ne permet plus l’acheminement des messages.
- **CS-004** : Une reprise normale de l’agent concerné réutilise le nom renommé sans intervention supplémentaire.

## Hypothèses

- Le nom choisi suit les mêmes règles de validité que les noms fournis au lancement d’un agent.
- Seul l’agent concerné peut demander son propre renommage ; aucun agent tiers ne peut renommer un autre agent.
- La commande de renommage est une action explicite de l’opérateur de l’agent.
- La fonctionnalité n’introduit pas de renommage en lot ni de redirection permanente de l’ancien nom.
