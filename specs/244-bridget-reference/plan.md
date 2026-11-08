# Plan 244 — Bridget de référence

## Faits établis le 8 novembre 2026

- Hors T3, `bridget auto-delegate` refuse sans identité attestée
  (`identity_not_found`). Un shell ordinaire ne peut pas être parent.
- `bridget spawn claude --no-persistent --cwd <dépôt> --posture discovery`,
  lancé sur le maître (Cartae), crée un agent détaché, attesté, sans T3
  (`claude_stream_json`). Lancé depuis un shell non attesté du Mac, il refuse.
- La qualification Codex, la préparation de projet et les sondes passent par
  `managed-bootstrap` et des scripts intégrés au binaire (`include_str!`).
- Les codes de refus d'admission sont fermés ; leurs guidages vivent dans
  `daemon/native_admission_guidance211.rs` et `remote_worker.rs`.

## Architecture retenue

Un seul composant nouveau, la sentinelle, et des renforcements ciblés.

1. **Sentinelle (E1, E5)** : un minuteur utilisateur sur le maître (systemd)
   lance `bridget sentinel run`. Ce sous-commande :
   - lit le catalogue de flotte et la liste des machines actives ;
   - crée un agent détaché via le parcours `spawn` existant, modèle léger ;
   - lui envoie le plan de contrôle (projets, stratégies, machines, moteurs) ;
   - l'agent exécute les délégations réelles et renvoie un rapport fermé ;
   - la sentinelle publie le rapport durable (`~/.cache/bridget-core/sentinel.json`),
     arrête l'agent, et prévient le mainteneur une fois par incident.
   Déclencheurs : quotidien, fin d'activation (`deploy-*`), changement d'image
   programme détecté par l'observateur.
2. **Réparation (E2)** : dans `prepare-project.py`, un mapping connu sans racine
   additionnelle, alors qu'une racine identique est déjà approuvée pour la même
   source sur la machine, est complété avec révision+1 sous le verrou existant.
   Le contrôle d'égalité maître/worker reste la garde finale.
3. **Erreurs gardées (E3)** : `check_program_schema` et les autres contrôles
   lancent le bootstrap avec une sortie d'erreur capturée et bornée (4 Kio,
   filtrée) ; la cause publiée du catalogue la référence. Une qualification
   `pending` au-delà de 10 minutes devient `program_check_failed` nommé.
4. **Règle des refus (E4)** : un test énumère tous les codes fermés émis par
   l'admission et le worker, et exige pour chacun un guidage non vide avec
   action. La sentinelle classe un refus sans guidage comme défaut.

## Ordre de livraison (lots utilisables)

| Lot | Contenu | Livrable vérifiable |
|---|---|---|
| L1 | E3 erreurs gardées + délai borné | qualification ratée nommée avec sa cause |
| L2 | E4 test des guidages, compléments manquants | test vert sur la liste fermée |
| L3 | E2 réparation des mappings | ancien mapping complété au premier usage |
| L4 | E1 sentinelle manuelle (`bridget sentinel run`) | rapport réel sur les 4 machines |
| L5 | E1 déclencheurs + E5 alerte | minuteur actif, alerte unique prouvée |
| L6 | E6 communication sans T3 : sentinelle messages/fils entre machines, statut de panne clair, file conservée | rapport sentinelle messages vert ; panne simulée d'un worker réparée sans T3 |

La sentinelle (L4) contrôle dès sa première version les délégations ET les
messages entre deux agents détachés sur deux machines. L6 ajoute la réparation
automatique des composants et le statut de panne lisible par les agents.

Chaque lot : tests ciblés, build, activation quatre hôtes, essai réel, push.

## Risques

- Coût modèle de la sentinelle : borné par modèle léger, réponse fixe, un passage
  par jour et par changement.
- Faux positifs sous charge : refus de capacité consigné, pas une alerte.
- La sentinelle ne doit jamais rejouer une mission métier ni toucher aux tâches
  d'autrui.
