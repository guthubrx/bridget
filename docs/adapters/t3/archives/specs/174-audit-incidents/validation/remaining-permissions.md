# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/remaining-permissions.md](../../../../../../../specs/174-audit-incidents/validation/remaining-permissions.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 `packages/contracts/src/baseSchemas.ts` accepts branded non-empty identifiers.
Imported thread identifiers are not restricted to UUIDs. With thread A=`example`
and thread B=`instance:example`, B's agent ID equals A's instance ID. This is an
equality across roles, not an equality of two agent IDs or two instance IDs.

<a id="extrait-2"></a>

## Extrait 2

Bridget `t3code_mcp::resolve_binding` supplies the pair derived from one thread.
`mcp_identity::load_credential` validates both fields in the credential file.
`daemon::register_auxiliary` first resolves the agent's route, then checks that
route's credential and instance, and finally the live pair. A value from another
role cannot pass these checks by itself. Revocations also use separate tables:
`native_delegation_revocations.agent_id` and
`native_delegation_grants.owner_instance`. Message IDs are stored in message
records, not looked up as agent authority. Reply IDs additionally carry `t3-`.
Existing tests `spec099_preuve_auxiliaire_valide_bornee_et_revoquee` and
`spec172_binding_refusal_preserves_guards_and_explains_failed_check` exercise
credential, instance and live-route rejection. The opaque identifier integration
test protects imported identifiers. No identifier migration is justified here.

<a id="extrait-3"></a>

## Extrait 3

T3 `apps/server/src/mcp/McpSessionRegistry.ts` binds a random credential to a scope
containing distinct thread and provider-instance fields. Its revocation API is
separate from session liveness. Bridget retains the same separation of identity
selection, credential proof and live authority. T3 has no remote child-answer
wire protocol or equivalent delivery-error mapping to copy.
