# Validation du correctif226

Cause constatée : snapshot expiré ferme tous les liens, journaux Mac du2novembre05:30–05:43UTC. Comportement introduit par9356b0f8 le14septembre, antérieur à0.7.4.
Sources c9c103f1/c6160be5. Quatre tests ciblés R2 PASS, 2744 autres non relancés.
Premier essai de fixture lisait une trame initiale au lieu du heartbeat : corrigé par lecture des trames réelles. Le renforcement de fixture utilisait une chaîne au lieu de ObservationKind : erreur de compilation corrigée. Aucun de ces essais non passants ne vaut preuve.
Revue757976fd : aucunP1, P2heartbeat horsgrâce corrigé par événement dédié SnapshotTimeout. P3connexion/réponse distingué. Delta relu par le responsable, sans nouvelle campagne. Reçu lu/ACK/gardeclose ; vue conservée tant que les usages restent incomplets.
Bornes :90s depuis début du dernier succès, PID/port/naissance identiques. Aucune autorité ou observation conservée ; dispatch relit l’état. Pas de retry RPC nouveau ni de mission métier rejouée.
Assemblages Mac/Linux portables R2 et activation : à compléter.
Limite séparée : rôle mainteneur bridget non transversal, mandat vide. Décision humaine demandée sans changer les permissions.
