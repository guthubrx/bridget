use super::*;

struct Fixture {
    root: PathBuf,
    repo: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("bridget-c19-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        let this = Self { root, repo };
        this.git(&["init", "-q"]);
        fs::create_dir(this.repo.join("sub")).unwrap();
        fs::write(this.repo.join("sub/tracked"), b"initial\n\0binary\xff").unwrap();
        fs::write(this.repo.join("code.js"), vec![b'c'; 200_000]).unwrap();
        fs::write(this.repo.join("large.bin"), vec![b'x'; 200_000]).unwrap();
        fs::write(
            this.repo.join(".gitattributes"),
            "sub/tracked export-ignore\ncode.js export-subst\n",
        )
        .unwrap();
        std::os::unix::fs::symlink("sub/tracked", this.repo.join("linked")).unwrap();
        std::os::unix::fs::symlink("missing", this.repo.join("dangling")).unwrap();
        this.git(&["add", "."]);
        this.git(&["commit", "-qm", "fixture"]);
        this
    }
    fn git(&self, args: &[&str]) -> String {
        let mut command = Command::new("git");
        for (name, _) in
            std::env::vars_os().filter(|(n, _)| n.to_string_lossy().starts_with("GIT_"))
        {
            command.env_remove(name);
        }
        let out = command
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-C",
                self.repo.to_str().unwrap(),
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim_end().into()
    }
    fn snapshot(&self) -> Vec<Vec<u8>> {
        vec![
            fs::read(self.repo.join(".git/HEAD")).unwrap(),
            fs::read(self.repo.join(".git/index")).unwrap(),
            self.git(&["show-ref"]).into_bytes(),
            self.git(&["status", "--porcelain=v1", "--untracked-files=all"])
                .into_bytes(),
            self.git(&["worktree", "list", "--porcelain"]).into_bytes(),
        ]
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn spec221_c19_exact_commit_export_preserves_dirty_shared_and_linked_checkout() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    fs::write(f.repo.join("sub/tracked"), b"next").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "descendant"]);
    let linked = f.root.join("linked-checkout");
    f.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "descendant",
        linked.to_str().unwrap(),
        "HEAD",
    ]);
    fs::write(f.repo.join("sub/tracked"), b"dirty").unwrap();
    fs::write(f.repo.join("untracked"), b"preserved").unwrap();
    let before = f.snapshot();
    let manifest = plan(&f.repo.join("sub"), &base).unwrap();
    let destination = f.root.join("view");
    export(&f.repo, &manifest, &destination).unwrap();
    assert_eq!(
        fs::read(destination.join("sub/tracked")).unwrap(),
        b"initial\n\0binary\xff"
    );
    assert_eq!(
        fs::read(destination.join("code.js")).unwrap(),
        vec![b'c'; 200_000]
    );
    assert!(!destination.join("large.bin").exists());
    assert_eq!(
        fs::read_link(destination.join("linked")).unwrap(),
        Path::new("sub/tracked")
    );
    assert!(
        fs::symlink_metadata(destination.join("dangling"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!destination.join(".git").exists());
    let public = fs::read_to_string(destination.join(NOTICE)).unwrap();
    assert!(public.contains("large.bin") && public.contains(&base));
    assert_eq!(f.snapshot(), before);
    assert!(
        export(&f.repo, &manifest, &destination).is_err(),
        "aucun écrasement"
    );
    assert_eq!(f.snapshot(), before);
    fs::set_permissions(
        destination.join("sub/tracked"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::write(destination.join("sub/tracked"), b"tampered").unwrap();
    assert!(
        audit(&manifest, &destination)
            .unwrap_err()
            .contains("view_integrity_failed")
    );
}
#[test]
fn spec221_c19_literal_objects_ignore_replace_and_source_is_not_needed_to_audit() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let original = plan(&f.repo, &base).unwrap();
    fs::write(f.repo.join("sub/tracked"), b"replacement").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "replacement"]);
    let next = f.git(&["rev-parse", "HEAD"]);
    f.git(&["replace", &base, &next]);
    let manifest = plan(&f.repo, &base).unwrap();
    assert_eq!(
        serde_json::to_vec(&manifest).unwrap(),
        serde_json::to_vec(&original).unwrap()
    );
    let destination = f.root.join("view");
    export(&f.repo, &manifest, &destination).unwrap();
    assert_eq!(
        fs::read(destination.join("sub/tracked")).unwrap(),
        b"initial\n\0binary\xff"
    );
    fs::rename(&f.repo, f.root.join("source-kept")).unwrap();
    audit(&manifest, &destination).unwrap();
    let omitted = manifest.entries.iter().find(|e| !e.selected).unwrap();
    fs::write(destination.join(&omitted.path), vec![b'x'; 200_000]).unwrap();
    audit(&manifest, &destination).unwrap();
    fs::write(destination.join("foreign"), b"extra").unwrap();
    assert!(audit(&manifest, &destination).is_err());
}
#[test]
fn spec221_c19_link_escape_cycles_names_and_budgets_refuse_before_export() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo, &base).unwrap();
    assert!(
        plan(&f.repo, &"0".repeat(40))
            .unwrap_err()
            .contains("commit_unavailable")
    );
    assert!(plan(&f.repo, &f.git(&["rev-parse", "HEAD:sub/tracked"])).is_err());
    assert!(plan(&f.root.join("absent"), &base).is_err());
    for path in [
        ".git/config",
        "sub/../file",
        "/file",
        "a//b",
        NOTICE,
        "sub/\nfile",
    ] {
        let mut altered = manifest.clone();
        altered.entries[0].path = path.into();
        assert!(export(&f.repo, &altered, &f.root.join("invalid")).is_err());
        assert!(!f.root.join("invalid").exists());
    }
    let mut altered = manifest.clone();
    altered.entries[0].bytes = MAX_BYTES + 1;
    altered.entries[0].selected = true;
    assert!(export(&f.repo, &altered, &f.root.join("invalid")).is_err());
    let mut links = BTreeMap::new();
    links.insert("a".into(), "b".into());
    links.insert("b".into(), "a".into());
    assert!(
        resolve_link("a", "b", &links)
            .unwrap_err()
            .contains("link_cycle")
    );
    links.clear();
    links.insert("a".into(), "../../outside".into());
    assert!(
        resolve_link("b", "a/file", &links)
            .unwrap_err()
            .contains("link_outside_view")
    );
    assert!(resolve_link("a", "/tmp/foreign", &links).is_err());
    assert_eq!(
        resolve_link("sub/a", "../target", &BTreeMap::new()).unwrap(),
        "target"
    );
    // Vrai objet Git, lien sortant, refus sans destination partielle.
    std::os::unix::fs::symlink("../../outside", f.repo.join("outgoing")).unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "outgoing link"]);
    let invalid = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    assert!(
        export(&f.repo, &invalid, &f.root.join("invalid"))
            .unwrap_err()
            .contains("link_outside_view")
    );
    assert!(!f.root.join("invalid").exists());
}

