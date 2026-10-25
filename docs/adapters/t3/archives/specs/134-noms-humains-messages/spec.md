# Archives techniques de l’adaptateur T3

Document d’origine : [specs/134-noms-humains-messages/spec.md](../../../../../../specs/134-noms-humains-messages/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Branche** : session-134-noms-humains-messages
**Date** : 2026-10-05
**Statut** : Implemented
**Priorité** : P1
**Demande** : afficher un nom humain à côté de l’identifiant de l’expéditeur dans les messages Bridget.
**Dépendances** : 098 (pont T3), 101 (identité T3), 110 (profil et nom de présentation), 114 (lots de messages), 133 (provenance des sous-agents).

<a id="extrait-2"></a>

## Extrait 2

Les messages Bridget affichent aujourd’hui un identifiant long. L’utilisateur ne
peut pas savoir rapidement quel agent parle. Le pont T3 connaît pourtant le titre
humain du fil, et Bridget possède déjà un champ de nom de présentation.

<a id="extrait-3"></a>

## Extrait 3

- **SC-13401** : 100 % des messages de test avec un nom exploitable affichent
  `nom (UUID)` dans leur en-tête.
- **SC-13402** : 100 % des messages sans nom exploitable gardent le rendu UUID
  historique et sont remis sans erreur.
- **SC-13403** : 100 % des éléments de lots testés affichent le bon nom et le bon
  UUID.
- **SC-13404** : 100 % des identités de test privées de profil retrouvent un
  profil après un nouvel enregistrement, sans créer une seconde identité.
- **SC-13405** : les suites de messages, profils, T3 et espace de travail passent,
  ainsi que le formatage, l’analyse statique et la construction de production.

<a id="extrait-4"></a>

## Extrait 4

- Masquer ou raccourcir l’UUID.
- Renommer automatiquement les agents sans titre T3.
- Modifier T3 Code ou les fournisseurs.
- Réécrire les anciens messages stockés.
- Créer une migration globale des profils en production.
