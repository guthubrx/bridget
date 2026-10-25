# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/implementation.md](../../../../../../specs/151-attente-fin-de-tache/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les journaux transmis ensuite prouvent le GREEN compatible : MCP 76 PASS et
1 ignoré ; contenu089 7 PASS ; isolation089 8 PASS et 1 ignoré ; preuve
d'identité T3 1 PASS et 1 ignoré ; connecteur 19 PASS. Les trois bancs clients
historiques restent inchangés. Status conserve sa réponse id2 après EOF en
105 ms, avec réception backend confirmée. EOF avec Wait3600 et Status muet
finit en 4504 ms et ferme les deux sockets. Les trois fixtures MCP unitaires
à entrée finie ont aussi leur forme originale restaurée et passent.
Ces passes ciblées ne réécrivent pas les échecs historiques de W002.

<a id="extrait-2"></a>

## Extrait 2

4. **Qu'est-ce qui a été vérifié ?** Six tests ont été vus rouges avant la
   production : migrations des deux ouvreurs, guichet, collision de scopes,
   coupe UTF-8 et atomicité de l'ACK. Sept tests sont ensuite verts ; le septième
   vérifie l'instantané filtré, la première réponse et l'absence d'écriture.
   Le test d'atomicité force un échec SQL : ni l'ACK ni l'état `answered` ne
   gagnent sans le résumé. Les filtres `store::tests` et
   `idempotency::tests` donnent respectivement 50 PASS et 46 PASS, sans échec.
   Le premier filtre inclut aussi les tests de blobs et de reçus.
   Relecture et contrôle d'espacement sont faits sur les sept fichiers.
   Le protocole a quatre tests verts. La première passe ciblée 151 a 42 PASS.
   La correction exacte de FR-034 a un vrai RED 0 PASS/1 FAIL, puis un GREEN
   de trois tests. T021 rejoue les six filtres complets : 177 PASS, 0 FAIL et
   1 ignoré, avec les mêmes noms et résultats que la référence. Les trois E2E
   151 donnent 3 PASS et deux tests filtrés, sans compter le test historique 148.
   La façade a aussi des régressions archivées : 14 PASS ciblés, MCP 73 PASS et
   1 ignoré, client 19 PASS, montage 5 PASS et 1 ignoré. Ces filtres se recouvrent.
   La fermeture EOF renforcée de quatre attentes de 3600 secondes est mesurée
   à 11 ms, puis à 10 ms dans le dernier rejeu. La preuve finale SC-001 ancre
   la mesure sur la capture SQLite, encadrée sur 6 ms : borne conservatrice
   de 980 ms, zéro Status avant ready. Le vrai ancien daemon refuse les deux
   attentes en 44,710/54,874 ms
   et ne reçoit que les sondes anciennes. Les constats Explore sont des lectures ;
   les compteurs viennent des journaux des exécutions coordonnées.
   SC-008 ajoute deux tests au calme et un sous charge. La trace mesure trois
   SELECT à environ 0, 5 et 10 secondes. Avec 64 attentes et un écrivain actif,
   Status prend 2,430 ms et Cancel 19,056 ms. Les réservations sont libérées.
   Le premier workspace complet donne 2029 PASS, 10 FAIL et 72 ignorés sur
   94 cibles ; le compte brut préliminaire de 2032 PASS incluait trois doublons.
   Ce bilan est non final. Le RED EOF mixte confirme FR-038 avec 0 PASS et 1 FAIL,
   délai observé 10,000712583 secondes contre la limite de 6 secondes.
   Après correction, le même EOF mixte prend moins de 1 ms. MCP : 75 PASS et
   1 ignoré ; client : 19 PASS ; JSONL : 3 PASS ; liste Claude : 1 PASS.
   Le callback avant connect a son RED 0 PASS/1 FAIL, puis le GREEN JSONL.
   Les notifications nonwait sont testées pendant connexion, authentification
   et opération. Le banc089 donne 1 PASS et 1 ignoré avec RequestWait publié
   réellement exécuté ; les six exemples138 et leurs assertions restent présents.
   Le full post-EOF donne 2030 PASS, 12 FAIL et 72 ignorés sur 94 cibles,
   code retour101. Le compte brut préliminaire de 2033 PASS/13 FAIL incluait trois PASS
   et le FAIL d'identité imbriqué une seconde fois. Huit FAIL sont hérités,
   quatre nouveaux. Le RED EOF rapide montre id2 absent malgré la réponse
   backend après 100 ms. Le GREEN compatible conserve id2 en 105 ms ; le
   mélange Wait3600 et Status muet finit en 4504 ms, sous les 6 s de FR-038.
   MCP : 76 PASS et 1 ignoré ; contenu : 7 PASS ; isolation : 8 PASS et
   1 ignoré ; identité T3 : 1 PASS et 1 ignoré ; connecteur : 19 PASS. Les trois bancs clients
   n'ont pas été modifiés. La lecture SQLite E2E observe séparément l'échéance
   de 2 s dépassée. La fixture151 à 15 s a ensuite trois E2E verts : SC-001
   à 945 ms, capture encadrée sur 7 ms, zéro Status ; SC-006 complet ; EOF
   à 11 ms. L'ancien148 reste vert séparément à son délai de 2 s.
   La comparaison de `execute_send` conserve exactement 6283 octets et le même
   SHA-256 dans les deux sources, selon les bornes d'extraction ci-dessus.
   L'ACK a un RED runtime de 0 PASS et 11 FAIL, compilation réussie, puis
   un GREEN de 11 PASS et 0 FAIL sans warning. Huit tests cœur et trois tests
   façade prouvent bornes, propriété du lot, états refusés, premier timestamp
   durable, rejeu, sauvegarde stale ou forgée et pureté des faits.
   L'échec du deuxième UPDATE et l'échec du COMMIT annulent tout le lot sans
   réveil. Un lot changé réveille une fois ; un rejeu ne réveille pas.
   L'ancien daemon refuse en 53 ms après la seule `DaemonIdentityRequest`.
   Les régressions `--lib` post-ACK donnent 58 PASS ciblés151 et 167 PASS
   hérités, sans FAIL ou ignoré. Les dix PASS et l'ignoré d'intégration de
   l'ancien T021 ne sont pas inclus dans ce lot `--lib`.
   Le GREEN E2E post-ACK donne 4 PASS et 1 ignoré : trois cas151 et l'ancien148.
   Les permissions Claude donnent séparément 4 PASS. TaskWait publié avec
   600 s et Ack publié sont réellement exécutés. SQLite confirme la date
   privée. SC-001 prend 971 ms, capture encadrée sur 5 ms, zéro Status.
   Quatre attentes de 3600 s se ferment sur EOF en 14 ms.
   W003 est terminé : 2045 PASS, 9 FAIL et 72 ignorés sur 94 cibles Cargo
   de premier niveau, code retour101. Huit FAIL sont connus du socle150.
   Le neuvième est un oubli T025 dans l'assertion du catalogue MCP : réel27,
   attendu26. Il reste distinct des huit échecs hérités.
   La correction de ce seul littéral, de 26 à 27, donne ensuite 79 PASS,
   0 FAIL et 1 ignoré sur le filtre MCP complet. L'EOF mixte Wait3600 et
   Status muet ferme les deux sockets en 4505 ms. Aucun autre code ne change
   dans ce correctif. W004 est terminé : 2046 PASS, 8 FAIL et 72 ignorés
   sur 94 cibles de premier niveau, RC101. L'ensemble des huit noms est
   exactement celui de la référence150. Aucun échec nouveau151 ne subsiste.
   La bibliothèque daemon entière donne 1347 PASS, 0 FAIL et 14 ignorés.
   Le principal coche alors T025 : 26 tâches sur 26 avant Converge 1.

<a id="extrait-3"></a>

## Extrait 3

Le patch cumulatif contient 33 fichiers suivis, avec150 et151. Le delta151
porte les 30 fichiers suivis différents du socle vérifié : 15 avec production,
11 de tests ou adaptations de format, trois documents et un sélecteur de
session. Le fichier T3 n'ajoute qu'un test dans son module existant. Trois
sources du cumul restent byte-identiques au socle150 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/desired_state.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/fleet.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/referent_control.rs
