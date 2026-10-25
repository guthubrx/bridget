# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/validation/analyze152.md](../../../../../../../specs/152-reprise-missions/validation/analyze152.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Question | Réponse de conception |
|---|---|
| Pourquoi nécessaire ? | Les cinq défauts sont identifiés dans le moteur existant ; panne/pause/annulation/refus sont le périmètre utilisateur. |
| Pourquoi plus simple ? | Une reprise ADR027, une chaîne d’exécutions, une garde et l’entretien existant ; pas de second moteur ni état dérivé stocké. |
| Quelles hypothèses ? | Deux reprises, délai hors pause continu, Lineage working ; identité/bootstrap et reprise imbriquée prouvés dans T004/T048 avant effet. |
| Qu’est-ce qui est vérifié ? | Lecture fraîche des artefacts et code ciblé ;26 items du plan, 29 FR/9 SC/6 US mappés ; baseline du principal239/0/1, six RC0. |
| Qu’est-ce qui ne l’est pas ? | Aucun test152 exécuté par cette passe, aucune suite complète actuelle, modèle/T3/activation réels, couverture ou mutation mesurées. |
| Que conserve-t-on et pourquoi ? | Compteur durable avant reprise, cancel_stopped et reçus fermés : faits non dérivables après crash ; ancien canon, ACK151 et sécurité des marqueurs conservés. |
