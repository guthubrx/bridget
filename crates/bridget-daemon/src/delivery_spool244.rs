//! Débordement durable avant Seen. Le maître reste l'autorité de remise.
use bridget_transport::protocol::DaemonToWrapper;
use rusqlite::{Connection, OptionalExtension, params};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

pub(crate) struct DeliverySpool {
    conn: Connection,
    instance: String,
}
impl DeliverySpool {
    pub(crate) fn open(path: &Path, instance: &str) -> Result<Self, String> {
        crate::environment::ensure_private_directory(path.parent().ok_or("spool_parent_missing")?)?;
        let file = std::fs::OpenOptions::new().read(true).write(true).create(true)
            .truncate(false).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(path).map_err(|e| e.to_string())?;
        let m = file.metadata().map_err(|e| e.to_string())?;
        if !m.is_file() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 || m.nlink() != 1 {
            return Err("spool_private_file_required".into());
        }
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
            | rusqlite::OpenFlags::from_bits_retain(rusqlite::ffi::SQLITE_OPEN_NOFOLLOW))
            .map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(1)).map_err(|e|e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS inbox244 (
                seq INTEGER PRIMARY KEY, delivery_id TEXT NOT NULL UNIQUE,
                instance TEXT NOT NULL, message_id TEXT NOT NULL, frame BLOB NOT NULL);
            CREATE INDEX IF NOT EXISTS inbox244_message ON inbox244(message_id);").map_err(|e|e.to_string())?;
        let foreign: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM inbox244 WHERE instance<>?1)", [instance], |r|r.get(0)).map_err(|e|e.to_string())?;
        if foreign { return Err("spool_instance_changed".into()); }
        Ok(Self { conn, instance: instance.into() })
    }
    pub(crate) fn has_pending(&self) -> bool {
        // Une erreur de lecture ne peut pas annoncer la file vide.
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM inbox244)", [], |r| r.get(0)).unwrap_or(true)
    }
    pub(crate) fn defer(&self, frame: &DaemonToWrapper) -> Result<(), String> {
        let DaemonToWrapper::DeliverIdempotent { delivery_id, recipient_instance_id, message, .. } = frame else {
            return Err("spool_idempotent_delivery_required".into());
        };
        if recipient_instance_id != &self.instance { return Err("spool_instance_changed".into()); }
        let bytes = serde_json::to_vec(frame).map_err(|e|e.to_string())?;
        let previous: Option<Vec<u8>> = self.conn.query_row("SELECT frame FROM inbox244 WHERE delivery_id=?1",
            [delivery_id], |r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if let Some(previous) = previous {
            return if previous == bytes { Ok(()) } else { Err("spool_envelope_changed".into()) };
        }
        self.conn.execute("INSERT INTO inbox244(delivery_id,instance,message_id,frame) VALUES(?1,?2,?3,?4)",
            params![delivery_id,self.instance,message.id,bytes]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub(crate) fn cancel(&self, message_id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM inbox244 WHERE message_id=?1",[message_id]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub(crate) fn next(&mut self) -> Result<Option<DaemonToWrapper>, String> {
        let tx = self.conn.transaction().map_err(|e|e.to_string())?;
        let first: Option<(i64,usize)> = tx.query_row("SELECT seq,length(frame) FROM inbox244 ORDER BY seq LIMIT 1", [],
            |r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((seq,len)) = first else { return Ok(None); };
        if len >= bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES { return Err("spool_frame_corrupt".into()); }
        let bytes:Vec<u8>=tx.query_row("SELECT frame FROM inbox244 WHERE seq=?1",[seq],|r|r.get(0)).map_err(|e|e.to_string())?;
        let frame: DaemonToWrapper = serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
        if !matches!(&frame,DaemonToWrapper::DeliverIdempotent{recipient_instance_id,..} if recipient_instance_id==&self.instance) {
            return Err("spool_instance_changed".into());
        }
        // Avant Seen seulement. Si le processus s'arrête ensuite, le maître
        // reprend la même remise ; les reçus existants interdisent un doublon.
        tx.execute("DELETE FROM inbox244 WHERE seq=?1",[seq]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e|e.to_string())?;
        Ok(Some(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(id: &str) -> DaemonToWrapper {
        let mut message=bridget_core::BridgetMessage::new("a","b",id);
        message.id=id.into();
        DaemonToWrapper::DeliverIdempotent { delivery_id:id.into(),recipient_instance_id:"instance".into(),
            delivery_generation:1,expires_at:200,message,
            execution:None,mid_turn_ok:None }
    }
    #[test]
    fn spec244_debordement_durable_ordre_doublon_et_identite() {
        let root=std::env::temp_dir().join(format!("spool244-{}",uuid::Uuid::new_v4()));
        let path=root.join("queue.sqlite");
        let spool=DeliverySpool::open(&path,"instance").unwrap();
        for i in 0..200 { spool.defer(&frame(&format!("d-{i}"))).unwrap(); }
        spool.defer(&frame("d-0")).unwrap();
        let mut changed=frame("d-0");
        if let DaemonToWrapper::DeliverIdempotent{delivery_generation,..}=&mut changed { *delivery_generation=2; }
        assert!(spool.defer(&changed).is_err());
        drop(spool);
        assert!(DeliverySpool::open(&path,"other-instance").is_err());
        let mut spool=DeliverySpool::open(&path,"instance").unwrap();
        for i in 0..200 {
            assert!(matches!(spool.next().unwrap(),Some(DaemonToWrapper::DeliverIdempotent{delivery_id,..}) if delivery_id==format!("d-{i}")));
        }
        assert!(!spool.has_pending());
        drop(spool); std::fs::remove_dir_all(root).unwrap();
    }
}