#[test]
fn spec221_c19_directory_descriptors_refuse_aliases_and_do_not_follow_substitution() {
    let f = Fixture::new();
    let stage = f.root.join("stage");
    fs::create_dir(&stage).unwrap();
    let directory = Directory::open(&stage).unwrap();
    let sub = directory.create("sub").unwrap();
    let outside = f.root.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::rename(stage.join("sub"), stage.join("kept-sub")).unwrap();
    std::os::unix::fs::symlink(&outside, stage.join("sub")).unwrap();
    assert!(directory.child("sub").is_err());
    let mut file = sub.file("proof", true).unwrap();
    file.write_all(b"owned directory only").unwrap();
    assert_eq!(
        fs::read(stage.join("kept-sub/proof")).unwrap(),
        b"owned directory only"
    );
    assert!(!outside.join("proof").exists());
    assert!(directory.file("sub/proof", true).is_err());
    assert!(directory.child("..").is_err());
    assert!(
        directory.file("sub", false).is_err(),
        "lien terminal refusé"
    );
}
#[test]
fn spec221_c19_manifest_never_reads_an_unbounded_materialized_omission() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo, &base).unwrap();
    let root = f.root.join("view");
    export(&f.repo, &manifest, &root).unwrap();
    let mut altered = manifest.clone();
    let entry = altered.entries.iter_mut().find(|e| !e.selected).unwrap();
    entry.bytes = MAX_BYTES + 1;
    let file = fs::File::create(root.join(&entry.path)).unwrap();
    file.set_len(entry.bytes).unwrap();
    let error = audit(&altered, &root).unwrap_err();
    // La carte publique peut refuser d'abord ; aucun gros fichier n'est lu.
    assert!(error.contains("byte_limit") || error.contains("view_integrity_failed"));
}

#[test]
fn spec221_c19_subdirectory_source_never_exports_inside_repository_or_external_git_dir() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo.join("sub"), &base).unwrap();
    for destination in [f.repo.join("view"), f.repo.join(".git/view")] {
        assert!(
            export(&f.repo.join("sub"), &manifest, &destination)
                .unwrap_err()
                .contains("destination_inside_source")
        );
        assert!(!destination.exists());
    }
    let linked = f.root.join("linked-checkout");
    f.git(&[
        "worktree",
        "add",
        "-q",
        "--detach",
        linked.to_str().unwrap(),
        &base,
    ]);
    let sibling = f.repo.join("must-not-be-created");
    assert!(
        export(&linked.join("sub"), &manifest, &sibling)
            .unwrap_err()
            .contains("destination_inside_source")
    );
    assert!(!sibling.exists());
    let destination = f.repo.join(".git/external-view");
    assert!(
        export(&linked.join("sub"), &manifest, &destination)
            .unwrap_err()
            .contains("destination_inside_source")
    );
    assert!(!destination.exists());
}
#[test]
fn spec221_c19_ten_thousand_files_have_independent_tree_and_entry_budgets() {
    let f = Fixture::new();
    let many = f.repo.join("many");
    fs::create_dir(&many).unwrap();
    for n in 0..10_000 {
        fs::write(many.join(format!("file-{n:05}.txt")), b"fixture").unwrap();
    }
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "large fixture"]);
    let manifest = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    assert_eq!(manifest.entries.len(), 10_006);
    let started = Instant::now();
    let destination = f.root.join("view");
    export(&f.repo, &manifest, &destination).unwrap();
    eprintln!("C19 10006 entries: export+audit {:?}", started.elapsed());
    assert!(started.elapsed() < DEADLINE);
}
#[test]
fn spec221_c19_expired_object_read_has_owned_child_stopped() {
    let f = Fixture::new();
    let manifest = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    let mut objects = Objects::open(&f.repo).unwrap();
    objects.deadline = Instant::now();
    let started = Instant::now();
    let mut output = Vec::new();
    assert!(
        objects
            .object(&manifest.entries[0], &mut output)
            .unwrap_err()
            .contains("git_timeout")
    );
    assert!(started.elapsed() < Duration::from_millis(100));
    objects.stop().unwrap();
    assert!(objects.child.try_wait().unwrap().is_some());
}

#[test]
fn spec221_c19_materialize_is_exact_idempotent_and_never_overwrites() {
    let f = Fixture::new();
    fs::create_dir(f.repo.join("omitted")).unwrap();
    fs::write(f.repo.join("omitted/data.bin"), vec![b'z'; 200_000]).unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "omitted fixture"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo, &base).unwrap();
    let root = f.root.join("view");
    export(&f.repo, &manifest, &root).unwrap();
    let before = f.snapshot();
    let notice_before = fs::read(root.join(NOTICE)).unwrap();
    assert!(materialize(&f.repo, &manifest, &root, "omitted/data.bin").unwrap());
    assert_eq!(
        fs::read(root.join("omitted/data.bin")).unwrap(),
        vec![b'z'; 200_000]
    );
    assert!(!materialize(&f.repo, &manifest, &root, "omitted/data.bin").unwrap());
    assert!(
        materialize(&f.repo, &manifest, &root, "../outside")
            .unwrap_err()
            .contains("path_not_in_commit")
    );
    assert_eq!(fs::read(root.join(NOTICE)).unwrap(), notice_before);
    audit(&manifest, &root).unwrap();
    assert_eq!(f.snapshot(), before);
    fs::set_permissions(
        root.join("omitted/data.bin"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::write(
        root.join("omitted/data.bin"),
        b"preserve local modification",
    )
    .unwrap();
    assert!(materialize(&f.repo, &manifest, &root, "omitted/data.bin").is_err());
    assert_eq!(
        fs::read(root.join("omitted/data.bin")).unwrap(),
        b"preserve local modification"
    );
}
#[test]
fn spec221_c19_missing_omitted_blob_keeps_view_and_does_not_create_a_placeholder() {
    let f = Fixture::new();
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo, &base).unwrap();
    let root = f.root.join("view");
    export(&f.repo, &manifest, &root).unwrap();
    let omitted = manifest.entries.iter().find(|e| !e.selected).unwrap();
    let loose = f
        .repo
        .join(".git/objects")
        .join(&omitted.oid[..2])
        .join(&omitted.oid[2..]);
    fs::remove_file(loose).unwrap();
    let notice_before = fs::read(root.join(NOTICE)).unwrap();
    assert!(
        materialize(&f.repo, &manifest, &root, &omitted.path)
            .unwrap_err()
            .contains("git_object_unavailable")
    );
    assert!(!root.join(&omitted.path).exists());
    assert_eq!(fs::read(root.join(NOTICE)).unwrap(), notice_before);
    audit(&manifest, &root).unwrap();
}

#[test]
fn spec221_c19_required_cwd_exists_even_when_its_files_are_omitted() {
    let f = Fixture::new();
    fs::remove_file(f.repo.join("sub/tracked")).unwrap();
    fs::write(f.repo.join("sub/only.bin"), vec![b'z'; 200_000]).unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "omitted cwd"]);
    let base = f.git(&["rev-parse", "HEAD"]);
    let manifest = plan(&f.repo.join("sub"), &base).unwrap();
    assert_eq!(manifest.cwd, "sub");
    let root = f.root.join("view");
    export(&f.repo, &manifest, &root).unwrap();
    assert!(root.join("sub").is_dir());
    assert!(!root.join("sub/only.bin").exists());
    audit(&manifest, &root).unwrap();
    assert!(materialize(&f.repo, &manifest, &root, "sub/only.bin").unwrap());
    fs::create_dir(f.repo.join("future")).unwrap();
    assert!(
        plan(&f.repo.join("future"), &base)
            .unwrap_err()
            .contains("cwd_not_in_commit")
    );
}

