# Checklist des exigences — SPEC145

Date : 2026-10-07. Revue documentaire initiale. Cette liste ne prouve aucune implémentation.

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

Gate de plan : vérifier les contrats et chemins avant création de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/tasks.md`. Le principal donne le GO.

Vérification finale : une recette distincte recevra les preuves SC145 avant clôture. Ce contrôle ne bloque pas la planification ou l'implémentation.
