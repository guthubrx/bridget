# T010159A — matrice ciblée finalisée

Chaque cas prescrit dispose d’une preuve ciblée. Les réussites restent liées à leurs reçus et SHA. Aucun succès global n’est déduit. Aucun test ni signal exécuté par cet owner.

| Cas prévu | Test existant ou prescrit | Frontière / précondition | Oracle / état |
|---|---|---|---|
| daemon après rétention | held159_daemon_arrete_apres_retention_conserve_la_remise | SQL held après SendIdempotent attesté, avant toute libération ; parent réel + cible busy attestée + message ordinaire ; TERM daemon cfg(test) | phase held, id, génération, octets et rang identiques après reprise — PASS confirmé ROOT dans lot 1P/3F ; lot global RC101 |
| transaction libération puis arrêt avant push ; point21T007 | held159_release_panne_entre_transaction_et_push_relivree_par_la_reprise | after_held_release_commit ; 3 admissions réelles Held ; FIFO observée après COMMIT ; aucun writer après TERM | reprise dispatching, ordinary1→2→3, ACK réels ; id/génération/octets/rang préservés — PASS 2P/0F exacts avec ACK absence |
| daemon après vraie poussée socket | held159_daemon_arrete_apres_push_reprise_dans_l_ordre | after_idempotent_socket_push ET after_receipt_seen_before_injection ; deux ready réels ; push writeln+flush effectué, Seen durable, zéro injection ancienne ; arrêt daemon puis wrapper/provider individuels | nouvelle incarnation, ordinary1→2→3 et trois ACK réels ; identités et enveloppes conservées — PASS 1P/0F exact |
| daemon après ACK | held159_daemon_arrete_apres_accuse_ne_reinjecte_pas | after_delivery_acked ; ordinary1 acked et suivants2/3 Held ; cible native terminale volontairement absente à la reprise | aucune réinjection1 cible absente ; id/génération/octets/rang2/3 préservés ; relance explicite ensuite — PASS 2P/0F exacts ACK/busy final |
| daemon puis porteur nouvelle incarnation (SC015) | held159_daemon_arrete_apres_accuse_ne_reinjecte_pas (même scénario, pas test doublon) | ACK → daemon TERM → absence AgentList → RelaunchOrder réel ; mission terminale non rouverte ; nouvelle instance effectivement observée après relance explicite | ordinary1 ancien déjàACKED, ordinary2 puis3 nouvelle instance ; rang/id/génération/octets conservés — PASS partagé avec ACK |
| porteur sans message en cours | held159_porteur_arrete_sans_message_en_cours | AgentList connected après Carte bridget-reprise réellement completed ; SpawnOrder réel, aucune ordinary ; TERM porteur/provider prouvés, relance explicite nouvelle instance | seules cartes identifiées peuvent être terminées ; zéro ordinaire inventé — PASS confirmé ROOT dans lot1P/2F ; échecs ACK/busy exclus |
| porteur avec message en cours | held159_porteur_arrete_avec_message_en_cours_preserve_ordre | AgentList busy réel, seed initial en cours, deux Held ; Carte startup drainée seulement si from bridget-reprise ; seed ordinaire réellement démarré | nouvelle instance, ordre1→2 sans doublon ;2 resteHeld tant que1 réel actif ; identité/octets/rang figés — PASS 2P/0F exacts ACK/busy final |
| porteur après Seen durable, avant injection ; même incarnation | held159_porteur_arrete_apres_seen_reprise_meme_instance_indeterminee | after_receipt_seen_before_injection → after_delivery_indeterminate ; deux Held réels ; Seen intact ; daemon TERM avant wrapper ; bootstrap API réelle avec ancienne instance/commande/génération et nouveau PID/birth après daemon prêt | report indeterminate du vrai wrapper ; même namespace/reçu ; zéro injection1 et suivant Held à frontière ; id/génération/octets/rang préservés — PASS 1P/0F exact ; dernier montage validé |
| réglage fermé SC001 : message/remise/phase/push | daemon::presence_tests::held159_hold_ferme_identique + registry::held159_hold_ferme_identique | handler réel, registre privé, flags None puis false ; même message canonique et heure ; capturer vraie DeliverIdempotent via writer | canon, octets remise et phase dispatching identiques ; poussée réelle ; flux auxiliaire de réattestation séparé — PASS dans lot22P/0F |
| ordre3 / réaffectation SC003 | tests avant push et après push partagés | frontières COMMIT/push ; nouvelles incarnations réelles ; 3 messages ordinaires réellement retenus ; compteur Python réinitialisé distingué de id enveloppe | ordinary1→2→3 exact, aucune perte/duplication contrôlée ; id/génération/octets/rang stables — PASS via les deux cas de libération |
| latence SC004 / défaut fermé | held159_latency_baseline_codex_simule + held159_latency_retenue_codex_simule | 20 échantillons proxy Python completed→nextturnstart, rang19 ; worker performance10000 distinct worker crash8 ; after busy barrière réelle avant admission, hors mesure | 27.763792→1028.132375ms ; écart1000.368583ms >250 ; défaut reste false — sampler PASS ; seuil dépassé ; activation par défaut refusée |

Dernier Seen :1PASS/0FAIL/0ignored, RC0, 23.49s. Le daemon démarre avant le nouveau bootstrap. La même identité réelle est conservée ; le reçu Seen n’est ni effacé ni fabriqué. Le vrai wrapper émet indeterminate ; le suivant est Held à la frontière observée. Ce constat ne promet pas de bloquer un futur repos réel.

Le cas après push utilise deux barrières réelles. Il ne prouve pas exactement une injection pour toute panne après push. Après libération, une issue indeterminate reste prévue par R17.

SC004 utilise le proxy Python, pas l’horloge de repos du daemon. Avant27.763792ms, après1028.132375ms, écart1000.368583ms >250ms. Le défaut reste fermé. Le sampler after a duré54.72s. Aucune mesure production ni optimisation supplémentaire.

Les échecs de montage ne sont pas des RED métier avant implémentation. Un défaut métier distinct a été observé après les premiers GREEN T007 : ordinary1 stranded ancien destinataire, ordinary2 avant1. ROOT l’a corrigé après lecture runtime/SQL. Les assertions d’ordre restent strictes. L’échec Seen16 concernait un bootstrap vivant créé avant la réconciliation startup ; l’origine syscall exacte de l’EPERM n’a pas été instrumentée. Le montage17 est validé.

Matrice et reçus/empreintes : /Users/moi/.cache/bridget-code159A-T010-20261012/T010-matrice.json
Reçu final Seen : /Users/moi/.cache/bridget-tests159A-20261012/T010-Seen-daemon-before-real-READY.json
Log final Seen : /Users/moi/.cache/bridget-tests159A-20261012/T010-Seen-daemon-before-real-READY.log
SHA log Seen : 6d78f9a2ebb3368bf361cb045e90749fba05a7ae12ccf22f96d0137809b7aac6
Preuve de latence : /Users/moi/.cache/bridget-code159A-T010-20261012/SC004-latence-preuve.json
Dernier correctif Seen : /Users/moi/.cache/bridget-code159A-T010-20261012/17-seen-daemon-before-ready.patch

ROOT conserve le formatage, le gel et la suite complète unique. Aucun nouveau banc, scénario bonus, service ou code de production ajouté par cet owner.
