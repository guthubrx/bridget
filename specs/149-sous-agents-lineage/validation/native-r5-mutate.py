import sys, subprocess, shutil, os, pathlib
SRC = pathlib.Path('/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage')
MUT = pathlib.Path('/tmp/b149t-r5/mut')
V = SRC / 'specs/149-sous-agents-lineage/validation'
ND = 'crates/bridget-daemon/src/daemon/native_delegation.rs'
DM = 'crates/bridget-daemon/src/daemon.rs'
WR = 'crates/bridget-daemon/src/wrapper.rs'
IT = 'crates/bridget-daemon/tests/native_provider_death149_test.rs'
M = {
 'M0-temoin-sans-mutation': [],
 'M1-prepare_restart-sans-echec': [(ND, 'if matches!(task.state.as_str(), "starting" | "mission_pending" | "working") {', 'if false && matches!(task.state.as_str(), "starting" | "mission_pending" | "working") {')],
 'M2-reconnexion-rejoue-la-remise-native': [(DM, 'if !matches!(native_mission, Ok(false)) {', 'if false && !matches!(native_mission, Ok(false)) {')],
 'M3-reprise-execution-generique': [(DM, 'Ok(Some(task)) if *parent_execution_id == format!("execution-{}", task.mission) => {', 'Ok(Some(task)) if false && *parent_execution_id == format!("execution-{}", task.mission) => {')],
 'M4-flotte-sans-exclusion-natifs': [(DM, 'if native_children.contains(&candidate.lease.name) {', 'if false && native_children.contains(&candidate.lease.name) {'), (DM, 'if native_children.contains(&name) {', 'if false && native_children.contains(&name) {')],
 'M5-natifs-restent-running': [(ND, 'if running.contains(&task.child) {', 'if false && running.contains(&task.child) {')],
 'M6-wrapper-relance-natif': [(WR, '&& std::env::var_os(NATIVE_MISSION_BOOTSTRAP_ENV).is_none();', '&& true;')],
 'M45-exclusion-et-running-retires': [(DM, 'if native_children.contains(&candidate.lease.name) {', 'if false && native_children.contains(&candidate.lease.name) {'), (DM, 'if native_children.contains(&name) {', 'if false && native_children.contains(&name) {'), (ND, 'if running.contains(&task.child) {', 'if false && running.contains(&task.child) {')],
}
names = sys.argv[1:] or list(M)
env = dict(os.environ, TMPDIR='/tmp/b149t-r5', CARGO_TARGET_DIR='/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target-mut', CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0')
os.umask(0o077)
for name in names:
    # copie fraîche des deux fichiers de production concernés (et des tests)
    for f in (ND, DM, WR, IT, 'crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs'):
        shutil.copy(SRC / f, MUT / f)
    for f, old, new in M[name]:
        t = (MUT / f).read_text()
        assert t.count(old) == 1, (name, old, t.count(old))
        (MUT / f).write_text(t.replace(old, new))
    log = V / f'native-r5-mutation-{name}.log'
    with open(log, 'w') as out:
        r = subprocess.run((['cargo','test','-p','bridget-daemon','--test','native_provider_death149_test','--offline'] if name.startswith('M6') else ['cargo','test','-p','bridget-daemon','--lib','--offline','--','native149_redemarrage','native149_apres_redemarrage','native149_reconnexion','native149_reprise_d_execution','native149_bail_actif']), cwd=MUT, env=env, stdout=out, stderr=subprocess.STDOUT)
    txt = log.read_text()
    res = [l for l in txt.splitlines() if l.startswith('test ') and ('... ' in l)]
    print(name, 'rc=', r.returncode, flush=True)
    for l in res: print('   ', l.split('native_delegation_permissions149_tests::')[-1], flush=True)
