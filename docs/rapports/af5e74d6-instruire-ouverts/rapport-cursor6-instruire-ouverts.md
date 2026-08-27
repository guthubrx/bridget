# Rapport — constats ouverts déjà réglés (sans fermeture)

## Mandat Maicie
- objective_id=`af5e74d6-293a-48b6-bd45-321d2957eab8`
- delegation_id=`d15643c8-b80b-4f0b-82d8-8cac2959fc5a`
- message_id=`d96ef48d-45c7-4ea0-a211-8cbcd6b8db4a`
- Reddition : rapport uniquement ; catalogue non touché ; reply Bridget non attendue.

- Agent : cursor6
- Base : `82c9757844a145829aaad77ff3432b4a01408ca8`
- Worktree lecture : `/Users/moi/.cache/bridget/cursor6-registre-instruire-ouverts` (détaché)
- Catalogue : **lu seulement** via `maicie registre list` + fichier config ; **aucune écriture**
- Pied Maicie au moment de la mesure : **174 ouverts / 73 blockers** (51 fermés listés)
- IDs : extraits du dump `/tmp/bt/cursor6-open-ids.txt` (pas recopiés à la main)

Légende de confiance :
- **VERIFIE** = preuve rejouée ici (sha ancêtre de HEAD, code lu, ou comportement runtime)
- **SUPPOSE** = texte du constat ou classement voisin, preuve non retrouvée ici → reste OUVERT

---

## A. Déjà réglés — candidats à fermeture (VERIFIE)

### A1. Défauts techniques dont le correctif est sur main

1. `review_amender:constat/jury-purge-migration-v4-refuse-de-demarrer`  
   - **Par quoi** : merge purge `d5ac20685ea2325cb8836ab83c50cef6800d391e` (`merge(purge): Ne pas orpheliner en silence`), ancêtre de HEAD.  
   - **Preuve** : `idempotency.rs` migration v4 fait `WHERE NOT EXISTS` + `DELETE` des fantômes + `warn!` ; oracle `migration_v4_nettoie_les_enfants_sans_parent_sans_bloquer_le_daemon`. Le démarrage ne meurt plus sur un enfant sans parent.

2. `review_amender:constat/config-profils-incompatible-avec-le-binaire`  
   - **Par quoi** : config de production elle-même.  
   - **Preuve runtime** : `/Users/moi/.config/maicie/config.json` — 60 profils, **0** sans `agent_type` (mesuré 2026-08-27).

3. `constat/filet-anti-co-autorat-perce-par-une-virgule`  
   - **Par quoi** : hook `commit-msg` (template `~/.git-templates` + hook local du dépôt).  
   - **Preuve** : motif générique `Co-Authored-By:` sans virgule Cursor ; mesure directe : trailer `Co-Authored-By: Cursor` + `Generated with Claude` **supprimés**, rc=0.  
   - **Borne** : `core.hooksPath` global est **vide** aujourd’hui ; la protection tient via hook **local** du dépôt et `init.templateDir=~/.git-templates` pour les nouveaux clones. Pas une preuve que les 13 clones cartae l’ont encore.

4. `review_amender:constat/carte-de-reprise-annulee-couple-de-production-non-visite`  
   - **Par quoi** : `04d76241616f6c50b9696ee729361f866d856838` (`merge(fix/annulee-hors-clos-sans-suite)`), ancêtre de HEAD.  
   - **Preuve** : merge sur main ; le STOP portait sur ce lot, désormais intégré.

5. `review_amender:constat/approbation-routine-hash-tautologique-FERME`  
   - **Par quoi** : garde `routine_approval_preflight` + `sealed_template_hash` sur main (introduite avec routines v15 ; présente dans l’arbre de `a26fda2286c84fc290bd86c45186b44bede63214`).  
   - **Preuve code** : `plugins/maicie/src/main.rs` compare hash recalculé vs stocké, refuse `gabarit altéré` ; oracle de mutant présent.  
   - **Note** : le Blocker source `gate_failed:constat/approbation-routine-hash-tautologique` est **déjà fermé** ; cette entrée est la consigne de fermeture encore ouverte (doublon de nature RESULTAT).

6. `review_amender:constat/023-levee-des-deux-bloquants-mesuree`  
   - **Par quoi** : `fd12643a9574…` déjà utilisé pour fermer les deux bloquants 023 ; merge `fd12643` sur main.  
   - **Preuve** : les deux constats source sont FERMÉS avec `ref=sha:fd12643a9574` ; cette entrée RESULTAT reste ouverte alors que la propriété est livrée.