#[test]
fn spec221_c19_link_aliases_are_checked_by_target_filesystem_without_following_them() {
    let f = Fixture::new();
    fs::create_dir(f.repo.join("dir")).unwrap();
    std::os::unix::fs::symlink("../B", f.repo.join("dir/a")).unwrap();
    std::os::unix::fs::symlink("DIR/a", f.repo.join("b")).unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "case alias"]);
    let manifest = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    let root = f.root.join("view");
    let case_alias = fs::symlink_metadata(f.repo.join("B")).is_ok();
    let result = export(&f.repo, &manifest, &root);
    if case_alias {
        assert!(result.unwrap_err().contains("link_cycle"));
        assert!(!root.join(NOTICE).exists(), "vue partielle non publiée");
    } else {
        result.unwrap();
        audit(&manifest, &root).unwrap();
    }
    let directory = Directory::open(&root).unwrap();
    assert!(
        directory
            .link_is_present("b", "/tmp/foreign", Instant::now() + DEADLINE)
            .unwrap_err()
            .contains("link_outside_view")
    );
    assert!(
        directory
            .link_is_present("b", ".", Instant::now())
            .unwrap_err()
            .contains("view_link_timeout")
    );
}
#[test]
fn spec221_c19_notice_names_unresolved_links_and_keeps_initial_selection_after_materialize() {
    let f = Fixture::new();
    let manifest = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    let root = f.root.join("view");
    export(&f.repo, &manifest, &root).unwrap();
    let notice_bytes = fs::read(root.join(NOTICE)).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&notice_bytes).unwrap();
    assert!(value["next_action"].as_str().unwrap().contains("bridget_read_commit_file"));
    assert!(value["next_action"].as_str().unwrap().contains("sans task_id"));
    let dangling = value["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "dangling")
        .unwrap();
    assert_eq!(dangling["exact_case_target_in_commit"], false);
    assert_eq!(dangling["exact_case_target_selected_at_export"], false);
    let linked = value["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "linked")
        .unwrap();
    assert_eq!(linked["exact_case_target_in_commit"], true);
    assert_eq!(linked["exact_case_target_selected_at_export"], true);
    materialize(&f.repo, &manifest, &root, "large.bin").unwrap();
    assert_eq!(fs::read(root.join(NOTICE)).unwrap(), notice_bytes);
    audit(&manifest, &root).unwrap();
}
#[test]
fn spec221_c19_implicit_directories_count_towards_entry_budget_before_writes() {
    let f = Fixture::new();
    let mut manifest = plan(&f.repo, &f.git(&["rev-parse", "HEAD"])).unwrap();
    let entry = manifest
        .entries
        .iter()
        .find(|e| e.mode == "100644")
        .unwrap()
        .clone();
    manifest.entries = (0..8000)
        .map(|n| {
            let mut entry = entry.clone();
            entry.path = format!("p{n}/a/b/c/d/e/f/g/file.txt");
            entry
        })
        .collect();
    let root = f.root.join("view");
    assert!(
        export(&f.repo, &manifest, &root)
            .unwrap_err()
            .contains("entry_limit")
    );
    assert!(!root.exists());
}

fn private_intent(f: &Fixture) -> (ViewProofs, ViewIntent, PathBuf) {
    let private = f.root.join("proofs");
    fs::create_dir(&private).unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
    let data = f.root.join("views");
    fs::create_dir(&data).unwrap();
    fs::set_permissions(&data,fs::Permissions::from_mode(0o700)).unwrap();
    let binding = ViewBinding {
        worker: bridget_transport::protocol::WorkerIdentity {
            machine_id: "fixture".into(),
            worker_id: "worker".into(),
            provisioning_revision: 1,
        },
        command: bridget_transport::protocol::WorkerCommandIdentity {
            command_id: uuid::Uuid::new_v4().to_string(),
            task_id: uuid::Uuid::new_v4().to_string(),
            instance_id: String::new(),
            generation: 0,
        },
        prepare_digest: "a".repeat(64),
    };
    let intent = ViewIntent::plan(binding, &f.repo, &data, &f.git(&["rev-parse", "HEAD"])).unwrap();
    (ViewProofs::open(&private).unwrap(), intent, private)
}

#[test]
fn spec221_c19_private_publication_reopens_without_source_and_keeps_exact_omissions() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    let reserved = intent.reserved_bytes().unwrap();
    proofs.write_intent(&intent).unwrap();
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    // Une extraction exacte sans publication n'autorise toujours pas la reprise.
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    proofs.publish(&intent).unwrap();
    proofs.publish(&intent).unwrap();
    proofs.write_intent(&intent).unwrap();
    assert!(
        fs::read_dir(&private)
            .unwrap()
            .map(|e| e.unwrap().metadata().unwrap().len())
            .sum::<u64>()
            <= reserved
    );
    let reopened = ViewProofs::open(&private).unwrap();
    let recovered = reopened
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    assert!(
        materialize(
            &intent.source,
            &recovered.manifest,
            &destination,
            "large.bin"
        )
        .unwrap()
    );
    assert!(
        !materialize(
            &intent.source,
            &recovered.manifest,
            &destination,
            "large.bin"
        )
        .unwrap()
    );
    // Les preuves n'ont pas besoin d'un .git ou d'objets accessibles pour l'audit.
    fs::rename(&f.repo, f.root.join("source-offline")).unwrap();
    reopened
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    assert!(!destination.join(".git").exists());
}

#[test]
fn spec221_c19_private_proof_requires_same_binding_roots_and_view_identity() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    proofs.publish(&intent).unwrap();
    let mut wrong = intent.binding.clone();
    wrong.prepare_digest = "b".repeat(64);
    assert!(
        proofs
            .recover(&wrong, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(
        proofs
            .recover(&intent.binding, &f.root, &intent.data_root)
            .is_err()
    );
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &f.root)
            .is_err()
    );
    let saved = f.root.join("original-view");
    fs::rename(&destination, &saved).unwrap();
    // Même contenu, autre inode : le reçu de publication ne l'adopte pas.
    export(&intent.source, &intent.manifest, &destination).unwrap();
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(proofs.publish(&intent).is_err());
    fs::remove_dir_all(&destination).unwrap();
    fs::rename(&saved, &destination).unwrap();
    proofs
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    fs::set_permissions(
        destination.join("sub/tracked"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::write(destination.join("sub/tracked"), b"divergent").unwrap();
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(
        private
            .join(format!("{}.intent.json", intent.binding.key().unwrap()))
            .is_file()
    );
    assert_eq!(
        fs::read(destination.join("sub/tracked")).unwrap(),
        b"divergent"
    );
}

#[test]
fn spec221_c19_partial_proof_or_view_is_conserved_without_replacement() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    let path = private.join(format!("{}.intent.json", intent.binding.key().unwrap()));
    fs::write(&path, b"{").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(proofs.write_intent(&intent).is_err());
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), b"{");
    // Simule une preuve absente : un dossier déjà présent interdit de la recréer.
    fs::remove_file(path).unwrap();
    fs::create_dir(intent.destination().unwrap()).unwrap();
    fs::write(intent.destination().unwrap().join("partial"), b"kept").unwrap();
    assert!(proofs.write_intent(&intent).is_err());
    assert_eq!(
        fs::read(intent.destination().unwrap().join("partial")).unwrap(),
        b"kept"
    );
}

#[test]
fn spec221_c19_private_proof_refuses_links_modes_unknown_fields_and_oversize() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    let path = private.join(format!("{}.intent.json", intent.binding.key().unwrap()));
    let outside = f.root.join("outside-proof");
    fs::write(&outside, intent.encoded().unwrap()).unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o600)).unwrap();
    std::os::unix::fs::symlink(&outside, &path).unwrap();
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(proofs.write_intent(&intent).is_err());
    fs::remove_file(&path).unwrap();
    fs::hard_link(&outside, &path).unwrap();
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    fs::remove_file(&path).unwrap();
    proofs.write_intent(&intent).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut value = serde_json::to_value(&intent).unwrap();
    value["unexpected"] = true.into();
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    let file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    file.set_len(MAX_INTENT as u64 + 1).unwrap();
    assert!(
        proofs
            .read_intent(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(ViewProofs::open(&private).is_err());
}

#[test]
fn spec221_c19_private_proof_directory_is_not_taken_from_the_public_notice() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    let nested = intent.data_root.join("proofs");
    fs::create_dir(&nested).unwrap();
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        ViewProofs::open(&nested)
            .unwrap()
            .write_intent(&intent)
            .is_err()
    );
    proofs.write_intent(&intent).unwrap();
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    let alias = f.root.join("proofs-alias");
    std::os::unix::fs::symlink(&private, &alias).unwrap();
    assert!(ViewProofs::open(&alias).is_err());
    // Une fausse publication dans la vue ne remplace jamais la preuve privée.
    fs::write(destination.join("published.json"), b"{}").unwrap();
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(proofs.publish(&intent).is_err());
    assert!(
        !private
            .join(format!("{}.published.json", intent.binding.key().unwrap()))
            .exists()
    );
}

