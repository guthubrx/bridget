# Référence T001159A avant code métier159

Un seul batch de neuf filtres OR plus la mesure prévue, puis transport. Pas de FULL préparatoire.
- Daemon initial : 541 PASS, 2 FAIL, 4 ignored, RC101, 261,69s. Les deux FAIL sont native149 O4 (groupe privé encore vivant à son court délai) et mesure latence (attente erronée Accepted au lieu de OutcomeUnknown).
- Native149 O4 : replay ciblé 1 PASS/0 FAIL, RC0, 116,00s dont115s compilation ; assertion/seuils inchangés. L'échec initial reste enregistré, sa cause précise reste incertaine.
- Transport : 360 PASS/0 FAIL/2 ignored, RC0, 131,79s. Deux résumés enfants de1PASS chacun sont exclus des360.
- Mesure : replay ciblé1PASS/0FAIL, RC0. Vingt échantillons dans un fournisseur Codex simulé, même instance et horloge monotone. p95 nearest-rank19 : 27.763792ms.
La mesure est émission turn/completed fournisseur vers réception turn/start suivant. Elle ne mesure pas l'instant de l'attestation dans le daemon. SC004 exige de distinguer cela lors de T010, jamais de présenter un timestamp fournisseur comme une lecture daemon.
Le worker privé existant utilise circuit10_000/180s, capacité4, keep_alive300s, poll1ms ; paramètres à garder identiques après. Le raccord existant BRIDGET_T908_MANAGED_EXECUTABLE est activé sous feature test-support uniquement. Sans la feature, la production conserve None. Aucun réglage held_queue ni code métier159 à ce stade.
Les traces JSON/logs CACHE font foi ; pas de GREEN global recomposé, pas de FULL relancée. Les cas échoués ont seulement été rejoués. T013 confrontera les mêmes filtres et une FULL finale.
Historique FULL158B : voir historique-full158B-pour159A.md/.json. Deux régressions DND corrigées ; huit FAIL hérités et deux oracles deadline099 historiques, dont un PASS au replay sans correctif. Rien ici ne vaut dix FAIL d'une FULL finale fraîche.
