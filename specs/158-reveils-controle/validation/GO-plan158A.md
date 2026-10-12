# GO du plan 158A
Le principal autorise l'implémentation le 12 octobre 2026.
Base bd37e927eee0b28f058df9d6a6992e38665a6ad0, livraison157 terminée.
Revue adverse R1 APPROVE_WITH_CHANGES : les sept points sont résolus dans les clarifications appliquées.
Le champ cancelled reste compatible et désigne les arbres ciblés avant commit, pas les arrêts physiques confirmés.
CallerScope réemploie le RPC natif et l'index existant ; aucun accès SQLite MCP ni nouvel outil.
Delegate et Continue composent halt/pause après rejeu ; les demandes enfant déjà admises gardent leur règle de pause.
Le callback des cibles ne s'exécute jamais au rejeu. Le verrou daemon reste tenu jusqu'aux demandes d'annulation.
La classification couvre les 30 outils ; les annotations dérivent de la base et journal null diffère de chaîne vide.
Les canaux de refus et la garde ControlStateV1 sont explicités.
Analyze : aucun finding bloquant ouvert. Réemploi : PASS 14/14, aucun doublon.
Baseline fraîche T001 : 253 PASS, 0 FAIL, 2 ignored ; 255 noms exacts, 217 sources inchangées.
22 tâches et 44 noms de tests prévus conservés ; section158B inchangée.
Le script canonique check-prerequisites.sh est absent (RC127), métadonnées et chemins vérifiés directement.
Aucun test158 ni build158 revendiqué à ce stade. T001 seule est cochée.
Estimation totale maintenue : 250–435 minutes, fin 14:55–18:00 CEST. 158B reste à planifier après A.
