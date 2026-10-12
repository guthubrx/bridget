# Réponses ROOT à la revue code158B R2

Brut lu : /Users/moi/.cache/bridget-docs158B-20261012/code-review158B-r2-result.json
Fournisseur Claude, modèle claude-haiku-5-5 ; tâche 158B-code-review-20261012-r2.
Verdict brut APPROVE_WITH_CHANGES, aucun défaut bloquant prouvé par la revue.
Les décisions ci-dessous sont celles de ROOT. Lecture ciblée, aucun code,
test, service ou fichier WT modifié par ce document.

## A — retenue comme précision de preuve

Le contrat agent demande l'absence de départ à expiration ; il n'impose pas
la finalisation terminale dans le callback. À daemon.rs:2479–2510, une garde
refusée sans begin_send_delivery laisse committed=None et rend Deferred.
À2196–2199, une lecture ultérieure avec now >= grace rend MissedBeforeDeparture ;
à2768–2804, le parcours CLI choisit Rejected puis appelle finalize.
Ces dernières lignes sont une preuve de lecture, sous réserve de stockage,
pas une assertion exécutée du passage ultérieur qui relit cette occurrence. G5 GREEN confirme uniquement
Deferred/no099/noFrame. Le patch documentaire rend cette limite explicite.
Aucune modification de phase, test, helper ou nouveau scénario nécessaire.

## B — non retenue, quatre frontières différentes

Les prélèvements frais à2231,2259,2393,2467 sont séparés par les transitions
CLI/102, l'acquisition writer et la réserve SQL. Réutiliser une valeur prélevée
plus tôt ferait perdre la fraîcheur au dernier puits. Un helper seulement pour
recopier le calcul est un raccord de confort ; aucune extraction demandée.

## C — non retenue, borne physique distincte des horloges de fixture

La production capture clock avant le calcul. Les fixtures peuvent fournir un
Instant futur ; G4/G5 utilise +900s (23930/23973) et une présence DND à+1800s
(23880). La borne min(clock+1s, mono_anchor+1s), à2115–2116, empêche ce clock
futur d'autoriser une attente physique de900s. Elle reste nécessaire aux
montages existants. Aucun changement des oracles ni du domaine d'horloge.

## D — triage corrigé : deux autres chemins retenus

Le checkpoint R2 initial réfutait le retard pour la boucle6565 seulement.
La conclusion ne valait pas globalement : Registered7090 et Availability21457
avaient Unix avant Mono. ROOT a retenu et corrigé ces deux appels avec un
return_clock local avant unix_now_secs, sans helper ni scénario nouveau.
Leurs lignes courantes ont été relues, ainsi que la boucle principale.

Dans les trois chemins désormais corrigés, clock précède Unix. wall_before
précède mono_anchor. Sous horloge civile continue,
lower = wall_before - (mono_anchor - clock) <= wall au clock, donc
floor(lower) <= floor(wall au clock) <= now et
current_now = now + floor(wall_fresh) - floor(lower) >= floor(wall_fresh).
La borne peut expirer trop tôt. Aucun retard garanti de1s n'est revendiqué.
Time/SQL non atomique et horloge civile susceptible de changer restent les
limites. La nouvelle preuve ciblée E couvre le source2124, pas une nouvelle FULL.

## E — empreinte GREEN vérifiée par le reçu d'exécution

Reçu lu : /Users/moi/.cache/bridget-tests158B-20261012/T109-departure-clock-GREEN-affected.json
RC0 ; quatre PASS ; 28,82 s. source_hashes_before et source_hashes_after donnent
pour daemon.rs 42a95a02f22c7b223eb6d433bab741f7e4722d771dd05f204f9331685a0316ec,
égal au fichier courant au checkpoint R2 initial. Journal SHA256
f7b697e7f436f79678b86f181fd0e0ba1a60c2dfa31ed85f5d24cf05cbe429e3.
Candidat lu : /Users/moi/.cache/bridget-code158B-natif-20261012/departure-clock-correction/GREEN-daemon.rs
SHA256 2a1490af0284174cbec5f0663caf4820d8c57b71caf5751b9b353e73d3a9ac5a.
Son manifest adjacent dit no_execution=true : c'est une préparation. ROOT
attribue l'écart au formatage final. L'empreinte attestée exécutée est 42a95a02 ;
la conclusion ne repose pas sur la seule présence de marqueurs ou sur les dates.
Aucune comparaison intégrale des deux fichiers n'est créditée à cet owner.

## Lecture et limites

Source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs :
2090–2135,2190–2207,2220–2285,2385–2513,2710–2809,24055–24108,6545–6572.
Ancres +900/+1800 vérifiées par recherche ciblée ; manifeste de préparation lu.
Le brut R2 a été lu intégralement. Le reçu GREEN a été lu par champs ciblés,
pas reproduit intégralement. Aucun tick final de CLI à l'expiration exécuté ici.
La FULL unique et ses échecs restent historiques ; aucun GREEN global,
Converge final, cochage ou livraison annoncé par ces réponses.


## Correction des deux retours et preuve finale ciblée

Le checkpoint R2 précédent sur daemon42a95a02 et son GREEN28,82 s reste
historique. La réfutation du retard sous horloge continue ne couvrait alors
que la boucle principale. ROOT a trouvé deux autres appels observer_return_tick,
Registered et Availability, qui évaluaient Unix avant Mono. Ils sont corrigés
par return_clock local capturé avant unix_now_secs, sans helper ni scénario neuf.
Le raisonnement de borne conservatrice s'applique désormais aux trois chemins
relus. Time/SQL non atomique et saut d'horloge civile restent des limites.

Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.log
SHA256 : a7fda2d6fa2a47e7556e4ef18fdef988b7ddb36552dbd727e48afe6ce982b5fd.
22,51 s ; RC0 ; **4 PASS, 0 FAIL, 0 ignored** ; assertions 1,84 s.
221 sources avant/après strictement identiques. SHA daemon exécuté et courant :
2124b1d0b42c3232698269918adf9f9df012a7aa8dae07cac98919f0c82064a5.
Les mêmes oracles G4/G5, G6, G8/G10 et CLI G7 passent. Ce lot ne prouve pas
la finalisation d'une occurrence par un passage ultérieur qui la relit.
La page CLI de 128 entrées ne garantit pas la relecture au prochain tick.
Aucun second FULL, succès global, nouveau scénario ou livraison annoncé.
