# Session 061 — Fermer l’usurpation d’identité par `--from`

## Métadonnées

- **Statut** : livré, non intégré
- **Branche** : `session-061-usurpation-emetteur`
- **Base gelée** : `9c6f47d` — tête de `session-060-identite-emetteur-cli`
- **Objectif** : `3d7053a6-e579-43eb-b5c0-5e72ddb27b17`
- **Délégation** : `eb448256-6e2f-41a3-9975-6f73ebcb25ab`
- **Message** : `a8412d35-385b-4ca1-859e-86d90ec7099c`

> **Cette session ne se branche PAS sur `main`.** Elle corrige du code introduit
> par la session 060, qui n’est pas intégrée. Partir de `main` produirait une
> branche où la fonction à corriger n’existe pas. `061` doit donc être intégrée
> après `060`, ou avec elle.

## Problème mesuré

`bridget send --from <nom d’un agent enregistré>` permet d’émettre sous
l’identité de cet agent. Le daemon accordait sa confiance sur une hypothèse
qu’il ne vérifie pas, écrite en commentaire :

> *CLI temporaire mais le from correspond à un agent enregistré → confiance
> accordée (le CLI tourne dans le contexte du wrapper)*

Rien ne l’établit : n’importe quel processus du même utilisateur peut passer
`--from humain`.

**Trois usurpations mesurées le 28/08**, toutes involontaires :

| heure | auteur réel | inscrit au ledger |
|-------|-------------|-------------------|
| 15:41:17 | `rc7-flux` (sonde T2) | `humain` |
| 16:37:24 | `rc7-flux` (sonde test B) | `humain` |
| 17:58 | le référent (vérification) | `humain` |

**Coût déjà payé.** `jc2-flux` a détecté le message de 16:37:24 sans réponse
depuis 72 minutes et a alerté à juste titre ; le référent a mis `rc7-flux` en
demeure de répondre à un humain **qui n’avait rien écrit**. La mesure était
juste, la prémisse était fausse, et la fausse prémisse venait d’une sonde.

La session 060 ne fermait pas cette voie : elle refusait un nom que *personne*
ne porte, et laissait passer sans contrôle le nom d’un agent existant.

## Propriété

`--from` ne permet plus d’émettre sous l’identité d’un agent qu’on n’est pas.

- CLI temporaire + `--from <agent connecté>` → **refus pour usurpation**.
- CLI temporaire + `--from <nom que personne ne porte>` → refus pour
  non-adressabilité. **Propriété de 060 préservée.**
- CLI temporaire **sans** `--from` → comportement d’origine, y compris le nom
  hérité de l’environnement d’un wrapper. Non-régression explicitement testée.
- Wrapper → son nom enregistré, comme avant.

## Portée et limite déclarée

La garde ne vise que le `--from` explicite. C’est la voie des trois usurpations
mesurées, et la seule qui se ferme sans casser d’usage existant : le chemin
implicite (`BRIDGET_AGENT_NAME`) reste ouvert et reste falsifiable par qui
contrôle l’environnement.

**Ce correctif n’est donc pas une garde de sécurité contre un adversaire.** Le
parc tourne sous un seul utilisateur ; quiconque peut lancer `bridget` peut
poser une variable d’environnement. Il rend l’attribution fiable contre l’erreur
et l’accident — ce qui est le mal constaté — pas contre une intention hostile.
Prétendre l’inverse serait le genre de faux succès que la session 060 a retiré.
