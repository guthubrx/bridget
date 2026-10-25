# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/implementation-158B.md](../../../../../../specs/158-reveils-controle/implementation-158B.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Nécessaire : FR101–107/US5–US7 exigent programmation, déclencheur local, provenanceonce et résumé corrélé. Le post102 déjà committé est retrouvé avant grâce/gardes de création pour éviter un double effet ou faux refus à la reprise.
2. Simple : tick1s, worker64, Store, garde referent_control, route/identité149, négociation humaine et registres099/102 sont réemployés. Trois modules/tables de domaine justifiés, aucun service/thread/dépendance/outbox/registre de quota supplémentaire. Les helpers ont leurs usages réels ou une contrainte écrite.
3. Hypothèses : owner et instance attestés, données lisibles, projet/membership revalidés au départ, heure serveur, fuseau dans le domaine minutes±24h. ADR011 mêmeUID reste une limite. Aucun libellé/body n’est pris pour autorité.
4. Vérifié : delta entier normalisé, trois nouveaux modules/tests entiers, canon avant effet, Immediate/CAS, bornes physiques16/128 et8192/64, pagination20/64/65536B, JSON16KiB, worker1s/deadline5s, épisodes et cutoffs distincts. ACKcurrent UUID/delivery/instance/génération et retour afterACKOk mêmeanswered_requestNone sont relus. Hashes du manifeste concordent au début/fin.
5. Non vérifié : aucun test/build/Cargo/DB/provider exécuté par ce lecteur. FULL et rejouages sont des preuves ROOT lues, pas des exécutions de cet agent. Aucun modèle réel/T3/activation/performance/couverture/mutation validé. Les 7 résumés enfants sont exclus du compteur Cargo.
6. Conservé/suppressible : conserver provenance avantonce, canon/UUID102/issued_at, horizon7j, phasesunknown et cutoffs nécessaires au crash. Aucune ligne métier supprimable prouvée. Le dépôt sans avis ET sans snapshot ne porte aucun effet : éviter seulement sa TX et ses scans. SnapshotSome doit rester committé même avec zéroavis. Aucun nouveau helper/scénario n’est nécessaire.

<a id="extrait-2"></a>

## Extrait 2

Nécessité : conserver la borne conservatrice Wall/Mono utilisée au vrai départ, aussi aux retours Register/DND. Simplicité : deux captures locales, aucun helper, API, type, état, calendrier ou dépendance neuf. Hypothèses inchangées : horloges serveur exploitables et preuves lisibles ; sinon refus sûr. Preuve : deltas exacts et trois appels relus, puis reçu ROOT frais. Limites : aucune nouvelle exécution de cet agent, aucun modèle réel/T3/performance/activation validé. Canon, issued_at, cutoffs et tous anciens oracles sont conservés.
