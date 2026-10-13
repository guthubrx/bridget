# Vérification et livraison SPEC-161

**Procédure préparée, NON EXÉCUTÉE ici.** ROOT l'exécute après tests, convergence, revue, audit et intégration du runner161. La source compilée demeure160 exacte ; cette procédure ne prouve ni build ni activation.

## Préconditions

Relever le SHA du runner161 intégré et son hash, puis utiliser le checkout Git détaché propre et canonique /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-source160 au commit e6812e3e7b2d051d630192857cc3f440180d51fa. Ne pas compiler depuis le chantier161. L'appel du runner /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/deploy-remote.sh exige --expect-commit e6812e3e7b2d051d630192857cc3f440180d51fa, --source sur ce checkout, --remote-prefix /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d et --remote-cargo /home/moi/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo. Les paramètres de liaison ci-dessous sont ceux de cartae-core, à recontrôler avant exécution. Un préfixe déjà occupé est refusé ; une reprise explicite utilise un suffixe neuf -r2, répercuté dans toutes les commandes et dans le reçu.

Le runner doit vérifier le manifest distant avant build et imposer offline/locked, target-dir explicite, jobs2 et nice10. Aucun Rust installé ni téléchargement tiers. Conserver les identités/naissances initiales des services protégés et vérifier les modes/possessions des parents sans chmod global.

## Commande complète du runner intégré — après les gates

```bash
bash /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/deploy-remote.sh \
  --label cartae161 --host 37.59.185.67 --user moi --port 2222 \
  --identity /Users/moi/.ssh/id_ed25519 \
  --known-hosts /Users/moi/.ssh/known_hosts \
  --source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-source160 \
  --remote-prefix /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d \
  --remote-cargo /home/moi/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo \
  --expect-commit e6812e3e7b2d051d630192857cc3f440180d51fa
```

Conserver commande, SHA du runner intégré, journal et RC. La source reste160 ; les documents et le runner161 ne font pas partie du checkout compilé. La commande n'est pas exécutée par cette documentation.

## Vérification du développement

```bash
bash -n /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/deploy-remote.sh
bash /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae/scripts/tests/federation_089_test.sh
```

Un succès du harness n'est pas une livraison. Pas de suite workspace Rust pour les seules modifications du script.

## Session SSH explicite pour les contrôles et la bascule

Exécuter les blocs suivants dans la même session Bash. Les paramètres sont explicites, sans configurationSSH implicite ni nouvelle clé acceptée. Aucun secret n'apparaît dans les sorties.

```bash
set -euo pipefail
bridget161_ssh=(ssh -F /dev/null -p 2222 -i /Users/moi/.ssh/id_ed25519
  -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes
  -o UserKnownHostsFile=/Users/moi/.ssh/known_hosts
  -o GlobalKnownHostsFile=/dev/null -o UpdateHostKeys=no
  -o ControlMaster=no -o ControlPath=none -o ControlPersist=no
  -o ForwardAgent=no -o ForwardX11=no -o RequestTTY=no
  -o ConnectTimeout=10 -o ConnectionAttempts=1 moi@37.59.185.67)
```

Avant toute commande du candidat, relever les processusBridget de l'utilisateur distant. RC1 de pgrep signifie absence, pas panne de la sonde. Une présence se compare au relevé initial ; ne jamais tuer un processus pour obtenir un état vide.

```bash
"${bridget161_ssh[@]}" 'if pgrep -u moi -x bridget; then printf "bridget_processes=present\n"; else bridget161_rc=$?; if [ "$bridget161_rc" -eq 1 ]; then printf "bridget_processes=absent\n"; else exit "$bridget161_rc"; fi; fi'
```

Préflight frais ROOT : /Users/moi/.cache/bridget-delivery161-20261013/prebuild-remote.json, RC0, bridget_processes et cargo_processes vides, lien096 intact, groupeprimaire moi seulement. Cette observation ne remplace pas le contrôle après recette.

Après build seulement, vérifier le candidat par son chemin absolu. `--version` rapporte seulement CARGO_PKG_VERSION ; `status` rapporte le Build ID du daemon, pas celui du client. Attester le Build ID client par les sorties Cargo/build-script rustc-env BRIDGET_BUILD_ID, la chaîne embarquée corroborée par le SHA du binaire et la provenance source160 exacte. Conserver les chemins et hashes des sorties build ; une chaîne isolée ne suffit pas. Les commandes ci-dessous vérifient séparément version produit, SHA/architecture/modes et communication avec le maître.