fn view_environment() -> BTreeMap<String, std::ffi::OsString> {
    ["PATH", "HOME", "TMPDIR"]
        .iter()
        .filter_map(|name| std::env::var_os(name).map(|value| ((*name).into(), value)))
        .collect()
}
fn wait_view_job(job: &mut super::job::ViewJob) -> super::job::JobPoll {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let result = job.poll().unwrap();
        if result != super::job::JobPoll::Running {
            return result;
        }
        assert!(Instant::now() < deadline, "le helper ne termine pas");
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn start_view_job(
    proofs: &ViewProofs,
    intent: &ViewIntent,
    env: &BTreeMap<String, std::ffi::OsString>,
) -> super::job::ViewJob {
    match super::job::ViewJob::start(
        proofs,
        intent,
        &super::job::ViewJobRequest::Export,
        &std::env::current_exe().unwrap(),
        env,
        |_| Ok(()),
    ) {
        Ok(job) => job,
        Err((error, mut retained)) => {
            if let Some(job) = retained.as_mut() {
                assert!(job.stop(Duration::from_secs(2)).unwrap());
            }
            panic!("lancement refusé : {error:?}");
        }
    }
}

#[test]
fn spec221_c19_managed_export_owns_group_before_io_and_rejects_concurrent_start() {
    use super::job::{JobPoll, ViewJob};
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut env = view_environment();
    env.insert(
        "PRIVATE_SHOULD_NOT_BE_INHERITED".into(),
        "not-a-secret".into(),
    );
    let mut job = start_view_job(&proofs, &intent, &env);
    let name = format!("view-{}.json", intent.binding.key().unwrap());
    assert!(private.join(&name).is_file());
    assert!(
        ViewJob::start(
            &proofs,
            &intent,
            &super::job::ViewJobRequest::Export,
            &std::env::current_exe().unwrap(),
            &env,
            |_| Ok(())
        )
        .is_err()
    );
    assert_eq!(wait_view_job(&mut job), JobPoll::Stopped(Ok(())));
    assert!(!private.join(&name).exists());
    proofs
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    let count: u64 = fs::read_dir(&private)
        .unwrap()
        .map(|e| e.unwrap().metadata().unwrap().len())
        .sum();
    assert!(count <= intent.reserved_bytes().unwrap());
}

#[test]
fn spec221_c19_managed_export_failure_is_closed_and_conserves_intent() {
    use super::job::{ExportFailure, JobPoll};
    let f = Fixture::new();
    std::os::unix::fs::symlink("../PRIVATE_OUTSIDE", f.repo.join("escape")).unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "link-fixture"]);
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut job = start_view_job(&proofs, &intent, &view_environment());
    assert_eq!(
        wait_view_job(&mut job),
        JobPoll::Stopped(Err(ExportFailure::LinkOutsideView))
    );
    assert!(!intent.destination().unwrap().exists());
    proofs
        .read_intent(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    assert!(
        !private
            .join(format!("view-{}.json", intent.binding.key().unwrap()))
            .exists()
    );
    assert_eq!(
        serde_json::to_string(&ExportFailure::LinkOutsideView).unwrap(),
        "\"link_outside_view\""
    );
}

