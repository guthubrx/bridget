# ADR 009 — GUI Bridget : plan de contrôle propriétaire, client progressif

**Date** : 2026-08-23
**Statut** : Accepté
**Instruction** : deux reconnaissances en lecture seule du dépôt T3 Code
(clone local `~/11.Repositories/t3code`), la première sur v0.0.21 (avril),
la seconde — après détection d'un retard de 1 385 commits — sur b1670ac7d
(2026-08-23). Conclusions convergentes.

## Contexte

Objectif produit : une interface graphique « équipe d'agents » (barre latérale
des équipiers, fils de conversation inter-agents repliables, vue intérieure
des tours avec outils dépliables, panneau missions/routines, composer humain),
à parité de vue avec le terminal (`attach`) quel que soit le mode d'attelage
(géré ACP ou interactif tmux).

Bridget possède déjà le plan de données : journaux de session (v1, streamables
par `Subscribe`), ledger des messages liés, annuaire `who`, greffe le service compagnon.
T3 Code (MIT, T3 Tools) offre une GUI desktop/web mûre pour agents de code,
parlant ACP nativement.

## Décision

1. **Bridget/le service compagnon restent le plan de contrôle et l'unique source de
   vérité** (sessions, identités, journaux, approbations). Aucune GUI ne
   reçoit d'autorité sur le cycle de vie des agents.
2. **Construire une GUI propre, progressive** — lecture/observation d'abord
   (équipe, fils, journaux), écriture ensuite (composer), sur un contrat
   Bridget dédié (Subscribe + ledger + who, puis abonnements).
3. **T3 Code sert de référence UX et de carrière de matériaux MIT** :
   réutilisation par emprunt ciblé de code source (ex. `packages/effect-acp`,
   patterns de rendu), jamais par fork du produit.
4. Un **fork très court reste admis comme POC jetable** pour éprouver un
   pont ACP, sans promesse de maintenance ni de GUI commune.

## Options rejetées

- **Adaptateur T3 externe (sans fork)** : aucun point d'injection supporté —
  le runtime ne charge qu'un catalogue de drivers compilé
  (`builtInDrivers.ts`, driver inconnu → snapshot « unavailable ») et le
  client n'a ni système de plugins ni slots de panneaux (routeTree générée,
  composition directe). Le contrat `ProviderDriverKind` ouvert n'est qu'une
  compatibilité de stockage.
- **Fork superficiel comme produit** : exigerait un driver serveur TypeScript
  + une UI sans slots, sous une dérive upstream de ~1 589 commits en quatre
  mois — il dégénère mécaniquement en fork profond.
- **Fork profond** : transférerait à T3 la propriété du cycle de vie des
  sessions, en conflit direct avec le daemon ; double autorité inacceptable.

## Conséquences

- Positives : source de vérité unique multi-canal (terminal et GUI rendent le
  même read-model) ; indépendance vis-à-vis de la cadence upstream ; emprunts
  MIT ciblés sans dette de merge ; la GUI peut avancer par tranches utiles.
- Négatives : le rendu « vue intérieure » est à notre charge (atténué par les
  emprunts et par un socle de chat existant) ; un POC de pont ACP éventuel
  est du travail jetable assumé.
- Prérequis nourriciers (catalogue v2) : session 014 (observabilité),
  journal enrichi, traducteur des sessions interactives, abonnement
  ledger/who, agrégateur local.
