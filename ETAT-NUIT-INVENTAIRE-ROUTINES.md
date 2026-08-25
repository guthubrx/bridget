# Lot routines — inventaire des conditions contre le diff complet

Établi par relec5 (polarité constructive) sur `45ffa11..9f7bcc6`, avant la
panne de shell. C'était l'office qu'on lui avait confié : à sept SHA en trois
heures, **vérifier qu'aucune condition du jury ne s'était perdue en route**.

## LA CHAÎNE EXACTE DES SEPT TÊTES

```
45ffa11  (base jugée en manche 4, BLOCKED unanime)
f85af9b  Clôturer les occurrences et corriger la garde B3
8a8b90f  Afficher le hash recalculé et exercer la garde de rejeu
09ab0d6  Adopter les mandats orphelins et afficher les six champs
061e773  Intégrer les oracles relec1 v2/v3 du crash réel   ← INTRODUIT LE DÉFAUT
f837ed0  Rendre l'écran d'approbation vérace et chiffrer la borne
9f7bcc6  Remplacer RELEC1_CRASH par une injection de test   ← LA VRAIE TÊTE
```

**⚠️ NE JAMAIS MERGER `061e773` NI `f837ed0`** : ce sont eux qui portent
l'interrupteur de perte de mandat (`RELEC1_CRASH`, lu à l'exécution, sans
garde de compilation, présent dans le binaire de production — `strings` le
trouve). Le défaut est né du remède lui-même en `061e773` et n'est levé
qu'en `9f7bcc6`, où la variable d'environnement est **supprimée** au profit
d'une injection par paramètre.

## LES DIX POINTS — AUCUNE CONDITION ABANDONNÉE

| # | Condition | État |
|---|---|---|
| 1 | B3 câblage : hash recalculé depuis les champs relus, garde dans le domaine, compare-and-swap sur l'état et le hash | **présent, mesuré** |
| 2 | B3 écran : six entrées scellées affichées + les deux empreintes + refus AVANT écran | **présent, mesuré** |
| 3 | Clôture d'occurrence dans la transaction de clôture d'objectif + rattrapage idempotent | **présent** |
| 4 | Oracle de panne qui COMPTE (exactement trois) | **présent** |
| 5 | Oracle de rejeu qui EXERCE la garde `load_occurrence` | **présent** |
| 6 | Oracle pause/resume sans rattrapage | **présent** |
| 7 | Borne de rattrapage + sentinelle chiffrée + motifs distingués | **présent** |
| 8 | Engloutissements silencieux retirés | **présent** |
| 9 | Contrat CLI des six sous-commandes | **TESTS oui, DOMICILE non** — `plugins/maicie/README.md` contient zéro occurrence de « routine ». Seul point ouvert de la liste. |
| 10 | Remède du mandat adopté + ses deux oracles intégrés | **présent et non décoratif** — le mutant qui retire le remède fait mourir les trois oracles, contrôle positif vert |

**Conclusion de l'inventaire** : la cadence n'a rien fait *perdre* — elle a
fait *ajouter* un défaut. C'était le risque exact qu'on demandait de
surveiller, sauf qu'il s'est matérialisé en régression plutôt qu'en oubli.

## Ce qui manque avant merge (déclaré, non maquillé)

- **Rejouer l'exploit B3 sur la tête réelle** si l'on exige une double main :
  les épreuves du relecteur portent sur `09ab0d6` et `9f7bcc6`, jamais sur
  `061e773` qu'il n'a pas compilé.
- **Un compte de suite complète sur `9f7bcc6`** : les deux comptes existants
  (866/4/16 et 876/4/16, `--no-fail-fast`) portent sur `8a8b90f`.
- Les **trois bancs de relec1** (voir `ETAT-NUIT-ANGLES-A-INSTRUIRE.md`),
  dont le premier peut rouvrir un motif déjà refermé.

## Les trois ajouts du verdict (APPROVE_WITH_CHANGES sur `9f7bcc6`)

1. **Reformuler la dette** — bascule en BLOCKED si elle reste dans ses termes
   actuels. Ce n'est pas « l'occurrence reste ouverte » : c'est, mesuré sur
   dix relèves, **la routine cesse définitivement de tourner, en silence,
   pendant que `routine list` affiche toujours `active`**.
2. Domicilier le contrat CLI des six sous-commandes (point 9 ci-dessus).
3. Un oracle sur le refus AVANT l'écran — mutant prouvé : 76 tests restent
   verts quand on retire la ligne ; contrôle positif fourni.
