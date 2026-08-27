# Plan 054

1. Mesurer les effets de `migrate`, `plage list` et `registre list` sans daemon.
2. Ajouter un témoin runtime sur les trois entrées et un inventaire structurel
   de tous les appels directs `MaicieStore::open*` dans le CLI.
3. Centraliser attestation et ouverture dans un helper recevant la même
   `MaicieConfig`.
4. Rejouer les témoins, les mutants de couverture et les gates ciblés.
