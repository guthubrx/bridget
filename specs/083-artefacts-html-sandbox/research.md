# Recherche - SPEC-083 Artefacts HTML sandboxés et navigateur latéral

**Date**: 2026-08-31

## Questions et décisions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/083-artefacts-html-sandbox/research.md#extrait-1).

## Enseignements utiles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/083-artefacts-html-sandbox/research.md#extrait-2).

## Risques à vérifier pendant l'implémentation

| Risque | Mesure prévue |
|---|---|
| `data_directory` non disponible en WKWebView sur macOS cible | test d'intégration de `data_store_identifier`, puis repli documenté ou contrainte de version macOS avant livraison |
| Insertion HTML qui casse l'enveloppe par `</script>` | transmettre les octets encodés et décoder dans un bootstrap de confiance, jamais interpoler du HTML brut dans une balise script |
| `postMessage` utilisé comme IPC implicite | schéma de message fermé, taille max, origine/instance validées, aucune commande native dérivée de texte libre |
| Redirection vers protocole dangereux | navigateur accepte HTTPS, route locale Bridget et cibles explicitement autorisées seulement ; intercept avant ouverture |
| État interactif très volumineux | plafond contractuel et création de version seulement après validation Bridget |
| Profil Browser confondu avec cache d'artefacts | répertoires et opérations de purge distincts ; l'effacement Browser ne touche jamais le magasin canonique |

## Baseline de sécurité

- Content Security Policy restrictive, aucune dépendance distante ni CDN.
- Capabilities Tauri réduites par label de WebView et testées négativement.
- Validation côté daemon des sources externes, y compris destinations et
  redirections, avant téléchargement ou restauration.
- Les actions Browser suivantes restent hors scope : sélection DOM assistée,
  clic ou saisie par agent, formulaires, téléchargement et publication.
