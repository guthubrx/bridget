"""Mutations jetables r9 : copie /tmp/b149t-r9/mut uniquement, jamais l'arbre de travail.
Cible Cargo SEPAREE (bridget149-target-mut) : l'incident r8 (binaire debug ecrase) ne doit pas se reproduire.
Chaque mutation remplace un motif unique (assert count == 1) dans la copie, relance les tests cibles, puis restaure."""
import sys, subprocess, shutil, os, pathlib
SRC = pathlib.Path('/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage')
MUT = pathlib.Path('/tmp/b149t-r9/mut')
V = SRC / 'specs/149-sous-agents-lineage/validation'
ND = 'crates/bridget-daemon/src/daemon/native_delegation.rs'
MP = 'crates/bridget-daemon/src/managed_process.rs'
CS = 'crates/bridget-transport/src/codex_socket.rs'
LIB = ['cargo', 'test', '-p', 'bridget-daemon', '--lib', '--offline', '--locked', '--', 'native149_e2', 'native149_f3', 'native149_o4']
ALIAS = ['cargo', 'test', '-p', 'bridget-transport', '--test', 'native_alias149_test', '--offline', '--locked']
M = {
 'R0-temoin-daemon': ([], LIB),
 'E1-file-native-non-fermee': ([(ND, '(task.state == "queued" && queued_owner_lost(st, &task)?)', '(false && task.state == "queued" && queued_owner_lost(st, &task)?)')], LIB),
 'E2-filiation-sans-controle-racine': ([(ND, '        || parent.root_owner() != task.root_owner()\n', '')], LIB),
 'E3-filiation-sans-controle-origine': ([(ND, '        || instance != task.origin_owner_instance\n', '')], LIB),
 'E4-filiation-sans-controle-lien-delegation': ([(ND, '        || link.delegation_id.as_deref() != Some(parent.task_id.as_str())\n', '')], LIB),
 'F1-annulee-non-fermee-au-redemarrage': ([(ND, 'if matches!(task.state.as_str(), "cancelled" | "cancelling") {', 'if false && matches!(task.state.as_str(), "cancelled" | "cancelling") {')], LIB),
 'F2-cancel-tree-sans-fermeture': ([(ND, 'if let Err(error) = close_lost_execution(&st, &entry, "native_cancelled") {', 'if let Err(error) = Ok::<(), String>(()) {')], LIB),
 'F3-filtre-annulee-retire': ([(ND, 'child.state == "cancelled"\n                                    || (child.state == "failed"', 'false\n                                    || (child.state == "failed"')], LIB),
 'O1-natif-traite-comme-ordinaire': ([(MP, 'if native_identities.contains_key(&name) {\n                native_groups.push((name, marker));', 'if false && native_identities.contains_key(&name) {\n                native_groups.push((name, marker));')], LIB),
 'O2-grace-native-nulle': ([(MP, 'let native_deadline = Instant::now() + native_grace;', 'let native_deadline = Instant::now();')], LIB),
 'O3-identite-instance-ignoree': ([(MP, '&& (marker.instance_id != expected.instance_id', '&& (false && marker.instance_id != expected.instance_id')], LIB),
 'O4-identite-commande-ignoree': ([(MP, '|| marker.command_id != expected.command_id', '|| false')], LIB),
 'O5-identite-generation-ignoree': ([(MP, '|| marker.generation != expected.generation)', '|| false)')], LIB),
 'O6-envoi-natif-sequentiel': ([(MP, 'Ok(birth) if birth == marker.birth => signal_group(marker.pgid, libc::SIGTERM)?,', 'Ok(birth) if birth == marker.birth => { signal_group(marker.pgid, libc::SIGTERM)?; let _ = wait_group_gone(marker.pgid, native_grace, poll_interval); }')], LIB),
 'O7-escalade-sigkill-native': ([(MP, '                return Err(ManagedProcessError::InvalidStatus(format!(\n                    "arrêt coopératif natif incomplet', '                signal_group(marker.pgid, libc::SIGKILL)?;\n                return Err(ManagedProcessError::InvalidStatus(format!(\n                    "arrêt coopératif natif incomplet')], LIB),
 'O8-ordinaire-sans-escalade': ([(MP, 'if !wait_group_gone(marker.pgid, cooperative_grace, poll_interval)? {\n                match signal_group(marker.pgid, libc::SIGKILL) {', 'if false && !wait_group_gone(marker.pgid, cooperative_grace, poll_interval)? {\n                match signal_group(marker.pgid, libc::SIGKILL) {')], LIB),
 'I1-identite-sans-controle-cwd': ([(ND, '            || entry.cwd != Path::new(&task.cwd)\n', '')], LIB),
 'I2-identite-sans-controle-definition': ([(ND, '            || entry.resolved_definition.as_ref() != Some(&task.definition)\n', '')], LIB),
 'I3-identite-sans-controle-type': ([(ND, '            || &entry.agent_type != agent_type\n', '')], LIB),
 'A0-temoin-alias': ([], ALIAS),
 'A1-alias-sans-controle-de-mode': ([(CS, '        || metadata.mode() & 0o077 != 0\n', '        || false\n')], ALIAS),
 'A2-alias-occupe-accepte': ([(CS, '        Ok(_) => Err(io::Error::new(\n            io::ErrorKind::AlreadyExists,\n            "chemin de socket Codex déjà occupé",\n        )),', '        Ok(_) => Ok(()),')], ALIAS),
 'A3-alias-parent-lien-suivi': ([(CS, 'let metadata = std::fs::symlink_metadata(parent)?;', 'let metadata = std::fs::metadata(parent)?;')], ALIAS),
}
names = sys.argv[1:] or list(M)
env = dict(os.environ, TMPDIR='/tmp/b149t-r9', CARGO_TARGET_DIR='/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target-mut', CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0', CARGO_NET_OFFLINE='true')
os.umask(0o077)
touched = set()
for name in names:
    muts, cmd = M[name]
    # copie fraiche : production et tests depuis l'arbre de travail
    for f in (SRC / 'crates').rglob('*.rs'):
        rel = f.relative_to(SRC)
        if (MUT / rel).exists() and (MUT / rel).read_bytes() != f.read_bytes():
            shutil.copy(f, MUT / rel)
    for f, old, new in muts:
        t = (MUT / f).read_text()
        assert t.count(old) == 1, (name, old, t.count(old))
        (MUT / f).write_text(t.replace(old, new))
    log = V / f'native-r9-mutation-{name}.log'
    with open(log, 'w') as out:
        r = subprocess.run(cmd + ([] if '--' in cmd or cmd is ALIAS else []), cwd=MUT, env=env, stdout=out, stderr=subprocess.STDOUT)
    txt = log.read_text()
    res = [l for l in txt.splitlines() if l.startswith('test ') and ('... ' in l)]
    ko = [l for l in res if 'FAILED' in l]
    print(name, 'rc=', r.returncode, 'tests=', len(res), 'echecs=', len(ko), flush=True)
    for l in ko: print('    ECHEC', l.split('::')[-1], flush=True)
