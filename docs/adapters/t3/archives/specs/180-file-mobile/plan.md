# Archives techniques de l’adaptateur T3

Document d’origine : [specs/180-file-mobile/plan.md](../../../../../../specs/180-file-mobile/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Référence lue : /home/moi/11.Repositories/t3code-local/apps/web/src/components/chat/QueuedRunsControl.tsx. Le composant attache un volet avec compteur/chevron au composeur, garde le texte visible, puis édite, annule ou promeut une entrée précise. Le chemin Mac demandé /Users/moi/11.Repositories/t3code est absent.

<a id="extrait-2"></a>

## Extrait 2

Réemploi Bridget : file native send_deliveries, rétention, remise au repos, annulation/promotion idempotentes et identité mobile existantes. Différence nécessaire : retenir aussi les messages du téléphone avant leur transfert au connecteur T3, afin que les actions restent natives et natives Bridget. Le connecteur annonce le tour réellement observé. La modification sous verrou et transaction exige une empreinte du texte courant ; le canon initial de création reste inchangé pour les rejeux.
