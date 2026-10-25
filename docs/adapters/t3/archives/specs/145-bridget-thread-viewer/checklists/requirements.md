# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/checklists/requirements.md](../../../../../../../specs/145-bridget-thread-viewer/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] Le besoin utilisateur et la surface native T3 sont décrits.
- [x] L'icône `b` neutre et l'absence de bleu spécifique sont explicites.
- [x] Les histoires sont prioritaires et testables indépendamment.
- [x] Les titres, membres, auteurs, dates, types et corps exacts sont exigés.
- [x] La consultation de plusieurs fils et la pagination bornée sont définies.
- [x] La recherche est locale et sa portée est indiquée.
- [x] Le rafraîchissement est manuel ; aucune boucle de lecture n'est ajoutée.
- [x] La liaison T3 est résolue côté serveur ; aucun identifiant navigateur ne suffit.
- [x] L'autorisation est vérifiée à chaque opération selon l'appartenance.
- [x] La voie humaine distincte est fermée à liste, détail et historique.
- [x] Les sorties excluent credentials, transport et curseurs agents.
- [x] L'absence d'émission, ACK, réveil, mission modifiée et appel modèle est testable.
- [x] Les réponses tardives, changements de contexte, fermeture et révocation sont couverts.
- [x] Les états daemon absent, liaison absente, refus, délai et version incompatible sont exigés.
- [x] L'accès clavier, le focus, la sélection et les noms accessibles sont exigés.
- [x] L'absence d'API Bridget actuelle dans T3 et le non-réemploi de `ui.rs` sont explicites.
- [x] Les limites d'autorisation de livraison et les worktrees exacts sont consignés.
- [x] Aucun helper SpecKit absent n'est déclaré exécuté.
- [x] Aucun point fonctionnel `[NEEDS CLARIFICATION]` ne subsiste. Les valeurs techniques seront fixées dans le contrat avant les tâches.
