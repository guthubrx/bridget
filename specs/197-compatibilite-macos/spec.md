# Session197 — Compatibilité macOS du candidat commun

Correction autorisée dans la livraison en production. Le build macOS du candidat196 échoue (E0308/E0277) dans remote_worker_read.rs : MetadataExt::mode rend u32, tandis que les constantes libc de mode sont u16 sur macOS et u32 sur Linux. Convertir les deux constantes vers u32, sans changer les masques ni les conditions de refus. Aucun nouveau diagnostic, droit, service ou appel.

Validation : compiler réellement macOS et Linux, puis contrôler le parcours de lecture existant. Conserver les binaires actifs jusqu’à maintenance admise. Le gel natif est posé par l’humain, génération7 ; aucune nouvelle mission de revue ne peut être admise pendant ce gel. Aucun autre moteur ou canal de délégation ne sera utilisé.
