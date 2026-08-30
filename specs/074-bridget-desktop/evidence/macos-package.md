# Paquet macOS - SPEC-074

Date : 2026-08-30.

## Build

Le paquet a été produit sur le Mac Apple Silicon avec Rustup stable et Cargo :

```text
/Users/moi/.cargo/bin/cargo test --manifest-path <copie-de-validation>/bridget-desktop/src-tauri/Cargo.toml
/Users/moi/.cargo/bin/cargo tauri build --bundles app
/Users/moi/.cargo/bin/cargo tauri bundle --bundles app --no-sign
```

Les 30 tests macOS ont réussi avant le bundle. Le bundle `.app` mesure environ 12 Mio.

## Artefact installé

`/Users/moi/Applications/Bridget Desktop 0.1.0.app`

- Identifiant : `app.cartae.bridget-desktop`.
- Version : `0.1.0`.
- Signature : ad hoc locale, validée par `codesign --verify --deep --strict`.
- SHA-256 du binaire, après correctif d'acceptation : `9e62f537f3d630f7203abe5c2c9ff185859cca960b976cb7e02f8b25620a5925`.

Le paquet a été reconstruit après l'activation explicite de `app.withGlobalTauri`, puis signé ad hoc et contrôlé avant copie dans le répertoire Applications.

## Limite de distribution

Le paquet ne porte pas de certificat Developer ID ni notarisation Apple. `spctl --assess` le refuse donc comme distribution externe, ce qui est attendu pour cette première construction locale. Pour l'ouvrir sur ce Mac, utiliser le Finder avec clic droit puis `Ouvrir` lors de la première exécution. Une distribution à d'autres machines demandera une identité Apple et une étape de notarisation, hors périmètre de cette SPEC.
