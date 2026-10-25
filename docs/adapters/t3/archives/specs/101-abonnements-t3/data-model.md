# Archives techniques de l’adaptateur T3

Document d’origine : [specs/101-abonnements-t3/data-model.md](../../../../../../specs/101-abonnements-t3/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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