### A2. Résultats / livraisons déjà sur main (nature RESULTAT encore ouvertes)

7. `resultat/031-lecture-totale-favorable-charge-historique-fermee-par-mutant`  
   - **Par quoi** : `7024df31de5b23bfeca27eb5588a5465a872a8b8` (`merge(031): Lecture totale…`), ancêtre de HEAD.

8. `resultat/anonymat-fixtures-favorable-et-trois-rouges-non-universels`  
   - **Par quoi** : `6eb77985114831450b8c7061aa281429b6a27151` (`merge(anonymat):…`), ancêtre de HEAD ; tête lot `866a93dbf4ae…` aussi ancêtre.

9. `resultat/la-gui-est-sur-main-arbre-merge-egale-arbre-mesure`  
   - **Par quoi** : `2330dfde7f9de8f663f84397b554f91d834096a2` sur main ; assets `crates/bridget-daemon/assets/ui/{app.js,index.html,theme.css}` présents à la base.

10. `resultat/perimetre-interface-clos-562234b`  
    - **Par quoi** : `562234b0f6bb5a5a16288ddc5bd628ca989057dc` (`merge(ui): En-tetes de cache…`), ancêtre de HEAD — déjà cité aussi pour fermer `constat/tout-etait-vert-et-la-page-ne-s-affichait-pas`.

11. `resultat/objectif-de-nuit-atteint-la-page-vit-et-la-reponse-arrive`  
    - **Par quoi** : parents cités `ac5ea45e652b…` et `47e238ded7f6…` ancêtres de HEAD (lots page/relais).  
    - **Borne** : la mesure Chromium du texte n’a **pas** été rejouée ici ; seule l’ancestralité Git est VERIFIEE.

12. `resultat/memoire-des-agents-claude-livree-et-prouvee-en-service`  
    - **Par quoi** : code `claude_provider_session` + injection `--resume` sur main (`385c82c`, `80a480e`, `23ac12c`), ancêtres de HEAD.  
    - **Borne** : preuve **code + commits** ; la mesure « en service » bout-en-bout agent Claude n’a pas été rejouée dans cette session.

13. `mesure/le-gate-test-support-est-desormais-jouable-sous-linux`  
    - **Par quoi** : même lot que la fermeture déjà faite de `constat/le-gate-test-support-est-injouable-sous-linux-et-aveugle-la-moitie-de-la-flotte` (`sha:af366bc` / `af366bc4ff7647b8985e4fd4f0ffe2e614013428`).  
    - Entrée RESULTAT jumelle encore ouverte.

---

## B. Semblent réglés mais ne le sont PAS (ou pas entièrement) — VERIFIE

1. `constat/trois-binaires-maicie-cohabitent-et-celui-du-path-est-le-plus-vieux`  
   - **Symptôme bloquant d’origine** (« PATH refuse schema 16, max 14 ») : **ne se reproduit plus**.  
     Mesure : `which maicie` → `/Users/moi/.local/bin/maicie` (27/08 03:07) ; `maicie status` OK ; `PRAGMA user_version=19`.  
   - **Ce qui reste vrai** : plusieurs binaires cohabitent encore (`maicie`, `maicie.avant-v19`, `maicie.bak-…`, `maicie.pre-3fe40b9` max 8, et `target/release/maicie` plus récent/plus gros). PATH ≠ build dépôt du jour.  
   - **Verdict** : le texte voisin laisse croire « tout est réglé » si on ne regarde que l’écriture au greffe ; la **cohabitation** et le risque de mauvais binaire restent. Ne pas fermer sur le seul symptôme schema.

2. `constat/binaire-maicie-du-path-ne-peut-plus-ecrire-au-greffe`  
   - Même famille. Symptôme « refuse d’écrire » : **réparé** pour le PATH actuel.  
   - Reste : rien n’empêche de réinstaller un vieux binaire sous ce PATH (preuves : binaires `.avant-v19` / `.pre-3fe40b9` toujours là).

3. `constat/validate-agent-name-protege-le-cli-pas-la-socket`  
   - **Toujours ouvert pour de vrai.**  
   - Preuve : `validate_agent_name` n’apparaît que dans `cli.rs` (appels CLI) ; pas sur le chemin Register socket/daemon à la base `82c9757`.

