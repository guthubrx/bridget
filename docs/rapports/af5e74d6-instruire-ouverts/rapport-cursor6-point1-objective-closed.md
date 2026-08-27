# Point 1 — Les fermetures `objective_closed` sans preuve

Objectif `af5e74d6-293a-48b6-bd45-321d2957eab8` · base `08c0bd83b5d8cd38b4668d065defcf25b62da69a`  
Catalogue : **lu seulement** · IDs : `/tmp/bt/cursor6-objective-closed-ids.txt` (39 transitions / **35 constats uniques**)

## D'où ça vient (VERIFIE dans le code)

1. **Conception session 017** (`specs/017-greffiere-catalogue/data-model.md`) : la seule transition prévue à l'origine est `open→delivered` avec `trigger=objective_closed` et `objective_id` du **lien d'arbitrage** `(constat_id, objective_id)`.

2. **Chemin automatique** (pas un geste humain `registre fermer`) :
   - Toute commande Maicie qui ouvre le store appelle `reconcile_catalogue_from_store` (`main.rs`).
   - Celle-ci lit les objectifs `Clos` + les liens d'arbitrage des délégations (`app.rs:1402`).
   - Pour chaque couple lien+clôture : `append_delivered_for_attested_closure` (`catalogue.rs:918`).

3. **La preuve est INTERDITE par construction**, pas oubliée :
   - `append_delivered_for_attested_closure` pose `raison: None`, `reference: None` en dur (`catalogue.rs:943-944`).
   - `validate_transition_shape` **refuse** raison/référence/sévérité/nature sur `objective_closed` (`catalogue.rs:1637-1646` : « raison/référence/sévérité/nature interdites »).
   - À l'inverse, `registre fermer` / `remedied_attested` **exige** raison typée + `sha:`/`mesure:` (`close_constat_attested`, `validate_transition_shape` branche RemediedAttested).

4. **Effet sur la vue** : `to=delivered` est le **même état** pour `objective_closed` et `remedied_attested`. `registre list --fermes` affiche donc `raison=- ref=-` — la projection n'a rien à montrer.

5. **Aggravant VERIFIE** : `rectify_constat_attested` **refuse** de défaire un `objective_closed` (`catalogue.rs:1142-1145` : « fermeture juste intacte »). Le code traite donc cette transition comme une fermeture de **fond** inattaquable, alors qu'elle n'atteste que la clôture de mission.

6. **Porte de service** : c'est exactement le contournement de l'exigence de preuve de la session 028 — non par oubli CLI, mais par une voie de réconciliation qui écrit `delivered` sans passer par `fermer`.

## Ce que `objective_closed` atteste vraiment

| Atteste | N'atteste PAS |
|---|---|
| Un objectif lié par arbitrage est `Clos` au greffe | Que le défaut du constat est corrigé |
| Le couple `(constat_id, objective_id)` était déclaré | Qu'un sha/mesure prouve le remède |
| La mission de suivi est soldée | Que le dû technique a disparu |

Une raison `objectif_clos` existe déjà pour le geste **manuel** `registre fermer` (`RaisonFermeture::ObjectifClos`) — preuve typée possible. L'auto-réconciliation n'emprunte pas ce chemin.

## Recommandation (alignée sur ton penchant)

**Ne pas imposer sha/mesure à `objective_closed`.** Ce n'est pas le même acte.

**Le distinguer explicitement comme fermeture de suivi**, pas de fond :

1. **Projection** : ne plus afficher `objective_closed` comme `[FERMÉ]` au même titre que `remedied_attested`. Libellé du genre `[SUIVI]` / `[MISSION SOLDÉE]` + `objective_id`, sans feindre une preuve de correction.
2. **Pied / comptes** : un constat seulement `objective_closed` ne doit **pas** compter comme « dû réglé » dans les stats de fond (sinon le registre gonfle à l'envers : des dus disparaissent sans remède).
3. **Rectification** : lever l'interdiction de rectifier un `objective_closed` **ou** documenter qu'il ne retire pas le dû de fond — aujourd'hui le code ment en le disant « juste ».
4. **Option forte (si tu veux aller plus loin)** : `objective_closed` n'écrit plus `to=delivered` ; il pose une transition/annotation de suivi et laisse le constat **ouvert** jusqu'à `registre fermer` avec preuve. C'est un lot.

## Faut-il un lot ?

**Oui, si tu veux que le registre cesse de confondre les deux.** Pas un correctif one-liner : projection + (probablement) politique de `delivered` + oracles TEMOIN (un objectif clos ne doit plus faire disparaître un dû non remédié de la vue « ouverts », ou doit le marquer suivi).

**Pas de lot dans ce tour** — tu as demandé le diagnostic d'abord. Dis-moi laquelle des options 1–4 tu tranches.

## Les 35 IDs uniques (extraits du fichier)

Voir `/tmp/bt/cursor6-objective-closed-ids.txt` (colonnes `constat_id`, `objective_id`, `observed_at`). Doublons de transition sur le même constat (4) : `revue/cablage-natif-g11`, `spawn-timeout-agent-fantome`, `outcome-unknown-sur-livraison-reussie`, `migration-de-branche-sur-base-production`.
