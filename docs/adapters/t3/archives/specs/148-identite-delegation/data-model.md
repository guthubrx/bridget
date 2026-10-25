# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/data-model.md](../../../../../../specs/148-identite-delegation/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La preuve de session T3 est éphémère et détenue par son registre MCP existant.
Le rattachement Bridget porte agent UUID, instance UUID et credential privé.
Une délégation durable appartient à un parent et conserve : clé et enveloppe,
tâche/enfant Bridget, message/reçu Bridget, phase atteinte et résultat terminal.
Le magasin SQLite natif impose l'unicité de la clé par propriétaire. Il conserve
la définition complète du fournisseur sélectionné et son répertoire canonique.
Les checkpoints distinguent capture du résultat, remise au parent et nettoyage.
Les grants natifs sont conservés par instance avec racine et posture maximale.
Leur révocation conserve un refus durable par AgentId stable pour fermer aussi
l'héritage implicite après changement d'instance. Un nouveau grant humain le lève.
La tâche conserve le propriétaire initial et le propriétaire courant. Une reprise
du même agent après changement d'instance exige une nouvelle preuve native gérée
ou un grant humain, après départ de l'ancienne présence et de sa connexion.
La seule connaissance de son UUID ne suffit pas. L'admission de la remise fixe
l'échéance de réponse ; la reprise ne prolonge jamais ce délai.
