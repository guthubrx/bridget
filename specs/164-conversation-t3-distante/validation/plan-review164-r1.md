# Clôture de contre-revue de plan164 — R1

**Verdict original :** APPROVE_WITH_CHANGES.
**Provider :** Claude ; lecture seule, aucun test/build exécuté.
**Résultat durable privé :** /Users/moi/.cache/bridget-delivery164-20261013/plan-review164-r1-final.json
**Portée :** clôture des objections documentaires avant GO code. Aucun succès d’implémentation n’en découle.

ROOT a vérifié les objections dans les sources T3 et les exemples de journal public existant. DOC164 a appliqué les décisions dans spec/plan/contrat/audit/stratégie/Tasks. Une nouvelle contre-revue de plan n’est pas requise par ROOT si cette relecture ferme toutes les objections. La contre-revue de code reste T014.

| Objection | Preuve et décision ROOT | Correction documentaire | État |
|---|---|---|---|
| O1 — ChatMarkdown non inerte | cwd obligatoire, résolution de liens/code-inline, actions filesystem et images actives vérifiées | Prop opt-in inerte explicitement autorisée, tests Markdown affectés obligatoires ; aucun chargement média ou accès local implicite | Fermée dans le plan, code à prouver T008/T012 |
| O2 — Plus d’une lacune perdue | Réducteur conserve seulement gap unique ; état sticky et conteneurs vérifiés | gaps éphémères bornés64, déduplication sticky, purge hors fenêtre et coupe conservative si pression, aucun wire/store neuf | Fermée dans le plan, code à prouver T005 |
| O3 — Map masque un conflit de rejeu | Réducteur append sans déduplication ; containers Map gardent silencieusement premier | mergeJournalEvents pur commun avant Map, témoin conflictuel borné et détail visible ; premier conservé | Fermée dans le plan, code à prouver T004 |
| O4 — Formes source/message_id | ROOT confirme neuf kinds, identité de livraison, Claude appel seul, ACP tool_call_id, refusal Bridget. Reçu runtime montre started sans item_id et completed explicite | Tableau de formes figé ; aucune corrélation started/completed présumée ; pas de parole agent pour refusal | Fermée dans le plan, code à prouver T003/T006 |
| O5 — Headcut local silencieux | slice4096 après merge des containers confirmé | Drapeau de coupe/extrait levé aussi par merge local | Fermée dans le plan, code à prouver T004/T011 |
| O6 — Recalcul et DOM doublé | Risque de reparsing des blocs clos et de montage brut anticipé ; pas de mesure de débit annoncée | Mémoïsation des entrées, partage structurel des blocs clos, React.memo, brut monté à l’ouverture ; contrôles observables sans benchmark neuf | Fermée dans le plan, code à prouver T007/T009/T013 |
| O7 — Markdown mobile privé | ROOT confirme useLiveThreadLinkLabels et cartes actives dans le composant complet | Reprendre primitives Markdown natives/règles/styles en contexte inerte ; aucune exportation brute ni dérogation texte seul | Fermée dans le plan, code à prouver T010 |
| O8 — Page historique remplacée | setHistory remplace chaque page confirmé | Projeter chaque page seule, début extrait sauf début attesté ; aucun assembleur inventé | Fermée dans le plan, code à prouver T011 |
| O9 — Compte avant Tasks/README | 0/20 était provisoire ; aucune mention README trouvée dans le plan figé lu | Tasks réellement générées20/20 identifiées, zéro cochage ; allégation README non étayée, aucune modification inventée | Fermée documentairement |

## Nuances conservées

La formulation du reviewer « commande unique completed » n’est pas une description exhaustive : l’observation ROOT prouve aussi started sans item_id. Les fixtures positives164 prennent ces deux formes réelles. Les anciennes fixtures payload chaîne restent des cas hérités/fallback, pas une preuve de forme native.

Le saut de séquence seul ne prouve pas une lacune lorsque le journal est filtré. La pression sur gaps64 ne doit pas autoriser une fusion à travers une frontière perdue. L’état métier ne provient pas de caught_up. Les exemples réels sont lus seulement ; aucune source Rust 164 n’est modifiée.

La vue native mobile reste demandée et implémentée. Une recette web ne prouvera pas la coque native mobile ; toute limite de recette doit être consignée sans réduire la fonctionnalité ni inventer un PASS.
