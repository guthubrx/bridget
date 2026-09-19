//! Managers spécialisés pour séparer les responsabilités du daemon

use std::collections::HashMap;
use std::io::BufWriter;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Gère les connexions actives des wrappers
pub struct ConnectionManager {
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    conn_counter: u64,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
            conn_counter: 0,
        }
    }

    pub fn register_connection(&mut self, conn_id: String, writer: BufWriter<UnixStream>) -> u64 {
        let conn_id = conn_id.clone();
        self.connections
            .insert(conn_id.clone(), Arc::new(Mutex::new(writer)));
        self.conn_counter += 1;
        self.conn_counter
    }

    pub fn set_connection_info(
        &mut self,
        conn_id: &str,
        name: String,
        host: String,
        os: String,
        instance_id: Option<String>,
    ) {
        self.conn_names.insert(conn_id.to_string(), name);
        self.conn_hosts.insert(conn_id.to_string(), host);
        self.conn_operating_systems.insert(conn_id.to_string(), os);
        if let Some(instance_id) = instance_id {
            self.conn_instances.insert(conn_id.to_string(), instance_id);
        }
    }

    pub fn remove_connection(
        &mut self,
        conn_id: &str,
    ) -> Option<Arc<Mutex<BufWriter<UnixStream>>>> {
        let writer_opt = self.connections.remove(conn_id);
        self.conn_names.remove(conn_id);
        self.conn_hosts.remove(conn_id);
        self.conn_operating_systems.remove(conn_id);
        self.conn_instances.remove(conn_id);
        writer_opt
    }

    pub fn get_writer(&self, conn_id: &str) -> Option<Arc<Mutex<BufWriter<UnixStream>>>> {
        self.connections.get(conn_id).cloned()
    }

    pub fn get_name(&self, conn_id: &str) -> Option<&String> {
        self.conn_names.get(conn_id)
    }

    pub fn get_host(&self, conn_id: &str) -> Option<&String> {
        self.conn_hosts.get(conn_id)
    }

    pub fn get_os(&self, conn_id: &str) -> Option<&String> {
        self.conn_operating_systems.get(conn_id)
    }

    pub fn iter_connections(&self) -> impl Iterator<Item = (&String, &String)> {
        self.conn_names.iter()
    }
}

/// Gère les demandes suivies et leur escalade
pub struct RequestManager {
    pending_replies: Vec<PendingReply>,
}

#[derive(Clone, Debug)]
pub struct PendingReply {
    pub msg_id: String,
    pub sender: String,
    pub target: String,
    pub created_at: Instant,
    pub timeout_secs: u64,
    pub escalation_level: u32,
}

impl Default for RequestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestManager {
    pub fn new() -> Self {
        Self {
            pending_replies: Vec::new(),
        }
    }

    pub fn add_request(&mut self, request: PendingReply) {
        self.pending_replies.push(request);
    }

    pub fn remove_request(&mut self, msg_id: &str) -> bool {
        if let Some(pos) = self.pending_replies.iter().position(|r| r.msg_id == msg_id) {
            self.pending_replies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_requests_by_sender(&self, sender: &str) -> Vec<&PendingReply> {
        self.pending_replies
            .iter()
            .filter(|r| r.sender == sender)
            .collect()
    }

    pub fn get_request(&self, msg_id: &str) -> Option<&PendingReply> {
        self.pending_replies.iter().find(|r| r.msg_id == msg_id)
    }

    pub fn iter_mut_requests(&mut self) -> impl Iterator<Item = &mut PendingReply> {
        self.pending_replies.iter_mut()
    }

    pub fn cleanup_expired(&mut self, max_age_secs: u64) {
        let now = Instant::now();
        self.pending_replies.retain(|r| {
            let elapsed = now.duration_since(r.created_at).as_secs();
            elapsed <= max_age_secs
        });
    }
}
