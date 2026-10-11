# Référence T001153 — avant tout code153

Exécution par le principal sur les sources152 livrées :
`/Users/moi/Nextcloud/10.Scripts/64.bridget`,
commit source `afd0f891ca4b1c03ff5e616d0e497cb1f7fc61f7`. Le WT153 vient de
`ee03ebdcfbba85028f0ba3962dde439919b48d29` ; ses200 empreintes Rust/Cargo.lock
sont égales à la capture brute. Vérification documentaire de ces empreintes
effectuée avant le GO153. Aucune source153 modifiée pendant les commandes.

| Filtre | PASS | FAIL | Ignorés | RC | Secondes |
|---|---:|---:|---:|---:|---:|
| native148 | 27 | 0 | 1 | 0 | 79.09 |
| native149 | 78 | 0 | 0 | 0 | 40.83 |
| native150 | 51 | 0 | 0 | 0 | 28.51 |
| delegation149 | 10 | 0 | 0 | 0 | 88.31 |
| spec_103 | 0 | 0 | 0 | 0 | 16.52 |
| spec_087 | 11 | 0 | 0 | 0 | 15.11 |
| spec103 | 17 | 0 | 0 | 0 | 12.58 |

Total des six filtres qui sélectionnent réellement des tests :
**194 PASS /0 FAIL /1 ignoré, six RC0**. Sept commandes exécutées,280,95s.
`spec_103` sélectionnait zéro test ; ce passage n'est aucune preuve de test.
Le filtre corrigé `spec103` sélectionne17 tests, RC0. Ce remplacement de filtre
ne crée aucun scénario. Tous les premiers journaux restent conservés.

Commande par filtre :
`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support <filtre> -- --test-threads=4 --nocapture`.
Environnement privé, HOME conservé, racine courte
`/private/tmp/b.acuurj1j`, BRIDGET_HOME privé et PATH du gestionnaire Python.
Le JSON brut signale `source_hashes_stable=true` sur toutes les sources Rust
et Cargo.lock. Ce reçu rapporte les exécutions du principal ; l'owner
documentaire n'a lancé ni Cargo ni test.

Copies durables sous `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation` :

| Fichier | SHA256 |
|---|---|
| baseline153-results.json | b7b7aebf006a1b15e9b95b4803a8a8071573a05e5ab34ffe369a059c8bfa8549 |
| baseline153-native148.log | cb91025b0dc78572df894e42506b501d1d61ae68980741ee9a85d7de817588d3 |
| baseline153-native149.log | 12ab317b4002b2f7a9740e6ae0b26bfaa2dfae6c019bcbc27e322238ba3916a7 |
| baseline153-native150.log | 5d7ad3f5c579339923259b4cd7b3d05733507afb3c2bc1499a6ed91a0538b4ee |
| baseline153-delegation149.log | e74fcd43fc64592f286efbc7658ecb095bec52f3f7b6c6617d38d7e06a38ea18 |
| baseline153-spec_103.log | a37c85314c20c3d5eb563b09c2653ca51fac40958606c7717f913b21c523a64e |
| baseline153-spec_087.log | 800260944b405719998eb01a868b895f87705dd3aebd6a6cf94ed7183ee9a4c5 |
| baseline153-spec103.log | 1fb1313f3dff8681c42ba64d3cb6976275b64159869812eae85b2824058670c5 |

Limites : référence de compatibilité ciblée avant153. Aucun test153, recette de
modèle, workspace complet ni activation153 ne sont déclarés par cette référence.
