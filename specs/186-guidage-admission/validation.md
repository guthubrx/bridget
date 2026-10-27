# Diagnostic et validation — Session186
## Preuves d'origine
Message monuments mcp-58353-6ae0780f-1 ; complément t3-ae4de3cf-56e4-4ef2-9b56-284c98192088.
Enveloppe et résultat conservés sur Cartae :
/home/moi/10.Scripts/69.opus2D/out/monuments-158/campagne-federation-20261026/relance-utilisateur-20261027-1005.json
Clé opus2d-reprise-20261027-1005-ks-galliera ; base fc95a1583134320d454349fb47bd913d4d613ff0.
Canonicalisation selon le contrat du protocole : admission a-127252abd0c7571ec427e3f5823b0594.
Worker KS : deux réponses corrélées à10:07:47 et10:07:57UTC, respectivement9698 et9734ms.
Maître : à10:08:01UTC, candidat c001, worker_unavailable, corrélation90939325-5b5b-477b-a15e-dc9d5da63a96.
Ce motif confirme l'absence de disponibilité attestée à la dernière vérification.
Il ne distingue pas un contact trop ancien, une connexion absente ou une écriture impossible. Aucune cause de compte, modèle ou droits déduite.
Le retour propriétaire porte admission pending, task_id null. Aucun rejeu effectué ici.
Catalogue actuel : Codex KS désactivé, Claude KS/sonnet seul ; même déclaration côté maître et worker. Cette lecture ne change pas le choix historique et ne justifie aucune réactivation automatique.
Lecture SYS58459353 : isError true, texte permission_attestation_unavailable seul. Cause précise inconnue. Le code ne prouve rien sur l'état de la tâche ni son dossier.

## Défauts confirmés et correction
Les deux relectures refresh_routing_selected perdaient les motifs collectés dans leur plan temporaire avant de rendre une erreur String.
Le correctif transporte uniquement kind/code/candidates fermés et bornés, puis les projette dans detail.reasons.
Chaque sonde repart sans les anciennes notes des autres candidats.
Le refus d'attestation fournit message, recovery et human_action. Le puits attesté indique la vérification manquante, sans exposer la tâche demandée avant autorisation.
Aucun protocole externe, critère d'acceptation, politique, compte ou configuration modifié.

## Contrôles
3tests spec186 PASS : guidage, filtrage, véritable refus Status avant lecture et sans fuite.
1test incident171 étendu PASS : la relecture réelle échoue avec worker absent, conserve son motif et ne devient pas un refus terminal d'admission.
Journaux hors Git :
/home/moi/.cache/bridget184/spec186-guidance-final.log
/home/moi/.cache/bridget184/spec186-reread.log
Régressions184 et contre-revue en cours.
Auto-revue hostile : contrôle des codes fermés, borne128/16Kio, absence de secrets et d'identité de tâche, refus conservé, absence de nouvelle sonde.
Minimalisme : un fichier moteur, aucun nouveau service, store ou dépendance ; abstraction réservée au pont String existant.
Complexité : O(R), R<=128. Charge future réduite par un diagnostic conservé ; volume limité au défaut démontré.
Correction technique : aucun diaporama d'interface requis.

## Limite de livraison
Runtime183 toujours actif. Maintenance native184/185 déjà refusée update_missions_active_or_unconfirmed ; une production Cartae est en cours selon le propriétaire.
Cette correction ne contourne pas la garde et ne répare pas58459353. Aucune relance, mission de substitution, modification directe de base ou redémarrage.
