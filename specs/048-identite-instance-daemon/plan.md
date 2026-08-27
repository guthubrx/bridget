# Plan technique — Session 048

## Décision

Ajouter une valeur opaque générée par `Uuid::new_v4()` lors de
`DaemonState::new`. Cette dépendance est déjà déclarée dans le workspace ;
aucune persistance ni nouvelle dépendance n’est nécessaire. L’état conserve la
valeur, et le gestionnaire de `DaemonIdentityRequest` la reprend dans chaque
réponse Client ou Service.

`BUILD_ID` est volontairement laissé inchangé : il décrit le commit compilé et
ne peut pas distinguer deux démarrages du même binaire.

## Étapes

1. Écrire l’oracle rouge : deux listeners successifs sur le même socket et la
   même configuration doivent obtenir des identités distinctes.
2. Étendre le rapport filaire et son test de tour complet.
3. Générer, conserver et répondre l’identité depuis `DaemonState` ; mettre à
   jour la sonde de statut interne qui lit ce rapport.
4. Vérifier les rôles Client et Service, rejouer le mutant qui remplace la
   valeur de démarrage par une valeur stable, puis exécuter le banc ciblé.

## Fichiers prévus

- `crates/bridget-transport/src/protocol.rs`
- `crates/bridget-daemon/src/daemon.rs`
- `specs/048-identite-instance-daemon/*`

## Risques et contrôles

- Générer par requête donnerait une identité différente dans un seul daemon :
  l’oracle à deux connexions du même état le détecte.
- Réutiliser une valeur de compilation ou persistée ferait passer un
  redémarrage local-vers-local : l’oracle sur socket réelle le détecte.
- Oublier un rôle rendrait une garde future universellement refusante : le
  contrôle positif demande le rapport sur Client et Service.