```bash
"${bridget161_ssh[@]}" 'file /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget'
"${bridget161_ssh[@]}" 'sha256sum /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget'
"${bridget161_ssh[@]}" 'stat -c "%U:%G %a %n" /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget'
"${bridget161_ssh[@]}" '/home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget --version'
"${bridget161_ssh[@]}" '/home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget status'
"${bridget161_ssh[@]}" '/home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget who'
"${bridget161_ssh[@]}" 'readlink /home/moi/.local/bin/bridget'
"${bridget161_ssh[@]}" 'sha256sum /home/moi/.local/lib/bridget-communication/096/bridget'
```

Lire les sortiesCargo pertinentes et les octets du binaire, puis archiver cette preuve **avant build.py clean**, qui peut retirer ces sorties. Le répertoireprivé de preuves existe déjà ; noclobber refuse l'écrasement d'une preuve antérieure. En reprise, choisir un nom neuf et le consigner.

```bash
set -o noclobber
"${bridget161_ssh[@]}" 'python3 -B -' > /Users/moi/.cache/bridget-delivery161-20261013/client-build-identity.json <<'PYREAD'
from pathlib import Path
import hashlib
import json

root = Path('/home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d')
expected = 'e6812e3e7b2d'
outputs = []
for path in sorted((root / 'source/target/release/build').glob('bridget-daemon-*/output')):
    data = path.read_bytes()
    lines = [line for line in data.decode('utf-8').splitlines()
             if 'rustc-env=BRIDGET_BUILD_ID=' in line]
    if lines:
        outputs.append({'path': str(path), 'sha256': hashlib.sha256(data).hexdigest(),
                        'identity_lines': lines})
binary = root / 'bin/bridget'
data = binary.read_bytes()
result = {'expected_build_id': expected, 'cargo_outputs': outputs,
          'binary': str(binary), 'binary_sha256': hashlib.sha256(data).hexdigest(),
          'embedded_expected_id_present': expected.encode('ascii') in data}
print(json.dumps(result, indent=2))
assert outputs, 'aucune sortie build identity trouvee'
assert all(line.split('rustc-env=BRIDGET_BUILD_ID=', 1)[1] == expected
           for item in outputs for line in item['identity_lines']), 'identite source inattendue'
assert result['embedded_expected_id_present'], 'identite attendue absente des octets'
PYREAD
shasum -a 256 /Users/moi/.cache/bridget-delivery161-20261013/client-build-identity.json
```

Cette lecture corrobore la provenance source160 et le manifest ; la chaîne seule n'identifie pas un binaire de confiance. L'absence d'avertissement de divergence sur stderr de status peut corroborer la cohérence client/maître ; conserver stdout et stderr. Elle ne remplace pas les preuves source/build/SHA, et --version seul ne prouve pas le BuildID.

Résultats attendus : candidat exécutableLinuxx86_64, mode700, connexion au maître monordinateur160, lien courant toujours096 avant bascule. `status --json` n'est pas une commande valide. Aucune relance du daemon ne doit suivre l'avertissement historique erroné du client096.

## Activation : exécuter le bloc contractuel exact

La commande suivante extrait l'unique bloc testé du contrat, sans le recopier ni créer un helper. Elle doit utiliser le contrat du runner intégré, dont le SHA est dans le reçu. Le lien temporaire doit être neuf. Comparer/valider également propriétaire et groupe du parent partagé avant la commande. Le contrôle de cible n'est pas un CAS face à un tiers non coopérant ; un seul livreur et arrêt sur divergence observée.

```bash
python3 - <<'PYBLOCK' | "${bridget161_ssh[@]}" 'python3 - /home/moi/.local/bin/bridget /home/moi/.local/lib/bridget-communication/096/bridget /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget /home/moi/.local/bin/.bridget-161-activate'
from pathlib import Path
s = Path('/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/contracts/client-delivery.md').read_text()
start = '# BEGIN SPEC161_ATOMIC_LINK'
end = '# END SPEC161_ATOMIC_LINK'
assert s.count(start) == s.count(end) == 1
print(s.split(start, 1)[1].split(end, 1)[0])
PYBLOCK
"${bridget161_ssh[@]}" 'readlink /home/moi/.local/bin/bridget'
"${bridget161_ssh[@]}" '/home/moi/.local/bin/bridget status'
"${bridget161_ssh[@]}" '/home/moi/.local/bin/bridget who'
```

ROOT traite explicitement tout RCnon nul ; un échec ne doit pas être ignoré au motif que le shell s'est arrêté. Le blocrollback suivant est une opération distincte, à exécuter seulement si nécessaire.

## Recette transport séparée, depuis le Mac

Cette recette utilise deux sondesPython et les sockets ; elle ne lance pas le binaire candidat. Elle atteste annuaire et échanges dans les deux sens du canal/maître. Elle inscrit deux agents synthétiques et envoie deux messages via les API normales, puis les déconnecte. Ces écritures de recette sont autorisées et leurs UUID sont conservés dans le journal. Aucun modèle ni mission. Aucune édition directe, migration ou suppression de base.

