# Archives techniques de l’adaptateur T3

Document d’origine : [specs/133-relais-sous-agents/tasks.md](../../../../../../specs/133-relais-sous-agents/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Objectif : un sous-agent T3 attesté utilise `bridget_who` et `bridget_send`; le message porte sa provenance et la réponse revient au parent.

<a id="extrait-2"></a>

## Extrait 2

- [x] T004 [P] [US1] Ajouter les tests RED de sélection d'un fournisseur imbriqué sous un parent T3 unique et de résolution MCP déléguée dans `crates/bridget-daemon/src/t3code_identity.rs` et `crates/bridget-daemon/src/mcp_identity.rs`; résultat observable : le cas nominal échoue avant code puis identifie parent, fournisseur et référence opaque [FR-13301] [FR-13302].
- [x] T005 [US1] Étendre l'inventaire et la publication privée dans `crates/bridget-daemon/src/t3code_identity.rs`, puis la résolution MCP dans `crates/bridget-daemon/src/mcp_identity.rs`; résultat observable : une preuve `delegated-pids/<pid>` valide rend le parent avec contexte enfant, sans identifiant natif complet [FR-13301] [FR-13302] [FR-13312].
- [x] T006 [US1] Autoriser `bridget_who` et `bridget_send` pour le contexte délégué et poser la provenance dans `crates/bridget-daemon/src/mcp.rs`; résultat observable : l'envoi garde le parent comme portée, un rejeu parental relit son issue et une réponse suivie vise le parent [FR-13303] [FR-13305] [FR-13307].
- [x] T007 [P] [US1] Afficher la provenance dans `crates/bridget-transport/src/acp.rs`, `crates/bridget-transport/src/codex_app_server.rs` et `crates/bridget-daemon/src/t3code.rs`; résultat observable : les trois rendus nomment le parent et « via sous-agent », sans session native [FR-13306] [FR-13312].

<a id="extrait-3"></a>

## Extrait 3

- [x] T010 [P] [US3] Ajouter les tests RED des refus et du nettoyage dans `crates/bridget-daemon/src/mcp_identity.rs` et `crates/bridget-daemon/src/t3code_identity.rs`; résultat observable : preuve enfant invalide plus principal valide donne `identity_not_found`, et une preuve tierce reste intacte [FR-13308] [FR-13309].
- [x] T011 [US3] Fermer la résolution et le cycle de propriété dans `crates/bridget-daemon/src/mcp_identity.rs` et `crates/bridget-daemon/src/t3code_identity.rs`; résultat observable : T010 passe, retrait borné au propriétaire, aucun repli principal après preuve enfant invalide [FR-13308] [FR-13309].

<a id="extrait-4"></a>

## Extrait 4

Test indépendant : suites identité, MCP, T3, protocole, rendus et workspace sur état privé.
