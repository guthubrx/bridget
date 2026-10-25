# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/044-relai-sous-agent-sous-autorite-parent.md](../../../../../decisions/044-relai-sous-agent-sous-autorite-parent.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Positives : le sous-agent peut contacter un autre agent ; le parent reste
  responsable ; les droits sont bornés ; aucun registre enfant durable.
- Négatives : le sous-agent ne reçoit pas directement une réponse ; la fonction
  dépend d'une filiation T3 attestable ; deux répertoires de marqueurs doivent
  être nettoyés ensemble.
