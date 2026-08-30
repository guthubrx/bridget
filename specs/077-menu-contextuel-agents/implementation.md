# Implémentation - SPEC-077

Date: 2026-08-30
Statut: terminée
Branche: session-077-menu-contextuel-agents
Base: f532a1bcd805d764fc97c1a6f7225f2449178451

## Résultat

L’interface possède maintenant un seul menu contextuel d’agent. Les trois
points, le clic droit, la touche Menu et Maj+F10 ouvrent le même nœud, la même
matrice de sept actions et le même répartiteur.

## Changements productifs

### app.js

- préférences locales v1 pour épinglage, masquage et dernier horodatage lu;
- normalisation fermée, déduplication et volume borné à 500 entrées;
- projection pure des agents actifs, arrêtés et masqués avec ordre épinglé stable;
- matrice unique des sept actions et raisons d’indisponibilité;
- menu global avec groupes, rôles ARIA et navigation clavier;
- ancrage au bouton, au pointeur ou à la ligne selon le déclencheur;
- persistance commune des lectures manuelles et automatiques;
- maintien du menu, de la sélection et du défilement pendant un rafraîchissement;
- réemploi sans modification des gardes, confirmations, routes et verdicts SPEC-075;
- retour du focus sur un élément actionnable après une commande lifecycle.

### index.html

- section repliée Agents masqués avec compteur et liste récupérable.

### theme.css

- menu compact sans bordure, ombre ou gradient;
- états focus, indisponible et destructif lisibles;
- logo fournisseur sans fond ou cadre parasite;
- présentation cohérente des sections arrêtées et masquées.

## Réutilisation

Aucune route, dépendance ou couche de service n’a été ajoutée. Le code réutilise:

- identityCardData et le catalogue runtime existant;
- identityCardPosition;
- agentLifecycleEligibility;
- openStopConfirmation;
- submitAgentStop;
- agentLifecycleFeedback;
- applyReadThrough et le rendu de flotte existants.

## Tests finaux

### Node

Commande:

```bash
node --test crates/bridget-daemon/assets/ui/app.js
```

Résultat: PASS, 93 tests passants, 0 échec.

### Rust UI

Commande:

```bash
/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib ui::tests:: -- --test-threads=1
```

Résultat: PASS, 53 tests passants, 0 échec.

### Suite Rust élargie

Commande:

```bash
/home/moi/.cargo/bin/cargo test -p bridget-daemon ui --lib -- --test-threads=1
```

Résultat: 134 tests passants et 1 échec environnemental hors UI, car le binaire
`bridget` de test est absent. La suite UI ciblée couvrant les assets modifiés
est entièrement verte.

### Hygiène du diff

- git diff --check: PASS;
- endpoint nouveau: aucun;
- dépendance nouvelle: aucune;
- fichier productif modifié: app.js, index.html et theme.css uniquement;
- jscpd: aucune duplication nouvelle.

## Livraison

Aucun commit, merge, push, déploiement ou redémarrage n’a été effectué.
