# Converge151 — passage3 indépendant après audit

Date :2026-10-11. Verdict : **CONVERGED**. Zéro manque confirmé, zéro tâche ajoutée. Les tâches sont restées byte-identiques.

Les39FR,13SC et7US restent couverts après les deux corrections d'audit. Lecture arrêtée à10:56:21UTC, dans la borne de quatre minutes. Aucune écriture, commandeGit, compilation, exécution de test ou action sur un service durant la phase6.

Le principal a accepté ce verdict après son recontrôle indépendant :47/47 fichiers inchangés, trois captures conformes,27 tâches/0 ouverte. Ce reçu est écrit après la fin de la phase6, sur autorisation explicite.

## Repères fichier:ligne actuels

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/converge151-passage3.md#extrait-1).

## Matrice FR

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/converge151-passage3.md#extrait-2).

## Matrice SC

| SC | Code et test actuels | Preuve retenue |
| --- | --- | --- |
| 001 | N:483 ; E:301 | Daemon réel, capture SQLite, zéro Status ; E012:971ms |
| 002 | N:546 ; E:301 | Expiration1s, enfant poursuivi, résultat unique |
| 003 | N:1243 ; E:301 ; N:2398 | Message conservé, lecture pure |
| 004 | N:535 ; N:2672 | Trois résultats, une réponse |
| 005 | T:25 ; N:2697 | 100 courses, aucune expiration perdue |
| 006 | N:90 ; E:454 | pending, claimed, delivered |
| 007 | N:84 ; N:2943 ; V | Clés historiques ; régressions et huit échecs hérités séparés |
| 008 | N:483 ; D:242 ; N:2487 ; D:419 ; N:2729 | Calme SQLite et progression64 ; contrôles sous500ms |
| 009 | D:7979 ; N:1958 | ACK réel après preuve open ; deux scopes ; 406µs/357µs archivés |
| 010 | D:211 ; M:96 | N:2630 ; M:2858,M:2928,M:3449,M:3631 |
| 011 | C:24 ; A:31 | Ancien daemon réel :44,710/54,874ms |
| 012 | M:190 ; E:749 | Quatre attentes3600 fermées à EOF ;14ms archivées |
| 013 | N:551 ; D:315 ; N:2454,N:2290 | Expiration sous1,5s ; Guichet sous2s avec filet60s |

## Matrice US

| US | Réalisation et tests actuels |
| --- | --- |
| US1 | N:483 ; E:301 ; N:2454 |
| US2 | N:73 ; N:2872,N:2918,N:2943 |
| US3 | N:90 ; N:2848,N:2972 ; E:454 |
| US4 | N:535 ; N:2672 ; B:3632 |
| US5 | D:242 ; R:699 ; N:1958 ; I:121 ; G:253 |
| US6 | C:24 ; P:7938 ; N:2943 ; A:31 ; W:8130 |
| US7 | N:796 ; T:190 ; huit tests N:3026 à N:3384 ; E:707 |

## Corrections revalidées

Les tests ne reposent plus sur une attente arbitraire de80ms.

- Cancel : garde N:2252, réservation active N:2257, thread encore actif N:2258, demande ouverte N:2259 ; stimulus N:2270.
- Guichet : garde N:2348, réservation active N:2353, thread encore actif N:2354, demande ouverte N:2355 ; stimulus N:2366.
- TaskWait : garde N:2558, réservation active N:2563, thread encore actif N:2564, tâche non terminale N:2565 ; publication N:2575.

Le journal actuel confirme59PASS/0FAIL/0ignored,RC0. La première tentative reste visible :22PASS/37FAIL, causés par SUN_LEN avec le répertoire temporaire long. Aucun test exécuté pendant Converge.

## Intégrité avant/après

Le source_document_freeze comporte47 fichiers. Chaque taille et SHA256 a été comparé au fichier réel avant puis après la lecture :47/47 conformes, changed=[] dans les deux contrôles. Empreinte d'inventaire inscrite au manifeste :1672c34c7b1f1ba9761b5af0be72328fe1a7f33b4b62b0218f609b9fccadfad9.

Tâches27cochées/0ouverte. SHA256 avant/après :d04cf9d3ab41aeed6af4a7ac1d3ec597691713b53ec9a84cb7721e7a80a408c3.

Native complet :9894185b28f67fa90f96c0b48354fa2dae4c3f64723585203dff7760c665e528.
Préfixe de production Native84725octets :73079979abb26706fa1528fea47a35529e0e25d6c997c61e1fac9e28d7b4fad6.
JSONL complet :7969f3c7c8d8ef7028be0cf8549ec6c759fac1822f3407a4e1fba1658a1c0dc7.

| Capture | Octets | SHA256 |
| --- | --- | --- |
| diff-151-final-r3.patch | 525432 | d740138583095e5b3bfc5506511bf5d077c8abab968f7f78502a01ad9e810a62 |
| delta151-vs150-final-r3.patch | 326290 | 89f0714a7c47c2de314bd0d3ea9fb33adbe1d8e3dd8589db514e6708bac7de70 |
| snapshot-151-final-r3-manifest.json | 122825 | 13371919fdc71db1036678bcf4e21986be8006a6b332d018aa08e47caf1bcc1f |
| audit-fix-tests151.log | 35033 | 9fc383962cb9ab45c3d70cb07f1736de5f535a8cf86e08d5f63cb713db0827f9 |
| audit-format151-final.log | 694616 | 2bfa949050a0874d881e5d018030c0fbcd2b3b50857d4d1ff1b6d9444219598f |

## Six réponses Article XX

1. Nécessité : attendre un résultat sans polling. Conserver la remise normale. Distinguer état de travail, état de boîte et accusé de lecture.
2. Simplicité : réutiliser les stores, handlers, identités, fixtures et chemins de remise. Le compteur partagé règle la course de réveil. Aucune abstraction supplémentaire n'est nécessaire pour les deux corrections.
3. Hypothèses : identité attestée, résultat stable après publication, premier résumé answered figé, socle150 conservé. Les barrières rendent explicite l'entrée effective dans l'attente.
4. Vérifié : contratsSpecKit, réalisation actuelle, assertions des tests, journaux existants et conformité des47fichiersfigés. Relecture complète des trois tests corrigés et du cheminJSONL concerné.
5. Non vérifié : aucun nouveau test exécuté. Aucun modèle réel, activation de production, attente complète3600s ou passage effectif par EINPROGRESS sur macOS. Certaines longues sorties de lecture ont été tronquées ; elles ne valent pas lecture intégrale des fichiers concernés.
6. Conservé : compteur et condition variable contre les réveils perdus ; résumé durable contre le remplacement du ledger ; registre et callback pour fermer les sockets ; grâceEOF4,5s pour les appels historiques. Les trois barrières de test sont justifiées par la preuve du stimulus.

## Limites et résultats distincts

W004 reste2046PASS/8FAILhérités150/72ignored,RC101. T027 reste une preuve distincte1PASS. Aucun totalglobal2047,W005 ou succès global de format déduit. FormatRC1/549différencesanciennes, sans nouveauhunk annoncé.

Conclusion : CONVERGED après audit. Aucun append dans les tâches. Aucun reçu créé pendant la phase6.
