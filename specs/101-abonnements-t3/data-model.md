# Données 101

- Identité native temporaire : PID + naissance + ID session natif → unique fil
  T3 actif → UUID Bridget + instance + preuve099 existante. Aucun secret nouveau.
- Capacité source : connexion primaire, agent, types de faits annoncés. Valide
  seulement tant que cette connexion reste vivante. Collision dérivée d'écriture.
- Fait T3 : ID activité/tour stable, type connu, chemin facultatif, origine
  notification système éventuelle ; écrit dans le journal existant.
- Abonnement : champs100 conservés ; état actif/source indisponible/interrompu
  après redémarrage. TTL absolu ; propriétaire inchangé. Interrompu ne déclenche
  plus de faits jusqu'à nouvel abonnement explicite.
- Persistance : tableau borné d'abonnements dans Store, pas de transcripts,
  ni de file durable de messages. Corruption/échec de sauvegarde = refus explicite.

Transitions : création → actif → source indisponible → actif ou expiration ;
redémarrage → interrompu ; désabonnement/expiration → supprimé ; once déclenché
→ consommé. La consommation et la livraison sont deux faits distincts.