4. `review_amender:constat/federation-le-daemon-maitre-valide-un-chemin-distant`  
   et `review_amender:constat/federation-le-cycle-de-vie-reste-chez-le-maitre-AMENDE`  
   - **Aucun merge federation cycle-de-vie distant trouvé** comme correctif livré ; restent des bloquants structurels. Ne pas fermer.

---

## C. Fermetures du référent — critiques (preuve faible / attribution douteuse)

### C1. Affichage `raison=- ref=-` (35/51 fermés)

`registre list --fermes` montre `raison=- ref=-` pour **35** entrées. Ce sont les transitions `objective_closed` **sans** champs `raison`/`reference` typés — seulement `objective_id`.  
Ce n’est pas forcément faux (lien d’objectif), mais **ce n’est pas une preuve typée sha/mesure** au sens du lot de cette nuit. Si la norme actuelle exige `raison`+`reference`, ces 35 sont sous-documentées.

### C2. Fermetures récentes typées mais attribution SHA contestable

| ID fermé | ref déclarée | Problème VERIFIE |
|---|---|---|
| `regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide` | `sha:475ef10` | **475ef10** = `merge(echeance): Tamponner l echeance…` — **sans rapport** avec la règle du zéro/univers non vide. Fermer une **règle de méthode** comme `corrige_en_production` avec un merge d’échéance est une preuve **fausse**. |
| `gate_failed:constat/approbation-routine-hash-tautologique` | `sha:a26fda2` | SHA résout, et l’arbre **contient** la garde ; mais le sujet du merge est « Adopter les mandats orphelins », pas routines/hash. Attribution **imprécise** (préférer `d621ec7` ou le merge routines). |
| `resultat/un-agent-claude-revient-seul-et-se-souvient` | `sha:c3782e7b…` | SHA = merge **échéance** ; le correctif mémoire Claude est ailleurs (`385c82c`…). L’ancestralité sauve la propriété, pas l’étiquette. |
| Plusieurs fermetures du lot nuit | sha courts (`a26fda2`, `475ef10`, `cfff497`, `af366bc`) | Uniques aujourd’hui (disambiguate=1) mais **non complets** ; et toutes ont `objective_id` vide. |

### C3. Ce que je n’ai pas pu confirmer comme « les deux » signalées par l’agent précédent

L’agent précédent a cité deux fermetures à preuve faible. Les plus claires sous cette exigence sont :
1. `regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide` (SHA hors-sujet) — **la plus grave** ;
2. le paquet des 35 `raison=- ref=-` (preuve non typée au format actuel).

---

## D. Non instruits en profondeur (restent OUVERTS — ni VERIFIE ni fermeture proposée)

~160 autres ouverts non listés en A/B. Parmi eux, beaucoup de `regle/`, `doctrine/`, `decision/` sont des **règles permanentes** (pas des défauts à corriger) : les fermer comme « corrigés » serait une erreur de nature, sauf décision explicite de bascule RESULTAT/archivage.

Classement cursor2 (fichier `/tmp/bt/classement-v2.json`) : 30 RESULTAT / 52 REGLE / 103 CONSTAT — **presque tous encore ouverts** dans le greffe ; seules ~14 fermetures typées de la nuit ont bougé le compteur 188→174.

Je n’ai **pas** affirmé « déjà réglé » sur un constat dont le texte dit corrigé **sans** retrouver sha/code/runtime.

---

## E. Méthode / limites

- Pas de redémarrage daemon, pas de port 17888.
- Catalogue non modifié (autre agent sur rectification de transition).
- Checkout principal non écrit ; worktree dédié utilisé pour lire le code à la base.
- TMPDIR=/tmp/bt ; artefacts :  
  - `/tmp/bt/cursor6-ouverts-full.txt`  
  - `/tmp/bt/cursor6-fermes-full.txt`  
  - `/tmp/bt/cursor6-open-ids.txt`  
  - `/tmp/bt/rapport-cursor6-instruire-ouverts.md` (ce rapport)

## F. Proposition au référent (actes à sa charge)

Fermetures typées recommandées **seulement** pour la section A (13 IDs), chacune avec `raison` + `reference=sha:<40 hex>` du merge/preuve ci-dessus.  
Ne pas fermer B.  
Réexaminer C2 surtout `regle/un-zero-…`.
