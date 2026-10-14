# Checklist de qualité SPEC-163

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 ajoutés ici restent à committer/pousser par ROOT, sans hash futur anticipé.

**Date initiale :** 2026-10-13 ; qualité documentaire initiale, résultats de clôture distingués dans les reçus.
**Spec courante :** /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/163-decision-admission/spec.md

## Qualité de la spécification

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

## Gates avant tâches

- [x] G8 et G9 du lot 162 attestés par ROOT et git-final162.json ; 46/46 tâches.
- [x] Gate de réemploi ROOT : inventaire indépendant de 34 éléments et six précisions relus ; PASS du réemploi indépendant ; R1 reçue et corrections relues par ROOT, gate soldé.
- [x] Contre-revue R1 reçue : APPROVE_WITH_CHANGES ; six objections vérifiées par ROOT et corrections documentaires intégrées.
- [x] ROOT a vérifié les objections R1 contre le code, puis relu les corrections O1–O6 ; gate de plan soldé. Le verdict externe reste APPROVE_WITH_CHANGES, sans inventer une seconde revue.
- [x] Décisions ROOT intégrées : transaction Immediate Task/lien/collision, UNION sans double compte, Hello/Accepted additifs négociés par époque. Aucun code déclaré validé.
- [x] Client minreq2.14.1 relu : sans timeout interne, tâche réseau unique, deadline caller séparée, corps/en-têtes/statut bornés, redirects0 ; blocages DNS/connexion/TLS/écriture/lecture durables explicités ; MINREQ_TIMEOUT présent bloque le seul service, garde au démarrage et avant appel, raison visible.

## Gates d’activation ultérieure

- [ ] Endpoint/protocole/accès depuis le maître et santé du service réel attestés.
- [ ] Profils fixes, de repli et préférences de production explicitement retenus.
- [ ] Pour le profil Laya, checkpoint exact et cas maximal de template vérifiés par recette.

La qualité documentaire est relue. Le plan est In Progress après GO ROOT ; T001–T004 sont prouvées et les corrections de convergence restent ouvertes. L’Analyze documentaire couvre 31/31 FR et SC, sans preuve de code futur ni d’activation. Aucun blocage de scope utilisateur ajouté ; ces gates demandent des preuves et des revues ciblées.

Checkpoint de correction : rétention7j sous gardes/tombstone de 30 jours après purge/refus de 30 jours sans plafond natif, borne metadata16Kio distincte du canon et mission64Kio corrigées. Préflight de santé intégré comme lecture déléguée sans reçu fichier ni POST. Le gate de protocole/checkpoint réel reste ouvert.

## Corrections documentaires R1 — contrôles distincts des preuves de code

- [x] O1 : limite minreq complète, garde MINREQ_TIMEOUT et deadline demandeur distincte.
- [x] O2 : entretien existant propriétaire des reçus orphelins, sans demandeur ni HTTP de reprise ; refus chosen terminal auto et erreurs techniques visibles non résolues.
- [x] O3 : refus auto comptés, purge sûre sous pression puis tombstone atomique ; états vivants/ambigus protégés.
- [x] O4 : repli v3 strict, compatibilité historique v2 conservée, pages homogènes.
- [x] O5 : collision native externe avant by_agent_request ; refus interne auto sans native_refusal concurrent.
- [x] O6 : Rust/Linux sur Cartae isolé, T3/macOS/signature sur Mac, recette depuis maître Mac, provenance et budget flotte.
- [ ] Validation d’implémentation et tests de ces corrections : non exécutés à ce stade.

## Gate avant implémentation et activation séparée

- [x] Tasks puis Analyze et seconde relecture finalisés ; ordre Converge → revue/audit → checkpoint/build Mac corrigé et ROOT informé.
- [x] Checkpoint historique du 13 octobre 2026 : ROOT a attesté T001–T003, worktrees Cartae/T3, rattachement MCP réussi et gouvernance de compilation/propriété ; 3/45 uniquement préparatoires à cette date.
- [x] Tous gates avant implémentation soldés. Le wire T004 est figé ; les essais préparatoires T3 sont distincts du gate final T030 d’interopérabilité/runtime, qui reste ouvert. Aucune nouvelle demande utilisateur.
- [ ] G8 activation : endpoint/protocole/accès maîtreMac, profils retenus et checkpoint/recette réelle restent à prouver dansT039/T040/T043 avant activation des automatismes. Les gates d’activation précédemment listés sont explicitement différés ici, sans les confondre avec le GO d’implémentation.
