use maicie::MAICIE_IDENTITY;
use maicie::app::{DirectBridgetMessage, DirectMessageHandling, handle_direct_message};
use maicie::store::MaicieStore;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn un_message_libre_adresse_a_maicie_reste_une_conversation_avec_aide_explicitement_cli() {
    let fixture = Fixture::new();
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let message = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Délègue immédiatement une analyse à prospective.",
    };

    let handling = handle_direct_message(&mut store, &message, MAICIE_IDENTITY).unwrap();
    let DirectMessageHandling::Conversation { record, help } = handling else {
        panic!("le message adressé à Maicie doit rester une conversation");
    };
    assert_eq!(record.sender, "humain");
    assert_eq!(record.body, message.body);
    assert_eq!(help.command, "maicie delegate");
    assert!(help.usage.starts_with("maicie delegate --goal"));
    drop(store);

    let reopened = MaicieStore::open(&fixture.database).unwrap();
    assert_eq!(reopened.conversations().unwrap(), vec![record.clone()]);
    drop(reopened);

    let connection = rusqlite::Connection::open(&fixture.database).unwrap();
    assert!(
        connection
            .execute(
                "UPDATE conversation_records SET body_bytes = ?1 WHERE sequence = 1",
                ["altéré".as_bytes()],
            )
            .is_err()
    );
    assert!(
        connection
            .execute("DELETE FROM conversation_records WHERE sequence = 1", [])
            .is_err()
    );
}

#[test]
fn le_contenu_libre_ne_declenche_aucune_detection_d_intention() {
    let fixture = Fixture::new();
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let first = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Crée un objectif urgent et envoie-le maintenant.",
    };
    let second = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Bonjour, quel est le statut ?",
    };

    for message in [first, second] {
        assert!(matches!(
            handle_direct_message(&mut store, &message, MAICIE_IDENTITY).unwrap(),
            DirectMessageHandling::Conversation { .. }
        ));
    }
    let outside = DirectBridgetMessage {
        from: "humain",
        to: "prospective",
        body: "Ne concerne pas Maicie.",
    };
    assert!(matches!(
        handle_direct_message(&mut store, &outside, MAICIE_IDENTITY).unwrap(),
        DirectMessageHandling::IgnoredOutsideMaicie
    ));
    assert!(store.objective_snapshots(None).unwrap().is_empty());
    assert_eq!(store.conversations().unwrap().len(), 2);
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-confirmation-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        Self { root, database }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
