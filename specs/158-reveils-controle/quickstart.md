# Vérification rapide - Session158a

Toutes les commandes partent du worktree158A isolé dont ROOT renseigne le
chemin absolu après la livraison157 vérifiée. Le chemin150 cité auparavant
est historique. Ne pas exécuter la recette depuis cette préparationcache.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=2
```

Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.

## Scénarios à observer

1. Vue globale par le parcours natif Bridget, client agent refusé : filtre `control158_overview`.
2. Arrêt global (4 combinaisons avec la pause, reçus A-B-rejeu, annulation en cascade) : filtre `control158_halt`.
3. Portées MCP (table, plafond, fils en lecture et ACK, surface) : filtre `mcp158_`.
4. Non-régression : filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`.

## Recette manuelle (jamais sur le daemon de production)

À faire seulement dans un `BRIDGET_HOME` privé avec un daemon de test.

```sh
bridget control delegations --state all --json
bridget control halt-delegations --reason "essai"      # terminal interactif
bridget control status                                   # montre l'arrêt
bridget control resume-delegations
```

La recette reste strictement isolée. ROOT assure ensuite la livraison
Git/disque et le nettoyage selon le pipeline utilisateur. Activation Bridget
seulement sous la condition déjà autorisée de zéro mission non terminale ;
aucune relanceT3. Aucun lancement n'est fait par cette préparation.

## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.

## Extension158B après livraison158A

La partie B complète US5–US7. Son artefact de référence est /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/quickstart-158B.md. Les preuves et choix158A ci-dessus restent historiques et conservés.
