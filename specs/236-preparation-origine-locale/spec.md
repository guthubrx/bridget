# Correction236 — Préparer une copie sur la machine source

Incident opus2d du6novembre2026 : demande de revue commit_view surmonordinateur
refusée origin_request_invalid, sans champ. FaçadeMCP9525 vérifiée235-r3/SHA846d.
Aucun reçu d’admission ni tâche pour la clé -03 ; ne pas rejouer pour diagnostiquer.

Le helper accepte déjà objects_only source=cible. Le validateur du relais
PrepareProject interdit pourtant toute cible égale au worker source. Corriger
ce désaccord pour commit_view seulement. Conserver les gardes des autres
workspaces, identités, permissions, périmètres, SHA et capacités.
Donner le champ exact de validation lorsque connu. Le texte historique ne doit
pas déduire une ancienne version de la façade depuis un code générique.
