# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/contracts/thread-api.md](../../../../../../../specs/102-fils-inter-agents/contracts/thread-api.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Amendement d'implémentation (T005)** : la capacité est annoncée par un fait
  de connexion `WrapperToDaemon::ThreadNoticeCapability { versions: [u16] }`,
  envoyé par le wrapper juste après `Registered`, comme `DiskSpace`/`JournalReady`,
  au lieu de champs ajoutés à `Register`/`Registered`. Motif : ajouter un champ
  aux deux variantes aurait modifié ~105 constructions littérales dans 30 fichiers
  pour le même effet ; le fait post-enregistrement conserve « sans nouveau
  handshake », la sélection liée à la connexion et la renégociation à chaque
  reconnexion. Au plus 8 valeurs lues ; seule la version 1 est sélectionnée ; un
  client auxiliaire n'est jamais éligible.
- Sélection liée à la connexion wrapper ; déconnexion et remplacement d'instance
  effacent ; reprise renégocie. Le daemon n'écho pas la sélection : le wrapper ne
  reçoit que des alertes de la version qu'il a annoncée ; `show` expose la version
  acceptée par membre.
- Ancien wrapper n'annonce rien et ne reçoit aucune alerte de fil
  (`capability_unavailable`) ; ancien daemon ignore la variante. DM historiques
  inchangés. Décodages testés (`spec102_v27_capacite_d_alerte_annoncee_apres_enregistrement`).
- Ce n'est ni ClientCapability ni ObservationCapabilities ; l'annonce est faite
  par le helper commun connect_and_register_at, donc par Codex/Claude gérés et
  interactifs et par le pont T3.
