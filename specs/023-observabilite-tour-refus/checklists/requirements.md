# Revue des exigences

- [x] Les deux propriétés décrivent un défaut observable, pas un geste imposé.
- [x] La présence positive est testée : un tour ouvert long reste occupé.
- [x] La reprise positive est testée : une nouvelle borne d'ouverture prime.
- [x] L'absence ou la corruption ne devient pas une preuve ; une attestation
  instantanée `busy` reste un contrôle positif distinct.
- [x] Un producteur sans borne terminale ne transforme pas `turn_start` en
  preuve durable d'activité.
- [x] La cause `state=busy` est établie par le code et le snapshot de l'incident.
- [x] Le domaine est explicitement exclu des causes décisionnelles de cette base.
- [x] La politique de refus pendant `busy` reste hors périmètre.
- [x] Aucun nouveau schéma, état durable ou événement n'est requis.
- [x] Les lots concurrents et leurs fichiers sont nommés.
- [x] Les sorties texte et JSON sont toutes deux couvertes.
- [x] Le témoin de journal corrompu exige une condition exacte et ne peut pas
  réussir seulement grâce à l'extension `.jsonl`.
- [x] L'activation reste sous la règle de la session 018 ; aucun canari de
  branche n'est installé.