```bash
python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/tests/federation_095_roundtrip.py \
  --host 37.59.185.67 --user moi --port 2222 \
  --identity /Users/moi/.ssh/id_ed25519 \
  --known-hosts /Users/moi/.ssh/known_hosts \
  --local-socket /Users/moi/.cache/bridget-core/bridget.sock \
  --remote-socket /home/moi/.cache/bridget-core/bridget.sock \
  --journal /Users/moi/.cache/bridget-delivery161-20261013/roundtrip161-after.jsonl \
  --timeout 15
```

Le journal est créé avec exclusivité. Si ce nom existe, ne pas l'écraser : choisir un nom de reprise exact et le consigner. Capturer RC et événements registered, same_inventory, delivered dans chaque sens et disconnected. Un status réussi ne remplace pas ces événements.

Après la recette, répéter le contrôle des processus distants et comparer au préflight et à la sonde avantcandidat :

```bash
"${bridget161_ssh[@]}" 'if pgrep -u moi -x bridget; then printf "bridget_processes=present\n"; else bridget161_rc=$?; if [ "$bridget161_rc" -eq 1 ]; then printf "bridget_processes=absent\n"; else exit "$bridget161_rc"; fi; fi'
```

Le démarrage du client peut créer son répertoire temporaire privé par init_process. Consigner les chemins réellement observés dans l'inventaire ; ce comportement normal n'autorise aucun daemon/restart et ne demande pas une autorisation hypothétique supplémentaire. Protéger les ressources encore actives lors du nettoyage.

## Retour arrière conditionnel

Uniquement après activation161 et échec de recette : vérifier que le lien est encore161. La primitive ci-dessous refuse toute autre cible. Elle conserve le binaire096 et ne touche pas le tunnel.

```bash
python3 - <<'PYBLOCK' | "${bridget161_ssh[@]}" 'python3 - /home/moi/.local/bin/bridget /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget /home/moi/.local/lib/bridget-communication/096/bridget /home/moi/.local/bin/.bridget-161-rollback'
from pathlib import Path
s = Path('/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/contracts/client-delivery.md').read_text()
start = '# BEGIN SPEC161_ATOMIC_LINK'
end = '# END SPEC161_ATOMIC_LINK'
assert s.count(start) == s.count(end) == 1
print(s.split(start, 1)[1].split(end, 1)[0])
PYBLOCK
"${bridget161_ssh[@]}" 'readlink /home/moi/.local/bin/bridget'
"${bridget161_ssh[@]}" '/home/moi/.local/bin/bridget status'
```

Si un tiers a changé le lien, arrêter sans écraser sa cible. Même réussi, ce rollback signifie lot161 non livré. Aucun redémarrage de T3, maître, pont ou fédération.

## Préparer le nettoyage, puis l'exécuter après preuve de livraison

La simulation suivante utilise l'outil déjà présent dans la source160 copiée. Elle borne seulement les caches traités par cet outil ; elle ne promet pas d'effacer tout source/ ou le préfixe.

```bash
"${bridget161_ssh[@]}" 'env BRIDGET_BUILD_CACHE_GIB=0 BRIDGET_BUILD_CACHE_DAYS=0 python3 /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/source/scripts/build.py clean --dry-run'
```

Après archivageSHA des preuves et vérification de l'absence de build actif, exécuter la même commande sans --dry-run. Conserver096rollback,161actif et cacheCargo verrouillé utile. La source restante, les caches protégés, les préfixes échoués et les sauvegardes d'index ont chacun une décision documentée ; aucun nettoyageglobal. Retirer checkoutsource et WT/branche fusionnés avec les commandesGit prévues, sansforce, après retour au checkoutdurable et contrôle d'usage. Comparer les processus protégés et vérifier le push par ls-remote. Tailleslogiques seulement, aucun gainphysique déduit.

## Cache : préparation désormais attestée, compilation non attestée

Preuve /Users/moi/.cache/bridget-delivery161-20261013/cache-supply-final.json, SHA256 e536e3d3f76f93a86c1f36775b7edd1d108987a097177c185b4f0442e5aee9e0. Le rapport indique success=true,28archives ajoutées (4 009 872octets),7entréesindex traitées et37objets relus. Les refus prudents des passes précédentes restent dans history_reports ; aucun téléchargement tiers ni Cargo n'a été exécuté par cette préparation. compilation_guaranteed=false. Les sauvegardes d'index et leurs empreintes sont conservées. Ce résultat remplace le constat initial « copie en cours » sans prouver le buildLinux.
