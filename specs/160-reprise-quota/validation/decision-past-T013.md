# Proposition conservatrice Past/T013 — décision ROOT requise
1. Admission157 inchangée : record et quota gardent leur filtrage actuel ; aucun quota expiré ne bloque une admission.
2. Pour160, garder la preuve d'épuisement d'une fenêtre, distincte de sa validité actuelle.
3. Borne métier : observation strictement après émission de mission ou fin de dernière attente.
4. Une fenêtre encore future à la décision suit Wait/TooFar/RoundsDone comme prévu.
5. Une fenêtre expirée ne rend Past que si son observation est postérieure ou égale au début du hold ouvert.
6. Ce hold dure au plus5s ; une ancienne observation périmée reste NoFact, même si son reset est conservé en mémoire.
7. Exemple : hold commence100 ; fait observé102, reset103 ; décision104 => usage_limit, limit_reset_at103.
8. Exemple : observation90, reset99 ; hold100, décision104 => fenêtre ignorée, puis erreur d'origine à105 sans nouveau fait.
9. L'API T013 proposée devient attested_exhausted_since(type,since,now,past_since:Option<i64>).
10. since reste la borne métier de3 ; past_since vaut hold.since seulement pendant sa résolution, sinonNone.
11. Filtre : preuve attestée + observation>since + (reset>now OU past_since présent et observation>=past_since).
12. Retour inchangé : Option<(reset maximal, observation la plus récente)> sur les seules fenêtres retenues par11.
13. Le paramètreOption est une borne fournie, pas une mémoire ni un nouvel état durable ; aucune nouvelle abstraction.
14. qualify_quota rend Past{reset_at} pour reset<=now ; ce champ porte limit_reset_at dans usage_limit (FR005).
15. Les observations non retenues par record157 ne sont pas inventées : aucun message d'erreur textuel ne devient preuve de quota.
16. Cette proposition couvre le fait accepté avant son reset puis périmé à la décision ; elle ne prétend pas restaurer une fenêtre déjà rejetée à l'enregistrement.
17. Aligner seulement la précision T013 « fenêtre périmée ignorée » sur « ancienne observation périmée ignorée » et cette signature après arbitrageROOT.
18. Oracles existants T013/T014/hold réemployés ; aucune tâche, famille de tests ou scénario ajouté ; aucun code/WT/Cargo.
