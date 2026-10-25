# Reçu de livraison — SPEC-165

Date : 2026-10-14  
Statut : Implemented  
Tâches : 18/18  
Pipeline : Livré

## Résultat

Le pont ferme maintenant les liens des sous-agents terminés. Il conserve les
racines, les forks, les fils V1, le travail actif et les obligations durables.
Il draine les derniers événements avant la fermeture. Les reprises durables
sont bornées à 32 liens simultanés.

La hausse locale de limite n'est plus utilisée. Le plist ne contient plus
`SoftResourceLimits.NumberOfFiles=16384`. Le service annonce une limite souple
de 4 096.

## Git

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-1).

## Contrôles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-2).

Les 91 échecs globaux ne sont pas déclarés GREEN. Ils restent hors des trois
fichiers Rust affectés. Les contrôles ciblés du correctif sont verts.

## Activation réelle

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-4).

## Retour arrière

Sauvegarde complète :

`/Users/moi/.cache/bridget-install165.UOCBU3`

Binaire précédent signé :

`/Users/moi/.cache/bridget-install165.UOCBU3/bridget.previous`

Plist précédent :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-5).

## Preuves

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/livraison165.md#extrait-6).

## Verdict

Le défaut est corrigé par la fermeture des ressources. Il n'est plus masqué
par une hausse de limite.
