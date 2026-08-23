# Analyse de cohérence

**Résultat** : PASS.

La spécification, le plan et les tâches convergent vers un tunnel SSH inverse de socket Unix et réutilisent le daemon existant. Aucun conflit avec le protocole actuel n’est détecté.

**Validation réelle** : le 14 août 2026, un hôte Linux distant `user@exemple.tld:2222` a été enrôlé. Le socket distant a été créé, `bridget agents --json` a retourné l'annuaire du daemon maître et un message distant vers un agent local a été accepté (id `d81f9c411e1a4`).

**Précondition documentée** : le compte SSH distant doit autoriser `AllowTcpForwarding remote` et `AllowStreamLocalForwarding yes`.

**Validation de reconnexion** : un wrapper Linux nommé `reconnect-test`, exécutant `sleep 30`, a disparu de l'annuaire pendant le retrait du tunnel puis s'est réinscrit sous le même nom après sa réinstallation. Son PID Linux est resté actif pendant toute la coupure.
