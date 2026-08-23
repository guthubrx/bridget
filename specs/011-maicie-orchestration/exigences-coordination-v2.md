# Exigences amont pour Maicie v2+ — jurisprudence d'orchestration

Extrait de la coordination manuelle des sessions 007-012 et 011 (2026-08-22/23),
tenue par le référent humain-agent. Chaque règle exécutée à la main ici est une
candidate à devenir un comportement produit de Maicie. Hors périmètre de la
v1 (spec 011) — intrants pour les itérations suivantes.

## Politiques de délégation observées (candidates à l'automatisation)

1. **Tout événement attendu a un messager.** Un commit, une fin de banc, un
   verdict sont des événements muets : personne n'est réveillé par git. La
   règle manuelle « livraison = hash annoncé par message » devrait devenir :
   Maicie observe l'attendu (par abonnement ACP ou déclaration) et notifie
   les dépendants elle-même.
2. **Déblocage des dépendants.** Quand une délégation se termine, la
   coordinatrice sait qui attendait quoi (graphe de dépendances des
   délégations d'un même objectif) et débloque sans qu'on le lui demande.
3. **Réassignation après N relances sans livraison.** Politique manuelle
   éprouvée : 2 relances factuelles sans commit → réassignation. Maicie
   pourrait porter cette politique par classe de délai (config T003).
4. **Contrat d'interface avant commit du fournisseur.** Quand deux
   délégations partagent une frontière, la coordinatrice fait négocier
   l'interface AVANT l'implémentation (observé : API store T006↔T008 —
   zéro STOP de review sur la frontière ensuite).
5. **Propriété des ressources partagées par phase.** Un fichier commun a un
   propriétaire unique par phase ; les autres passent commande. Maicie
   pourrait tenir ce registre de propriété par objectif.
6. **Vérification factuelle avant relais.** Aucun verdict/blocage relayé sans
   vérification indépendante (grep/exécution). Pour Maicie : ne jamais
   propager un état déclaré sans observable (transport_snapshot, fraîcheur).
7. **Diffusion de groupe.** Le transport reste point-à-point à dessein ;
   « informer tous les participants d'un objectif » est une sémantique
   Maicie (qui doit savoir, qui doit accuser) — pas une primitive transport.
8. **Escalade humaine bornée.** Les questions remontées à l'humain pendant la
   nuit : gates externes (SSH), arbitrages de périmètre, dérogations. Tout le
   reste s'est arbitré au niveau coordination — bon partage à conserver.

## Anti-patterns constatés (à ne pas reproduire dans Maicie)

- Polling aveugle (sleep + vérifier) au lieu d'attente événementielle.
- Relance d'un agent qui a déjà répondu par un canal non lié (cause :
  corrélation perdue — d'où l'exigence d'identifiants intégraux de bout en
  bout, réglée côté transport le 2026-08-23).
- Double réponse à une même demande après rappel.
