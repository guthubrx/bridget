# SPEC139 — Tous les en-têtes Bridget discrets
Date : 2026-10-07. Autorisée par « go ». Statut : sources livrées, paquet validé.
Commit, fusion, push et déploiement ensuite autorisés explicitement par l'utilisateur.

## Scénarios utilisateur
US1 (P1) : distinguer le contenu d'une sollicitation de son en-tête technique.
Acceptation : sollicitations, messages directs, observations et lots ont un
en-tête plus petit et gris, dans les thèmes clair et sombre.
US2 (P1) : conserver intégralement les noms, identifiants, textes et copies.
Acceptation : anciens et nouveaux messages bénéficient du rendu sans migration.
Les citations, messages ordinaires et enveloppes incomplètes restent ordinaires.
US3 (P1) : recevoir un paquet réellement préparé, sans interrompre les agents.
Acceptation : archive construite, vérifiée et identifiée ; installation distincte.

## Exigences et succès
FR01 : tous les formats Bridget émis par le pont T3 sont couverts.
FR02 : seul le premier paragraphe de l'enveloppe complète est atténué.
FR03 : texte, copie, historique, transport et relances inchangés.
FR04 : tests ciblés et aperçu isolé ; aucune écriture dans les données actives.
FR05 : arrêter proprement l'application avant remplacement ; livrer un paquet
traçable avec sauvegarde, contrôle santé et retour arrière. Ne pas modifier les données.
Succès : tests positifs/négatifs PASS, mesures visuelles et archive vérifiées.
