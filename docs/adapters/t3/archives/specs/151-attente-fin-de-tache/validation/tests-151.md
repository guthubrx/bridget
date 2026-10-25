# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/validation/tests-151.md](../../../../../../../specs/151-attente-fin-de-tache/validation/tests-151.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les 42 tests ciblés du daemon ont réussi dans V005. Une vérification ultérieure
de FR-034 a d'abord utilisé une attente erronée. Le contrat réel 150 rend
`result: null` hors `result_available`. R006 puis V009 prouvent la correction
du test et la conservation de ce comportement, avec 3 PASS. Les six régressions
finales de T021 sont terminées : 177 PASS, 0 FAIL et 1 ignoré, comme la référence.
Le premier passage complet W001 donne 2029 PASS, 10 FAIL et 72 ignorés,
code retour 101, après recomptage des 94 cibles Cargo. Il reste non final :
huit échecs sont hérités de 150, deux proviennent de tests dont le périmètre
doit suivre les ajouts 151. FR-038 a
ensuite une reproduction rouge R007, puis un GREEN ciblé V010. Les deux
fixtures ajoutées sont vertes dans V013/V014.
le full post-EOF W002 est terminé : 2030 PASS, 12 FAIL et 72 ignorés sur
94 cibles, code retour 101 et neuf cibles en échec. La relecture indépendante
après correctif n'avait confirmé aucun défaut. W002 révèle toutefois quatre
échecs nouveaux : contenu089, isolation089, preuve d'identité T3 et message
unique E2E151. R009 confirme la perte de réponses ordinaires à l'EOF. D001
observe séparément l'échéance trop courte de la fixture E2E. V015 à V019
prouvent ensuite la grâce EOF compatible. E010/E011 prouvent la correction
E2E et le maintien de l'ancien148. Les quatre FAIL nouveaux ont leurs GREEN
isolés ; W002 reste une suite en échec. Le principal clôt T022 avec « échecs
expliqués » et T026, puis autorise le snapshot T023 : 23 cases sur 26,
T023/T024/T025 encore ouvertes à cet instant historique. Le principal vérifie
et coche ensuite T023, puis T024 après le RED ACK. A001 compile puis donne
0 PASS / 11 FAIL ; A002 donne 11 PASS / 0 FAIL sans warning. Les régressions
post-ACK `--lib` donnent 58 PASS ciblés151 et 167 PASS hérités, sans échec ou
ignoré. E012 apporte quatre E2E verts et un ignoré ; V020 apporte quatre
permissions Claude vertes. W003 est terminé en échec : 2045 PASS, 9 FAIL,
72 ignorés, RC101 sur 94 cibles Cargo de premier niveau. Huit FAIL sont connus150 ;
le neuvième est l'assertion T025 du catalogue MCP, attendu26 contre réel27.
Le seul littéral attendu passe ensuite de 26 à 27. V021 donne 79 PASS,
0 FAIL et 1 ignoré sur MCP complet. W004 est terminé : 2046 PASS, 8 FAIL,
72 ignorés, RC101 sur 94 cibles. Les huit noms correspondent exactement à la
référence150. La bibliothèque daemon entière donne 1347 PASS et 14 ignorés.
Le principal coche alors T025 : 26/26 avant la nouvelle T027 de Converge 1.

<a id="extrait-2"></a>

## Extrait 2

