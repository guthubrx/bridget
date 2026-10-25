T029 : **763/763 PASS par preuve cumulative**, sans neuf relances. L’unique FULL prouve762 noms PASS ; le témoin baseline de contrôle corrigé passe dans le replay ciblé réel du principal. Zéro nom manquant, zéro mauvaise cible/package, zéro doublon de premier niveau ; sept ignorés baseline conservés séparément.

Preuve : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T029-full156-once-comparison.json
SHA256 : 218af3b449aab1af086d29ebf1b5168a12adb1b6a3a1832e96823026db03f27c

FULL brut conservé :2353 PASS /11 FAIL /73 ignorés, RC101,700,10s,216 sources stables. Cargo premier niveau :2350 PASS /11 FAIL /73 ignorés. Trois PASS enfants Rust imbriqués sont séparés : adaptateur_d_arret_child1 et atomic_temporary_names_are_unique_across_processes2. Aucun compteur supplémentaire de contratbuild. La baseline774 brute avait déjà2 PASS enfants transport ;772 occurrences de premier niveau et763 PASS distincts donnent9 recouvrements de filtres. La référence initiale763 reste exacte ; sa classification des11 répétitions est corrigée explicitement dans le JSON.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/156-guidage-file/validation/T029-full156-once-comparison.md#extrait-1).

Les trois nouveaux FAIL du FULL ont leurs replays exacts PASS : les deux fixtures de bibliothèque,2P0F/RC0/8,27s ; la fixture138,1P0F/RC0/7,67s. Ces trois PASS ne sont pas ajoutés au total FULL. Les fixtures respectent désormais la vraie identité/lien parent ou le refus de contrôle sans autorité et entre projets. Leurs résultats FULL initiaux restent FAIL dans la preuve.

Replays : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-two-old-fixtures-green.json et /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-spec138-refusal-green.json. Leurs SHA/logs/noms exacts sont vérifiés et conservés dans le JSON. Ces reçus ciblés ne contiennent pas de snapshots de hashes sources ; aucune stabilité rétroactive n’est inventée. ROOT a déclaré des corrections de fixtures uniquement.

Aucune seconde suite complète sur le hash après corrections n’a été exécutée. Aucun GREEN workspace global ni cochage décidé. Le témoin Cursor au nom conservé PASS dans le FULL ; aucun skip de régression. Aucun test exécuté par cet agent.
