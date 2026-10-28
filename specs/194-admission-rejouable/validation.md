# Qualification194 — Admission rejouable

## Cause et demande réelle
Requête propriétaire `opus167-revue-fermeture-et-gardes-20261028-01`, owner `3de3f2a6-6b4a-4415-8a1b-a870fe5a010f`. Reçu durable `decision_started`, époque60/révision1, tâche nulle, décision nulle. Le journal corrélé à `a-89e64b706b74d41cb0655cb7fbb5a7ef` confirme un dernier refus `candidate/workspace_diverged`, candidatc001, corrélation29bcfef8-ba8c-4db3-8e7e-bb51f53459c3. La base demandée47edec6 n’est plus celle du shared courant (Mac et Cartae fe405e7 au relevé). Les anciens refus resource_limit/capability_unavailable du lot initial ne sont pas la cause de cette dernière sonde. Ni panne de compte ni panne réseau déduite.

Dans183, le choix n’est persisté qu’après la sonde sélectionnée. Son refus `routing_candidate_unavailable` n’est pas terminal dans la liste fermée ; le reçu reste donc `decision_started`. Un rejeu de même époque retourne `admission_pending`, sans reprendre le choix. Preuve restreinte : /home/moi/.cache/bridget194/admission-opus167-diagnostic.json. Aucun rejeu métier, annulation, service, configuration ou store modifié.

## Correction et preuves
Le choix et son effort sont persistés avant la sonde finale. La matrice initiale reste figée. Le parcours exclusif existant relit la source et revalide le profil initial et l’effort décidé, puis la préparation vérifie encore les droits. Un refus temporaire garde `chosen`, réutilisable au rejeu strict sans décideur. Aucun nouveau champ durable ou seuil.

Contre-épreuve : le test194 échoue avant correction (exit101, DecisionStarted au lieu de Chosen). Après correction, il passe pour l’effort initial high et l’effort choisi low. Il vérifie store rouvert, zéro tâche et zéro lancement après refus, puis une seule tâche au rejeu et décision/canon inchangés. Les35contrôles spec168 et3contrôles spec183_rejeu passent : sélection/efforts, repli, récupération après époque, concurrence, source et révocation. Total39contrôles ciblés PASS, aucune FULL. Logs /home/moi/.cache/bridget194/test194.log, test168.log, test183.log. Guides32pages/522liens/13paires et diff --check PASS.

## Limites et livraison en cours
Revue indépendante et candidat à qualifier. Le maître chargé reste183 ; la maintenance native est bloquée par une ancienne clôture non attestée. La correction ne modifie pas le binaire déjà vivant et ne répare pas par lecture l’ancien reçu decision_started. Aucun restart forcé ou modification manuelle du store admis. Chargement et usage propriétaire restent ouverts.

## Complément après contre-revue
La revue native ccda1f30 a relevé un vrai P1 : la garde distante comparait encore l’effort décidé au défaut initial du candidat. Le candidat fee0621 est donc refusé pour activation. Correction ciblée : la garde compare désormais l’effort de la décision persistée et l’égalité du canon Delegate. Les sondes restent liées au profil initial puis à l’effort choisi ; la préparation reçoit les empreintes de ce dernier. Aucun critère de droits retiré.

Contre-épreuve distante avant complément : le test échoue avec routing_connection_changed au lieu d’atteindre PrepareV2. Après complément : une vraie conversation protocolaire de worker refuse d’abord workspace_busy, puis les deux rejeux atteignent PrepareV2 avec low et les empreintes attendues. Le worker contrôlé refuse volontairement Prepare par ResourceLimit : aucune tâche, Start ou Release et aucun effet fournisseur. Le choix/canon restent strictement identiques. Cette épreuve ne prétend pas lancer une mission distante réelle. L’unicité de la tâche et l’effort initial/différent sont vérifiés par le scénario local194 ; concurrence/source/révocation par183. Le résultat de la première revue a été lu, acquitté et sa garde clôturée nativement (cleanup_done confirmé).
