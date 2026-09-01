# Inventaire de provenance - runtime production

| Élément | Valeur attestée lors du build |
|---|---|
| Plateforme | Linux amd64 |
| Base | `debian@sha256:d7e12182ce18b85b93007c1dedf31f2d29e01ccf3182cc4017c709b6259bc132` |
| Image locale de vérification | `sha256:6f53b434a16dfa1464212634aa4f6eca144454d222342c4094cdb8d4ca8849f5` |
| Git | 2.47.3 |
| Cargo | 1.85.0 |
| Node | 20.19.2 |
| pnpm | 10.15.0 |
| Python | 3.13.5 |

La commande de vérification est `docker run` avec rootfs en lecture seule,
utilisateur `1002:1002`, tmpfs borné et le programme
`/usr/local/bin/bridget-runtime-smoke`. Aucun fichier de credential n'est
présent dans le contexte de build ou dans l'image.