#[test]
fn spec221_c19_managed_stalled_export_keeps_marker_and_never_claims_publication() {
    use super::job::{ExportFailure, JobPoll, ViewJob, source_group_stopped};
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut env = view_environment();
    env.insert("B221_VIEW_TEST_STALL".into(), "1".into());
    let mut job = start_view_job(&proofs, &intent, &env);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !private.join("stall-ready").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!job.stop(Duration::from_millis(30)).unwrap());
    assert_eq!(job.poll().unwrap(), JobPoll::StopUnconfirmed);
    let reopened = ViewProofs::open(&private).unwrap();
    assert!(!source_group_stopped(&reopened, &intent).unwrap());
    assert!(
        private
            .join(format!("view-{}.json", intent.binding.key().unwrap()))
            .exists()
    );
    assert!(
        ViewJob::start(
            &reopened,
            &intent,
            &super::job::ViewJobRequest::Export,
            &std::env::current_exe().unwrap(),
            &env,
            |_| Ok(())
        )
        .is_err()
    );
    assert!(
        reopened
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    // La fixture coopère après le constat de rétention ; aucun SIGKILL.
    fs::write(private.join("allow-exit"), b"").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match job.poll().unwrap() {
            JobPoll::Stopped(result) => {
                assert_eq!(result, Err(ExportFailure::TimedOut));
                break;
            }
            _ => {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    assert!(source_group_stopped(&reopened, &intent).unwrap());
    assert!(
        reopened
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
    assert!(!intent.destination().unwrap().exists());
}

#[test]
fn spec221_c19_export_submode_requires_managed_context_before_reading_proof_paths() {
    // Le processus de test n'est pas un wrapper. Les chemins inexistants ne sont
    // jamais lus : le refus est celui du canal géré manquant, pas du disque.
    let result =
        super::job::run_commit_view_export(&["/PRIVATE_MISSING_PROOF".into(), "a".repeat(64)]);
    assert_eq!(result.unwrap_err(), refusal("managed_context_required"));
}

#[test]
#[ignore]
fn c19_export_child() {
    let Some(root) = std::env::var_os("B221_VIEW_PROOFS") else {
        return;
    };
    assert!(std::env::var_os("PRIVATE_SHOULD_NOT_BE_INHERITED").is_none());
    let root = PathBuf::from(root);
    if std::env::var_os("B221_VIEW_TEST_STALL").is_some() {
        let mut reporter = crate::managed_process::ManagedStatusReporter::from_environment()
            .unwrap()
            .unwrap();
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("stall-ready"))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("allow-exit").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        reporter.startup_succeeded();
        std::process::exit(0);
    }
    let key = std::env::var("B221_VIEW_KEY").unwrap();
    let request: super::job::ViewJobRequest =
        serde_json::from_str(&std::env::var("B221_VIEW_REQUEST").unwrap()).unwrap();
    let result = if std::env::var_os("B221_VIEW_PLAN").is_some() {
        super::job::run_commit_view_plan(&[root.to_str().unwrap().into(),key])
    } else if matches!(request, super::job::ViewJobRequest::Export) {
        super::job::run_commit_view_export(&[root.to_str().unwrap().into(), key])
    } else {
        super::job::run_commit_view_materialize(&[
            root.to_str().unwrap().into(),
            key,
            serde_json::to_string(&request).unwrap(),
        ])
    };
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}

#[test]
fn spec221_c19_workspace_report_uses_private_anchor_without_git_facts_or_live_source() {
    let f = Fixture::new();
    let (proofs, intent, _) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    export(
        &intent.source,
        &intent.manifest,
        &intent.destination().unwrap(),
    )
    .unwrap();
    proofs.publish(&intent).unwrap();
    let record = proofs.workspace(&intent).unwrap();
    assert!(matches!(record.strategy, WorkspaceStrategy::CommitView));
    assert!(record.branch.is_empty());
    assert!(
        record.head_commit.is_none()
            && record.branch_commit.is_none()
            && record.head_on_branch.is_none()
            && record.commit_count.is_none()
    );
    let report = super::super::report(&record).unwrap();
    assert_eq!(report.state, WorkspaceState::Kept);
    assert_eq!(report.dirty, Some(false));
    assert!(crate::remote_worker::same_workspace_anchor(
        &record,
        &serde_json::to_value(&report).unwrap()
    ));
    fs::rename(&f.repo, f.root.join("source-offline")).unwrap();
    assert_eq!(super::super::report(&record).unwrap().dirty, Some(false));
    fs::set_permissions(
        intent.destination().unwrap().join("sub/tracked"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::write(
        intent.destination().unwrap().join("sub/tracked"),
        b"changed",
    )
    .unwrap();
    assert!(super::super::report(&record).is_err());
    assert_eq!(
        fs::read(intent.destination().unwrap().join("sub/tracked")).unwrap(),
        b"changed"
    );
}

#[test]
fn spec221_c19_workspace_anchor_rejects_private_reference_substitution_and_keeps_legacy_bytes() {
    let f = Fixture::new();
    let (proofs, intent, _) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    export(
        &intent.source,
        &intent.manifest,
        &intent.destination().unwrap(),
    )
    .unwrap();
    proofs.publish(&intent).unwrap();
    let record = proofs.workspace(&intent).unwrap();
    let mut changed = record.clone();
    changed.commit_view.as_mut().unwrap().intent_digest = "c".repeat(64);
    assert!(!crate::remote_worker::same_workspace_anchor(
        &record,
        &serde_json::to_value(&changed).unwrap()
    ));
    assert!(super::super::report(&changed).is_err());
    changed = record.clone();
    changed
        .commit_view
        .as_mut()
        .unwrap()
        .binding
        .command
        .task_id = "another-task".into();
    assert!(!crate::remote_worker::same_workspace_anchor(
        &record,
        &serde_json::to_value(&changed).unwrap()
    ));
    assert!(super::super::report(&changed).is_err());
    changed = record.clone();
    changed.commit_view.as_mut().unwrap().proof_root = f.root.to_string_lossy().into();
    assert!(super::super::report(&changed).is_err());
    changed = record.clone();
    changed.commit_view = None;
    assert!(
        super::super::report(&changed)
            .unwrap_err()
            .starts_with("workspace_commit_view_audit_required")
    );
    // Le champ neuf reste absent des anciennes valeurs Worktree.
    changed.strategy = WorkspaceStrategy::Worktree;
    let old = serde_json::to_value(&changed).unwrap();
    assert!(old.get("commit_view").is_none());
    let mut unknown = serde_json::to_value(&record).unwrap();
    unknown["commit_view"]["extra"] = true.into();
    assert!(serde_json::from_value::<WorkspaceRecord>(unknown).is_err());
    assert_eq!(super::super::report(&record).unwrap().dirty, Some(false));
}

#[test]
fn spec221_c19_release_callback_failure_never_runs_export_and_retains_owned_bootstrap() {
    use super::job::{ExportFailure, JobPoll, ViewJob};
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut retained = match ViewJob::start(
        &proofs,
        &intent,
        &super::job::ViewJobRequest::Export,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |_| Err(ExportFailure::ProofUnavailable),
    ) {
        Err((cause, Some(job))) => {
            assert_eq!(cause, ExportFailure::ProofUnavailable);
            job
        }
        _ => panic!("échec d'autorisation ignoré"),
    };
    assert!(!intent.destination().unwrap().exists());
    assert!(retained.stop(Duration::from_secs(2)).unwrap());
    assert_eq!(
        retained.poll().unwrap(),
        JobPoll::Stopped(Err(ExportFailure::ProofUnavailable))
    );
    assert!(!intent.destination().unwrap().exists());
    drop(retained);
    // Le verrou local libéré ne vaut pas clôture de la preuve durable.
    assert!(matches!(
        ViewJob::start(
            &proofs,
            &intent,
            &super::job::ViewJobRequest::Export,
            &std::env::current_exe().unwrap(),
            &view_environment(),
            |_| Ok(())
        ),
        Err((ExportFailure::StopUnconfirmed, None))
    ));
    assert!(
        private
            .join(format!("{}.intent.json", intent.binding.key().unwrap()))
            .is_file()
    );
    assert!(
        proofs
            .recover(&intent.binding, &intent.source, &intent.data_root)
            .is_err()
    );
}

#[test]
fn spec221_c19_execution_paths_replace_only_the_admitted_mapping_root() {
    let f = Fixture::new();
    let (_, mut intent, _) = private_intent(&f);
    intent = ViewIntent::plan(
        intent.binding.clone(),
        &f.repo.join("sub"),
        &intent.data_root,
        &f.git(&["rev-parse", "HEAD"]),
    )
    .unwrap();
    let view = intent.destination().unwrap();
    assert_eq!(
        intent.execution_paths(&f.repo).unwrap(),
        (view.clone(), view.join("sub"))
    );
    assert_eq!(
        intent.execution_paths(&f.repo.join("sub")).unwrap(),
        (view.join("sub"), view.join("sub"))
    );
    for root in [
        f.root.clone(),
        f.root.join("other"),
        f.repo.join("sub/.."),
        PathBuf::from("repo"),
    ] {
        assert!(intent.execution_paths(&root).is_err(), "root {root:?}");
    }
    // La dérivation est une lecture privée et ne touche aucun dossier.
    assert!(!view.exists());
}

#[test]
fn spec221_c19_managed_omission_uses_exact_blob_and_owned_instance_before_io() {
    use super::job::{ExportFailure, JobPoll, ViewJob, ViewJobRequest};
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut export = start_view_job(&proofs, &intent, &view_environment());
    assert_eq!(wait_view_job(&mut export), JobPoll::Stopped(Ok(())));
    drop(export);
    let destination = intent.destination().unwrap();
    assert!(!destination.join("large.bin").exists());
    let mut command = intent.binding.command.clone();
    command.instance_id = "reader-instance".into();
    command.generation = 3;
    let request = ViewJobRequest::Materialize {
        command: command.clone(),
        sequence: 1,
        path: "large.bin".into(),
    };
    let mut job = match ViewJob::start(
        &proofs,
        &intent,
        &request,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |marker| {
            assert_eq!(marker.instance_id, command.instance_id);
            assert_eq!(marker.generation, command.generation);
            assert_ne!(marker.command_id, command.command_id);
            assert!(
                private
                    .join(format!("view-{}.json", intent.binding.key().unwrap()))
                    .exists()
            );
            assert!(
                !destination.join("large.bin").exists(),
                "no file IO before caller authorization"
            );
            Ok(())
        },
    ) {
        Ok(job) => job,
        Err((error, mut retained)) => {
            if let Some(job) = retained.as_mut() {
                assert!(job.stop(Duration::from_secs(2)).unwrap());
            }
            panic!("materialization launch {error:?}");
        }
    };
    assert_eq!(wait_view_job(&mut job), JobPoll::Stopped(Ok(())));
    drop(job);
    assert_eq!(
        fs::read(destination.join("large.bin")).unwrap(),
        vec![b'x'; 200000]
    );
    proofs
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    assert!(!destination.join(".git").exists());

    // Une nouvelle demande d'instance ne peut pas écraser un travail divergent.
    fs::set_permissions(
        destination.join("large.bin"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::write(destination.join("large.bin"), b"preserve this").unwrap();
    let mut check = match ViewJob::start(
        &proofs,
        &intent,
        &request,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |_| Ok(()),
    ) {
        Ok(job) => job,
        Err((error, mut held)) => {
            if let Some(job) = held.as_mut() {
                assert!(job.stop(Duration::from_secs(2)).unwrap());
            }
            panic!("owned check {error:?}");
        }
    };
    assert_eq!(
        wait_view_job(&mut check),
        JobPoll::Stopped(Err(ExportFailure::IntegrityFailed))
    );
    assert_eq!(
        fs::read(destination.join("large.bin")).unwrap(),
        b"preserve this"
    );
}

#[test]
fn spec221_c19_managed_omission_rejects_unbound_requests_and_denied_release() {
    use super::job::{ExportFailure, JobPoll, ViewJob, ViewJobRequest};
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut export = start_view_job(&proofs, &intent, &view_environment());
    assert_eq!(wait_view_job(&mut export), JobPoll::Stopped(Ok(())));
    drop(export);
    let mut command = intent.binding.command.clone();
    command.instance_id = "reader".into();
    command.generation = 1;
    let valid = ViewJobRequest::Materialize {
        command: command.clone(),
        sequence: 1,
        path: "large.bin".into(),
    };
    let mut requests = vec![
        ViewJobRequest::Materialize {
            command: command.clone(),
            sequence: 0,
            path: "large.bin".into(),
        },
        ViewJobRequest::Materialize {
            command: command.clone(),
            sequence: 1,
            path: "../elsewhere".into(),
        },
        ViewJobRequest::Materialize {
            command: command.clone(),
            sequence: 1,
            path: "not-in-commit".into(),
        },
        ViewJobRequest::Materialize {
            command: command.clone(),
            sequence: 1,
            path: "linked".into(),
        },
        ViewJobRequest::Materialize {
            command: intent.binding.command.clone(),
            sequence: 1,
            path: "large.bin".into(),
        },
    ];
    command.task_id = "other-task".into();
    requests.push(ViewJobRequest::Materialize {
        command: command.clone(),
        sequence: 1,
        path: "large.bin".into(),
    });
    command.task_id = intent.binding.command.task_id.clone();
    command.command_id = "other-command".into();
    requests.push(ViewJobRequest::Materialize {
        command,
        sequence: 1,
        path: "large.bin".into(),
    });
    for request in requests {
        assert!(
            ViewJob::start(
                &proofs,
                &intent,
                &request,
                &std::env::current_exe().unwrap(),
                &view_environment(),
                |_| panic!("unbound request")
            )
            .is_err()
        );
        assert!(
            !private
                .join(format!("view-{}.json", intent.binding.key().unwrap()))
                .exists()
        );
    }
    let mut held = match ViewJob::start(
        &proofs,
        &intent,
        &valid,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |_| Err(ExportFailure::ProofUnavailable),
    ) {
        Err((ExportFailure::ProofUnavailable, Some(job))) => job,
        _ => panic!("retain the abandoned owned bootstrap"),
    };
    assert!(matches!(
        wait_view_job(&mut held),
        JobPoll::Stopped(Err(ExportFailure::ProofUnavailable))
    ));
    assert!(!intent.destination().unwrap().join("large.bin").exists());
    assert!(
        private
            .join(format!("{}.intent.json", intent.binding.key().unwrap()))
            .is_file()
    );
}

#[test]
fn spec221_c19_lease_release_is_not_delayed_by_an_inherited_description() {
    let f = Fixture::new();
    let (proofs, intent, _private) = private_intent(&f);
    let key = intent.binding.key().unwrap();
    let held = proofs.lease(&key).unwrap();
    // dup conserve la même description ouverte que fork avant exec.
    // Aucun descendant n'acquiert l'autorité de garder ce créneau.
    let inherited = held.0.try_clone().unwrap();
    assert!(
        proofs.lease(&key).is_err(),
        "the current owner must keep exclusivity"
    );
    drop(held);
    assert!(
        proofs.lease(&key).is_ok(),
        "a stale inherited descriptor must not retain the released slot"
    );
    assert!(inherited.metadata().is_ok());
}

#[test]
fn spec221_c19_closed_reference_preserves_files_and_reports_without_source_but_cannot_write() {
    let f = Fixture::new();
    for i in 0..512 {
        fs::write(f.repo.join(format!("item-{i:04}.txt")), b"fixture").unwrap();
    }
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "larger manifest"]);
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    proofs.publish(&intent).unwrap();
    let record = proofs.workspace(&intent).unwrap();
    let card_path = destination.join(NOTICE);
    let card = fs::read(&card_path).unwrap();
    let metadata = fs::metadata(&card_path).unwrap();
    let regular = fs::read(destination.join("code.js")).unwrap();
    let full_bytes =
        fs::metadata(private.join(format!("{}.intent.json", intent.binding.key().unwrap())))
            .unwrap()
            .len();
    let source_offline = f.root.join("source-offline");
    fs::rename(&f.repo, &source_offline).unwrap();
    let reserved = proofs.compact_closed(&intent).unwrap();
    assert_eq!(reserved, intent.closed_reserved_bytes().unwrap());
    let encoded =
        fs::read(private.join(format!("{}.intent.json", intent.binding.key().unwrap()))).unwrap();
    assert!((encoded.len() as u64) < full_bytes / 10);
    eprintln!(
        "C19 closed metadata {full_bytes} -> {} bytes; public card unchanged {} bytes",
        encoded.len(),
        card.len()
    );
    assert_eq!(fs::read(&card_path).unwrap(), card);
    assert_eq!(fs::metadata(&card_path).unwrap().ino(), metadata.ino());
    assert_eq!(fs::metadata(&card_path).unwrap().mtime(), metadata.mtime());
    assert_eq!(fs::read(destination.join("code.js")).unwrap(), regular);
    assert!(!destination.join("large.bin").exists());
    let reopened = ViewProofs::open(&private).unwrap();
    assert_eq!(
        reopened
            .load(&intent.binding.key().unwrap())
            .unwrap()
            .proof_digest()
            .unwrap(),
        intent.proof_digest().unwrap()
    );
    let report = report_view(&record).unwrap();
    assert_eq!(report.dirty, Some(false));
    assert_eq!(report.state, WorkspaceState::Kept);
    assert!(reopened.write_intent(&intent).is_err());
    assert!(reopened.publish(&intent).is_err());
    let attempted = super::job::ViewJob::start(
        &reopened,
        &intent,
        &super::job::ViewJobRequest::Export,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |_| panic!("closed before release"),
    );
    assert!(matches!(
        attempted,
        Err((super::job::ExportFailure::ProofUnavailable, None))
    ));
    let before =
        fs::read(private.join(format!("{}.intent.json", intent.binding.key().unwrap()))).unwrap();
    reopened.compact_closed(&intent).unwrap();
    assert_eq!(
        fs::read(private.join(format!("{}.intent.json", intent.binding.key().unwrap()))).unwrap(),
        before
    );
}

#[test]
fn spec221_c19_closed_reference_authenticates_public_manifest_and_publication_identity() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    proofs.publish(&intent).unwrap();
    proofs.compact_closed(&intent).unwrap();
    let card_path = destination.join(NOTICE);
    let card = fs::read(&card_path).unwrap();
    let key = intent.binding.key().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&card).unwrap();
    value["manifest"]["entries"][0]["oid"] = "1".repeat(40).into();
    for bad in [
        b"{}".to_vec(),
        serde_json::to_vec(&value).unwrap(),
        vec![b'x'; MAX_INTENT + 1],
    ] {
        fs::write(&card_path, bad).unwrap();
        assert!(proofs.load(&key).is_err());
    }
    fs::write(&card_path, &card).unwrap();
    fs::remove_file(&card_path).unwrap();
    assert!(proofs.load(&key).is_err());
    let outside = f.root.join("outside-card");
    fs::write(&outside, &card).unwrap();
    std::os::unix::fs::symlink(&outside, &card_path).unwrap();
    assert!(proofs.load(&key).is_err());
    fs::remove_file(&card_path).unwrap();
    fs::write(&card_path, &card).unwrap();
    let moved = f.root.join("original-view");
    fs::rename(&destination, &moved).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(&card_path, &card).unwrap();
    assert!(proofs.load(&key).is_err());
    fs::remove_file(&card_path).unwrap();
    fs::remove_dir(&destination).unwrap();
    fs::rename(&moved, &destination).unwrap();
    proofs
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    // Un champ ajouté à la petite preuve n'est jamais ignoré.
    let path = private.join(format!("{key}.intent.json"));
    let bytes = fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["extra"] = true.into();
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(proofs.load(&key).is_err());
}

#[test]
fn spec221_c19_closed_reference_replacement_is_recoverable_and_refuses_partial_temporary() {
    let f = Fixture::new();
    let (proofs, intent, private) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let destination = intent.destination().unwrap();
    export(&intent.source, &intent.manifest, &destination).unwrap();
    proofs.publish(&intent).unwrap();
    let key = intent.binding.key().unwrap();
    let temporary = private.join(format!("{key}.closed.tmp"));
    let original = fs::read(private.join(format!("{key}.intent.json"))).unwrap();
    let lease = proofs.lease(&key).unwrap();
    assert!(proofs.compact_closed(&intent).is_err());
    assert_eq!(
        fs::read(private.join(format!("{key}.intent.json"))).unwrap(),
        original
    );
    drop(lease);
    proofs
        .write_new(&format!("{key}.closed.tmp"), b"{")
        .unwrap();
    assert!(proofs.compact_closed(&intent).is_err());
    assert_eq!(
        fs::read(private.join(format!("{key}.intent.json"))).unwrap(),
        original
    );
    proofs
        .recover(&intent.binding, &intent.source, &intent.data_root)
        .unwrap();
    // Réparer uniquement notre fixture, pas un effacement automatique produit.
    fs::remove_file(&temporary).unwrap();
    proofs
        .write_new(
            &format!("{key}.closed.tmp"),
            &ClosedViewIntent::from_intent(&intent)
                .unwrap()
                .encoded()
                .unwrap(),
        )
        .unwrap();
    ViewProofs::open(&private)
        .unwrap()
        .compact_closed(&intent)
        .unwrap();
    assert!(!temporary.exists());
    assert_eq!(
        proofs.load(&key).unwrap().proof_digest().unwrap(),
        intent.proof_digest().unwrap()
    );
}

#[test]
fn spec221_c19_stopped_job_repeats_the_same_outcome_until_controller_persists_it() {
    let f = Fixture::new();
    let (proofs, intent, _) = private_intent(&f);
    proofs.write_intent(&intent).unwrap();
    let mut job = match ViewJob::start(
        &proofs,
        &intent,
        &ViewJobRequest::Export,
        &std::env::current_exe().unwrap(),
        &view_environment(),
        |_| Ok(()),
    ) {
        Ok(job) => job,
        Err((cause, _)) => panic!("fixture {cause:?}"),
    };
    let completed = wait_view_job(&mut job);
    assert_eq!(completed, JobPoll::Stopped(Ok(())));
    assert_eq!(job.poll().unwrap(), completed);
    assert_eq!(job.poll().unwrap(), completed);
}


#[test]
fn spec221_c19_managed_plan_persists_small_seed_before_git_and_exports_exact_base() {
    use super::job::{ViewJob,JobPoll};
    let f=Fixture::new();let (proofs,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let seed=ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,intent.base_commit(),&f.repo,"fixture:c19").unwrap();
    let key=seed.binding.key().unwrap();
    let before=f.snapshot();
    proofs.write_plan(&seed).unwrap();
    assert!(seed.encoded().unwrap().len()<4096);
    assert!(!proofs.path.join(format!("{key}.intent.json")).exists());
    let attempted=std::cell::Cell::new(false);
    let mut job=ViewJob::start_plan_registered(&proofs,&seed,&std::env::current_exe().unwrap(),&view_environment(),
        ||{
            assert!(!intent.destination().unwrap().exists());
            assert!(!proofs.path.join(format!("{key}.intent.json")).exists());
            attempted.set(true);Ok(())
        },|marker|{
            assert!(attempted.get());
            assert_eq!(proofs.source_marker(&seed.binding).unwrap().as_ref(),Some(marker));
            assert!(!proofs.path.join(format!("{key}.intent.json")).exists());
            Ok(())
        }).unwrap_or_else(|(cause,_)|panic!("plan refused: {cause:?}"));
    assert_eq!(wait_view_job(&mut job),JobPoll::Stopped(Ok(())));
    let planned=proofs.load(&key).unwrap();seed.matches(&planned).unwrap();
    assert_eq!(planned.proof_digest().unwrap(),intent.proof_digest().unwrap());
    assert_eq!(proofs.load_plan(&key).unwrap(),seed);
    assert_eq!(f.snapshot(),before);
    assert_eq!(fs::read(planned.destination().unwrap().join("sub/tracked")).unwrap(),b"initial\n\0binary\xff");
    assert!(!planned.destination().unwrap().join(".git").exists());
    // Même après fin du helper, aucun second export n'est permis.
    assert!(ViewJob::start_plan_registered(&proofs,&seed,&std::env::current_exe().unwrap(),&view_environment(),
        ||panic!("must not fork twice"),|_|panic!("must not release twice")).is_err());
}

#[test]
fn spec221_c19_plan_seed_never_renews_deadline_or_accepts_other_roots() {
    let f=Fixture::new();let (proofs,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let mut seed=ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,intent.base_commit(),&f.repo,"fixture:c19").unwrap();
    proofs.write_plan(&seed).unwrap();
    seed.created_at-=1;seed.expires_at-=1;
    assert!(proofs.write_plan(&seed).unwrap_err().contains("proof_binding_mismatch"));
    let mut changed=intent.clone();changed.source=f.root.join("other");
    assert!(seed.matches(&changed).is_err());
    seed.created_at=1;seed.expires_at=121;
    assert!(seed.live().unwrap_err().contains("preparation_expired"));
    assert!(super::job::ViewJob::start_plan_registered(&proofs,&seed,&std::env::current_exe().unwrap(),&view_environment(),
        ||panic!("expired seed must not fork"),|_|panic!("expired seed must not release")).is_err());
    assert!(super::job::run_commit_view_plan(&[]).unwrap_err().contains("managed_context_required"));
}


#[test]
#[ignore = "Recette privée : BRIDGET_C19_BINARY doit désigner le binaire natif construit depuis ce snapshot."]
fn spec221_c19_native_binary_plans_exports_and_reports_without_provider_or_git_workspace() {
    let binary=PathBuf::from(std::env::var_os("BRIDGET_C19_BINARY").expect("BRIDGET_C19_BINARY requis"));
    assert!(binary.is_absolute() && binary.is_file());
    let f=Fixture::new();let (proofs,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let before=f.snapshot();
    let seed=ViewPlan::new(intent.binding.clone(),&intent.source,&intent.data_root,intent.base_commit(),&f.repo,"fixture:c19").unwrap();
    proofs.write_plan(&seed).unwrap();
    let mut env=view_environment();env.insert("B221_VIEW_NATIVE_BINARY".into(),"1".into());
    let mut job=match super::job::ViewJob::start_plan_registered(&proofs,&seed,&binary,&env,
        ||Ok(()),|marker|{
            assert_eq!(proofs.source_marker(&seed.binding).unwrap().as_ref(),Some(marker));Ok(())
        }) {
        Ok(job)=>job,
        Err((cause,mut retained))=>{
            if let Some(job)=retained.as_mut(){assert!(job.stop(Duration::from_secs(2)).unwrap());}
            panic!("native recipe failed: {cause:?}");
        }
    };
    assert_eq!(wait_view_job(&mut job),super::job::JobPoll::Stopped(Ok(())));
    let planned=proofs.load(&seed.binding.key().unwrap()).unwrap();
    seed.matches(&planned).unwrap();
    let record=proofs.workspace(&planned).unwrap();
    assert_eq!(record.base_commit,intent.base_commit());
    assert_eq!(record.dirty,Some(false));
    assert_eq!(fs::read(planned.destination().unwrap().join("sub/tracked")).unwrap(),b"initial\n\0binary\xff");
    assert_eq!(f.snapshot(),before);
    assert!(!planned.destination().unwrap().join(".git").exists());
    assert!(!planned.destination().unwrap().join("large.bin").exists());
    // Même image réelle, sans fournisseur : le fichier omis passe lui aussi
    // par le bootstrap géré de production et sa corrélation exacte.
    let mut command=planned.binding.command.clone();command.instance_id="native-file-reader".into();command.generation=1;
    let request=super::job::ViewJobRequest::Materialize{command,sequence:1,path:"large.bin".into()};
    let expected=request.identity(&planned).unwrap();
    // La possession de l'export garde son verrou jusqu'à son retrait du runtime.
    // Le conflit est attesté avant fork ; ne pas déduire un arrêt du seul résultat.
    assert!(matches!(super::job::ViewJob::start_registered(&proofs,&planned,&request,&binary,&env,
        ||panic!("lease busy must refuse before fork"),|_|panic!("lease busy must not release")),
        Err((super::job::ExportFailure::OperationCapacity,None))));
    drop(job);
    let mut file=match super::job::ViewJob::start_registered(&proofs,&planned,&request,&binary,&env,
        ||Ok(()),|marker|{
            assert_eq!(proofs.operation_marker(&planned.binding,&expected).unwrap().as_ref(),Some(marker));Ok(())
        }) {
        Ok(file)=>file,
        Err((cause,mut retained))=>{
            if let Some(file)=retained.as_mut(){assert!(file.stop(Duration::from_secs(2)).unwrap());}
            panic!("native file recipe failed: {cause:?}");
        }
    };
    assert_eq!(wait_view_job(&mut file),super::job::JobPoll::Stopped(Ok(())));
    assert!(proofs.file_available(&planned,"large.bin").unwrap());
    assert_eq!(fs::read(planned.destination().unwrap().join("large.bin")).unwrap(),vec![b'x';200000]);
    assert_eq!(f.snapshot(),before);
    eprintln!("C19_NATIVE_RECIPE {}",serde_json::json!({
        "binary_sha256":view_digest(&fs::read(&binary).unwrap()),
        "base":record.base_commit,"dirty":record.dirty,"source_unchanged":true,
        "native_bootstrap":true,"native_file_materialized":true,"provider_started":false,"git_workspace_created":false,
        "group_stopped":job::source_group_stopped(&proofs,&planned).unwrap()
    }));
}


#[test]
fn spec221_c19_native_placement_creates_private_sibling_only_after_base_is_known() {
    let f=Fixture::new();let (_,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let data=ViewPlan::native_data_root(&f.repo).unwrap();
    let seed=ViewPlan::new(intent.binding.clone(),&f.repo,&data,intent.base_commit(),&f.repo,"fixture:c19").unwrap();
    let before=f.snapshot();
    assert!(!data.exists());
    let mut missing=seed.clone();missing.base="f".repeat(40);
    assert!(missing.planned_intent().unwrap_err().contains("commit_unavailable"));
    assert!(!data.exists());
    eprintln!("C19_PLACEMENT_OBSERVED {}",serde_json::json!({
        "uid":unsafe{libc::geteuid()},
        "parent_mode":fs::metadata(&f.root).unwrap().mode()&0o777,
        "parent_uid":fs::metadata(&f.root).unwrap().uid(),
        "parent_dev":fs::metadata(&f.root).unwrap().dev(),
        "source_dev":fs::metadata(&f.repo).unwrap().dev(),
        "reserve_exists":data.exists()
    }));
    let planned=seed.planned_intent().unwrap();
    assert_eq!(planned.data_root,data);
    assert_eq!(fs::symlink_metadata(&data).unwrap().mode()&0o777,0o700);
    assert!(!planned.destination().unwrap().exists()); // plan ne publie ni vue ni record
    assert_eq!(f.snapshot(),before);
}

#[test]
fn spec221_c19_native_placement_refuses_submapping_inside_git_without_effects() {
    let f=Fixture::new();let (_,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let source=f.repo.join("sub");let data=ViewPlan::native_data_root(&source).unwrap();
    let seed=ViewPlan::new(intent.binding.clone(),&source,&data,intent.base_commit(),&source,"fixture:c19").unwrap();
    let before=f.snapshot();
    assert!(seed.planned_intent().unwrap_err().contains("destination_inside_source"));
    assert!(!data.exists());
    assert_eq!(f.snapshot(),before);
}

#[test]
fn spec221_c19_native_placement_never_changes_existing_permissions_or_follows_reserve_link() {
    let f=Fixture::new();let (_,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let data=ViewPlan::native_data_root(&f.repo).unwrap();
    fs::create_dir(&data).unwrap();
    fs::set_permissions(&data,fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(data.join("kept"),b"kept").unwrap();
    let seed=ViewPlan::new(intent.binding.clone(),&f.repo,&data,intent.base_commit(),&f.repo,"fixture:c19").unwrap();
    assert!(seed.planned_intent().unwrap_err().contains("destination_invalid"));
    assert_eq!(fs::symlink_metadata(&data).unwrap().mode()&0o777,0o755);
    assert_eq!(fs::read(data.join("kept")).unwrap(),b"kept");
    let outside=f.root.join("outside");
    fs::rename(&data,&outside).unwrap();
    std::os::unix::fs::symlink(&outside,&data).unwrap();
    assert!(seed.planned_intent().unwrap_err().contains("destination_invalid"));
    assert!(fs::symlink_metadata(&data).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(outside.join("kept")).unwrap(),b"kept");
}


#[test]
fn spec221_c19_native_placement_resolves_submapping_outside_git_without_changing_shared() {
    let f=Fixture::new();let (_,intent,_)=private_intent(&f);
    f.git(&["config","remote.origin.url","fixture:c19"]);
    let source=f.repo.join("sub");
    let seed=ViewPlan::new_native(intent.binding.clone(),&source,intent.base_commit(),&source,"fixture:c19").unwrap();
    let before=f.snapshot();
    let planned=seed.planned_intent().unwrap();
    assert_eq!(planned.data_root,f.root.join(".bridget-commit-views"));
    assert!(!f.repo.join(".bridget-commit-views").exists());
    seed.matches(&planned).unwrap();
    let mut wrong=planned.clone();wrong.data_root=f.root.join("unapproved");
    assert!(seed.matches(&wrong).is_err());
    wrong=planned.clone();wrong.data_root=f.repo.join(".bridget-commit-views");
    assert!(seed.matches(&wrong).is_err(),"un ancêtre du sous-mapping situé dans Git n'est jamais une destination");
    assert!(!wrong.data_root.exists());
    assert_eq!(f.snapshot(),before);
}
