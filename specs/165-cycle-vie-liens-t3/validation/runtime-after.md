# Validation runtime après activation — SPEC-165

Date : 2026-10-14  
Source : `3b4f101e0732cc2c857e644469707fe8b5528618`  
Binaire signé SHA-256 : `45bc4836c874273012af9c3fffa52d5f676a9147809737548d59120c256dc1a4`

## Activation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/runtime-after.md#extrait-1).

## Mesures

| Mesure | Avant | Après stabilisation |
|---|---:|---:|
| Liens exposés | 1 130 | 32 |
| Fichiers ouverts | 4 539 | 167 |
| Sockets Unix | 3 390 | 128 |
| Fichiers réguliers | 1 146 | 37 |
| Limite souple | 16 384 | 4 096 |

Le second contrôle, vingt secondes plus tard, garde 32 liens et 167 fichiers
ouverts. Le fil courant `5b199e2d-3647-416c-a622-54d7a3059f79` reste exposé.

## Santé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/runtime-after.md#extrait-2).

## Retour arrière

La sauvegarde est conservée dans :

`/Users/moi/.cache/bridget-install165.UOCBU3`

Le binaire précédent signé est :

`/Users/moi/.cache/bridget-install165.UOCBU3/bridget.previous`

SHA-256 : `6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063`.

Le plist précédent est :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/runtime-after.md#extrait-3).

SHA-256 : `c9cd21e16d996f6c6f86f8e859c6e071f31ca7daa3d1976083c30f3e87227f02`.

## Verdict

La recette est validée sous la limite normale de 4 096. La hausse locale à
16 384 n'est plus nécessaire.
