# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/validation/audit169.md](../../../../../../../specs/169-priorite-bridget/validation/audit169.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'audit ci-dessous est le reçu figé de la phase source avant commit/build. Ses mentions de HEAD=base, plage historique vide et J2 ouvert sont celles de cet instant. ROOT a ensuite publié la même source35 sous le commit T3 58ddcf3a16205a24f940164cba500089f49ec6c0, après vrai hook formatter et sans bypass ; les35 SHA restent R2 identiques. Aujourd'hui J2 est partiel :T3 publié, documents Bridget encore à committer par ROOT. Aucun paquet ni runtime169 n'est qualifié par ces copies.

<a id="extrait-2"></a>

## Extrait 2

| Source lue | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery169/audit-v15-169-01/recu-final169.md | `c06bbfd74cc76cd5e382e6f0919f86ac70ef598e37a070e8b4c254e168e8516d` |
| /Users/moi/.cache/bridget-delivery169/audit-v15-169-01/grade.json | `e4c1051f66fb9e713d3718d5a2f023a48bdcbe79504ed19b33ed78fb5ba55556` |
| /Users/moi/.cache/bridget-delivery169/audit-v15-169-01/axis-scoring.md | `08eb70d571f2c94ef267e1add0d077f9911f1f9d9ea3f748ac4668cd5026babe` |
| /Users/moi/.cache/bridget-delivery169/audit-v15-169-01/notes/analyse169.md | `1f60ef647088100556b503897e9473d4a079daca36f79747b6f2d9b63a0ceb86` |
| /Users/moi/.cache/bridget-delivery169/audit-v15-169-01/notes/traceabilite169.md | `57d1d0d22713c5e371d020d03766111e6763596289f221768b04aec76f978956` |
| /Users/moi/.cache/bridget-delivery169/git-t3-source169.json | `cf9c2f64fc9c4aa31e8e9a941f1d0b1c502ce4fdd640d4cb6a27709c2906ace7` |

<a id="extrait-3"></a>

## Extrait 3

| Critère | Niveau /4 | Justification |
|---|---|---|
| Gestion des refus et messages utilisateur | 2 | Refus nommés et blocked explicite dans texte/tests ; visibilité UI de l’erreur de divergence non prouvée. Niveau3/4 demande le parcours réel de restitution. |
| Gestion des erreurs partielles | 3 | Lecture non fiable refusée avant secret, candidates retirées fermées, divergence avant prompt, Pi sans avancement sur reçu absent. Niveau4 demande panne native positive et sortie physique qualifiée. |
| Ergonomie des parcours concernés | 2 | Exception T3 et slash conservées ; commande privée cachée/refusée. Pas de parcours cible d’activation ni preuve UI de divergence. Niveau3/4 exige cette qualification. |

<a id="extrait-4"></a>

## Extrait 4

| Exigence | Mécanisme examiné | État et borne |
|---|---|---|
| FR001 | Schéma ServerSettings : un booléen par fichier, hors patch public. | Couvert source et tests de schéma/fichier. |
| FR002 | Défaut false, absence fiable OFF ; activation ON séparée. | Source couverte ; ON cible non acquis. |
| FR003 | Snapshot réglages/confiance ; refus writer avant secrets. | Couvert fichier/watch/JSON/I/O et tests. |
| FR004 | Lecture avant chaque runtime.startTurn et injection de mode. | Couvert source, cinq tours intégrés, matrices ; modèle réel non exercé. |
| FR005 | Exception T3 explicite, outils hors délégation, slash. | Couvert texte et fixtures des chemins concernés. |
| FR006 | Axes humains préservés, axes libres laissés à Bridget. | Couvert consigne ; pas d’admission réelle nouvelle169. |
| FR007 | Aucun choix de flotte ni dispatch ajouté à l’hôte. | Couvert diff et absence de dépendance/service neuf. |
| FR008 | Mode sans mutation de credential ; montage réel et opt-outs hérités. | Couvert source/fixtures et réemploi déclaré ; santé daemon future exclue. |
| FR009 | blocked explicite, pas de repli d’hôte. | Couvert matrices ; aucune promesse d’obéissance absolue du modèle. |
| FR010 | Références et rejeu Bridget existants conservés. | Source couverte ; socle réutilisé, pas nouvelle admission réelle. |
| FR011 | Nouveau contexte seulement ; suivi et moteur initial conservés. | Couvert source/régression ; reprise cible J5 ouverte. |
| FR012 | Aucun delta de projection Lineage ; régression149 reçue. | Réemploi borné ; pas de nouvelle preuve visuelle. |
| FR013 | Moteur natif autonome inchangé. | Couvert périmètre ; pas nouveau parcours standalone réel. |
| FR014 | Codex/Claude : fait réel ; ACP/OpenCode1/2/Cursor/Pi : blocked. | Couvert six familles ; Pi réel et reprise Claude existante positive non acquis. |
| FR015 | Réglage unique, input existant, commande Pi privée fermée. | Couvert source ; aucun nouveau protocole public ou état durable. |
| FR016 | Principal authentifié HTTP, seul visibleTurnItems vide. | Couvert vrai HTTP isolé ; reprise du gros fil cible non acquise. |
| FR017 | Procédure de livraison prévue ; aucune exécution par cet audit. | Gate opérationnelle après build, hors décision source ; reste ouverte. |

<a id="extrait-5"></a>

## Extrait 5

| Critère | Verdict de cette phase |
|---|---|
| SC001 | Matrices source/fixture couvertes sur les six familles. OFF avec MCP byte-identique ; reset sans MCP explicitement différent. |
| SC002 | Erreurs explicites, refus avant secret et fichier conservé prouvés en intégration isolée. |
| SC003 | Consignes et séparation des compteurs prévues ; recette native réelle ouverte. |
| SC004 | Contraintes et stabilité d’identité couvertes au niveau consigne/montage/fixture ; pas admission réelle169. |
| SC005 | Absence/refus et absence de dispatch secondaire couverts en source/fixtures ; socles hérités bornés. |
| SC006 | Transitions couvertes ; suivi réutilisé ; reprise de la cible ouverte. |
| SC007 | Exceptions T3 et slash couvertes ; commande Pi privée refusée au chemin utilisateur. |
| SC008 | Aucun nouveau service, dépendance, UI ou stockage distinct constaté dans le diff35. |
| SC009 | Gate opérationnelle après installation ; code chargé et reprise réelle non acquis. |
