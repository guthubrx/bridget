# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/checklists/requirements.md](../../../../../../../specs/163-decision-admission/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] Le problème et la valeur utilisateur précèdent la solution.
- [x] Les cinq histoires couvrent configuration, choix, absence de service, reprise et visibilité/livraison.
- [x] Les sections problème, solution, scénarios, exigences, critères, dépendances et hors périmètre sont présentes.
- [x] La spec décrit les résultats attendus ; les propositions de stores/messages et HTTP sont séparées dans le plan et les contrats.
- [x] Les 22 exigences sont numérotées, observables et rapprochées des parcours.
- [x] Les neuf critères ont un résultat vérifiable ; aucun avantage qualitatif universel d’un modèle n’est promis.
- [x] Les quatre combinaisons d’automatismes sont couvertes, désactivées par défaut.
- [x] L’absence de service implique zéro réseau ; indisponibilité et délai de reprise sont distincts.
- [x] Les contraintes explicites, permissions et bornes restent dominantes.
- [x] Rejeu, concurrence, crash et réponse tardive ont une issue documentée.
- [x] Le repli précède l’admission, sans déplacement après dispatch.
- [x] Anciennes interfaces et absence de T3 restent couvertes.
- [x] Les limites de preuve réelle, dont Jev et Laya, sont nommées.
- [x] Aucun nouveau scheduler, moteur Git ou installation de modèle ajouté au scope.
- [x] Le socle 162 livré et ses gates G8/G9 validés sont distingués des futures validations 163.
- [x] Tâches générées seulement après les gates ; aucun résultat de test anticipé ni tâche future cochée. Branche et sélection SpecKit sous responsabilité ROOT.

<a id="extrait-2"></a>

## Extrait 2

- [x] O1 : limite minreq complète, garde MINREQ_TIMEOUT et deadline demandeur distincte.
- [x] O2 : entretien existant propriétaire des reçus orphelins, sans demandeur ni HTTP de reprise ; refus chosen terminal auto et erreurs techniques visibles non résolues.
- [x] O3 : refus auto comptés, purge sûre sous pression puis tombstone atomique ; états vivants/ambigus protégés.
- [x] O4 : repli v3 strict, compatibilité historique v2 conservée, pages homogènes.
- [x] O5 : collision native externe avant by_agent_request ; refus interne auto sans native_refusal concurrent.
- [x] O6 : Rust/Linux sur Cartae isolé, T3/macOS/signature sur Mac, recette depuis maître Mac, provenance et budget flotte.
- [ ] Validation d’implémentation et tests de ces corrections : non exécutés à ce stade.

<a id="extrait-3"></a>

## Extrait 3

- [x] Tasks puis Analyze et seconde relecture finalisés ; ordre Converge → revue/audit → checkpoint/build Mac corrigé et ROOT informé.
- [x] Checkpoint historique du 13 octobre 2026 : ROOT a attesté T001–T003, worktrees Cartae/T3, rattachement MCP réussi et gouvernance de compilation/propriété ; 3/45 uniquement préparatoires à cette date.
- [x] Tous gates avant implémentation soldés. Le wire T004 est figé ; les essais préparatoires T3 sont distincts du gate final T030 d’interopérabilité/runtime, qui reste ouvert. Aucune nouvelle demande utilisateur.
- [ ] G8 activation : endpoint/protocole/accès maîtreMac, profils retenus et checkpoint/recette réelle restent à prouver dansT039/T040/T043 avant activation des automatismes. Les gates d’activation précédemment listés sont explicitement différés ici, sans les confondre avec le GO d’implémentation.
