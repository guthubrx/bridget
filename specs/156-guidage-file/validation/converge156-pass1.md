# Converge156 — passage1

**Verdict : manque confirmé, une tâche T032 proposée.** FR016 seulement. ROOT applique l’append et la correction après cette clôture ; aucun autre manque confirmé.

Snapshot31/31. Tasks avant/après : 51ca315a8a0a379cb997cb4f919e996be82b273617de8898f42772a3111bf0f2 ; octets identiques.

## Tâche proposée

- [ ] T032 [Convergence] Satisfaire FR-016 pour les intentions valides refusées avant la garde par le projet (SendIdempotent, Send, ControlExecution et CancelRequest) et après autorisation par la persistance. Réutiliser guidance_events et la trace existante ; garder canon/réservation/rejeu dans leur ordre, zéro admission/consommation de quota/émission sur refus. Étendre seulement les branches déjà prévues de guidance156_rate_refus_non_compte pour affirmer la trace refused correspondante et le rejeu inchangé, sans scénario nommé supplémentaire. ROOT vérifie le gate réemploi/ArticleXIX si un petit helper interne partagé est nécessaire. Aucun outil/service/type public ni changement d’autorité.

## Matrice24FR/13SC/4US

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/156-guidage-file/validation/converge156-pass1.md#extrait-1).

## Preuves et limites

FULL unique :2350PASS Cargo +3PASS enfants séparés/11FAIL/73ignored,RC101. 80unitaires156daemon+4transport et3E2E sont dans cette suite, pas ajoutés au total. Trois nouvellesfixtures résolues par2+1PASS ciblés ; huit modes hérités conservés. Comparaison baseline763/763 cumulative et7ignored ; aucun GREENglobal.

Log : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-full156-once.log SHA fdb7deaff78eb9fa82a7af2879f59c3a0dd5858300450e8e198d80bd891e0274.
Comparaison : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T029-full156-once-comparison.json SHA 218af3b449aab1af086d29ebf1b5168a12adb1b6a3a1832e96823026db03f27c.

Code/artefacts relus dans les plages JSON. La gardepure entière et les trois corpsE2E sont lus ; gros fichiers, montageE2Einitial et tous les hunks ne sont pas relus entiers. Les lectures complètesT031 sont des preuves attribuées distinctes. Aucun test/build/service/modèle/Gitmutation personnel. Aucun WT modifié.

## Six réponsesXX

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/156-guidage-file/validation/converge156-pass1.md#extrait-2).