| Preuve | Passe | PASS | FAIL | Ignorés | Portée |
|---|---|---:|---:|---:|---|
| R001 | Protocole rouge | 1 | 3 | 0 | Les variantes historiques étaient déjà stables ; les trois ajouts échouaient. |
| V001 | Protocole vert | 4 | 0 | 0 | Octets historiques, aller-retour des trames, ancien rapport sans features et champs fermés. |
| R002 | Stockage et façade rouges | 1 | 15 | 0 | Six tests de résumé figé échouent avant migrations et transition atomique. |
| R003 | Cœur rouge, travailleurs verts | 20 | 11 | 0 | Sept tests de stockage verts ; vue, boîte et attente encore rouges. |
| R004 | Cœur vert, attente de réponse rouge | 32 | 6 | 0 | Quatre tests daemon d'envoi et deux échanges façade échouent avant finalisation. |
| V002 | Première passe globale ciblée | 36 | 2 | 0 | Deux échanges façade encore rouges ; ce relevé est historique. |
| V005 | Passe ciblée initiale consolidée | 42 | 0 | 0 | Cœur, stockage, façade, compteur et regroupement. Antérieure au contrôle supplémentaire de FR-034. |
| V006 | Filtre store::tests | 50 | 0 | 0 | Comprend aussi artifact_blob_store et receipt_store ; durée 1,15 s. |
| V007 | Filtre idempotency::tests | 46 | 0 | 0 | Régressions de remise et de résumé ; durée 0,29 s. |
| E002 | Première reprise E2E, expiration | 0 | 1 | 0 | Assertion de livraison échouée ; durée 3,23 s. T013 ouverte. |
| E003 | Bout en bout après correction | 3 | 0 | 1 | Extrait authentique archivé de la sortie d'outil ; durée 11,89 s. |
| E004 | Rejeu EOF renforcé | 1 | 0 | 0 | Journal intégral : quatre attentes de 3600 s fermées en 11 ms ; stdout final vide. |
| R006 | Contrat réel 150, test correct rouge | 0 | 1 | 0 | Le résultat privé hors result_available était rendu à tort pendant le correctif intermédiaire. |
| V009 | Contrat réel 150, vue corrigée verte | 3 | 0 | 0 | Champ result filtré comme en 150 ; durée 0,13 s. |
| E005 | Scénario d'avis d'échec rouge | 0 | 1 | 0 | Fixture sans branche d'échec : résultat au lieu de failed ; durée 4,21 s. |
| E006 | Scénario d'avis d'échec vert | 1 | 0 | 0 | pending -> claimed -> delivered, un avis natif, nettoyage durable ; durée 6,40 s. |
| E007 | Trois E2E 151, reprise finale | 3 | 0 | 0 | Deux tests filtrés ; durée 13,83 s. Inclut résultat, échec et EOF. |
| E008 | SC-001 ancré sur la capture SQLite | 1 | 0 | 0 | Borne antérieure à la capture -> ready : 978 ms ; capture encadrée sur 6 ms. |
| E009 | Trois E2E 151, ancrage SQLite final | 3 | 0 | 0 | Deux tests filtrés ; durée 12,69 s. SC-001 : 980 ms ; EOF : 10 ms. |
| S001 | SC-008, SELECT au calme | 2 | 0 | 0 | TaskWait : [0, 5000, 10001] ms ; RequestWait : [0, 5002, 10002] ms. Durée 10,09 s. |
| S002 | SC-008, contrôles sous charge | 1 | 0 | 0 | 64 attentes ; Status 2,430 ms, Cancel 19,056 ms. Durée 0,09 s. |
| W001 | Premier passage complet du workspace | 2029 | 10 | 72 | 94 cibles Cargo, code retour 101, sept cibles en échec. Bilan non final. |
| R007 | EOF mixte Wait + Status bloqué | 0 | 1 | 0 | Fin en 10,000712583 s, au-delà des 6 s de FR-038. Durée 10,05 s. |
| R008 | Callback avant connect, rouge | 0 | 1 | 0 | Backlog Unix réel plein sur macOS : callback absent, 0 observation. |
| V010 | MCP après correctif EOF | 75 | 0 | 1 | EOF mixte inférieur à 1 ms ; notification nonwait préservée pendant connexion, authentification et opération. Durée 1,25 s. |
| V011 | Connecteur après correctif EOF | 19 | 0 | 0 | Budgets et gardes du client conservés ; durée 3,01 s. |
| V012 | JSONL après callback avant connect | 3 | 0 | 0 | Callback observé une fois avant connect ; annulation synthétique avant dial en 14 microsecondes. |
| V013 | Liste fermée Claude corrigée | 1 | 0 | 0 | Les 22 outils attendus restent vérifiés exactement ; durée 1,26 s. |
| V014 | Exemples publiés138 et RequestWait151 | 1 | 0 | 1 | Vrai MCP et demandeur initial, réponse connue, six exemples138 conservés ; durée 2,93 s. |
| W002 | Full post-EOF terminé, non final | 2030 | 12 | 72 | 94 cibles Cargo, code retour 101, neuf cibles en échec. Huit FAIL hérités et quatre nouveaux. |
| R009 | EOF perd une réponse ordinaire rapide | 0 | 1 | 0 | Backend Status authentifié, réponse après 100 ms absente ; initialize id1 seul, id2 absent. Durée 0,13 s. |
| V015 | MCP après grâce EOF compatible | 76 | 0 | 1 | Réponse id2 transmise après EOF en 105 ms ; mélange Wait3600/Status muet fermé en 4504 ms. Durée 4,66 s. |
| V016 | Contenu089, banc inchangé | 7 | 0 | 0 | Le client réel conserve sa réponse après EOF ; durée 2,49 s. |
| V017 | Isolation089, banc inchangé | 8 | 0 | 1 | Namespace privé et client réel conservés ; durée 0,67 s. |
| V018 | Identité T3, banc inchangé | 1 | 0 | 1 | Preuves partielles refusées sans réponse perdue ; durée 0,12 s. |
| V019 | Connecteur après grâce compatible | 19 | 0 | 0 | Budgets et gardes conservés ; durée 3,01 s. |
| E010 | Trois E2E151 après délai de fixture corrigé | 3 | 0 | 0 | Deux filtrés ; SC-001 945 ms, capture encadrée sur 7 ms ; EOF quatre attentes en 11 ms. Durée 6,41 s. |
| E011 | Ancien148, délai de fixture maintenu | 1 | 0 | 0 | Quatre filtrés ; recette historique conservée à 2 s, durée 4,60 s. |
| A001 | RED ACK runtime avant production | 0 | 11 | 0 | Compilation réussie ; huit tests cœur et trois façade échouent sur opération, champ ou outil absent. Durée 2,04 s. |
| A002 | GREEN ACK cœur et façade | 11 | 0 | 0 | Compilation sans warning ; rollback UPDATE/COMMIT, premier timestamp durable, refus ancien en 53 ms. Durée 0,41 s. |
| R010 | E2E, assertion erronée sur une date publique | 0 | 1 | 0 | Status rend la boîte acknowledged ; le test attendait hors contrat un champ acknowledged_at public. |
| V020 | Permissions Claude post-ACK | 4 | 0 | 0 | Catalogue fermé vérifié ; durée 1,40 s. |
| E012 | E2E post-ACK, exemples publiés exécutés | 4 | 0 | 1 | Trois cas151 et ancien148 ; TaskWait publié600 puis Ack publié ; date privée confirmée par SQLite. SC-001 971 ms, capture encadrée sur 5 ms, EOF quatre attentes en 14 ms. Durée 6,49 s. |
| W003 | Workspace post-ACK terminé, historique | 2045 | 9 | 72 | 94 cibles Cargo de premier niveau, RC101. Huit FAIL connus150 et un oubli d'oracle T025, attendu26 contre réel27. |
| V021 | MCP complet après correction d'oracle | 79 | 0 | 1 | Seul littéral attendu26 remplacé par27, aucun autre code changé ; EOF Wait3600/Status muet ferme les deux sockets en 4505 ms. Durée 4,65 s. |
| W004 | Workspace final après correction d'oracle | 2046 | 8 | 72 | 94 cibles Cargo de premier niveau, RC101. Ensemble des huit noms identique à la référence150 ; aucun nouveau FAIL151. Bibliothèque daemon entière : 1347 PASS / 0 FAIL / 14 ignorés. |
| S003 | T027, SC-009 après ACK normal | 1 | 0 | 0 | Preuve distincte de W004 : scopeA406 µs, scopeB357 µs, résumés exacts puis stables, collision du ledger, capacité libérée ; durée0,07 s. |
