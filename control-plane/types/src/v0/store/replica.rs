//! Definition of replica types that can be saved to the persistent store.

use crate::v0::{
    message_bus::{
        mbus,
        mbus::{
            CreateReplica, NodeId, PoolId, Protocol, ReplicaId, ReplicaOwners, ReplicaShareProtocol,
        },
    },
    store::{
        definitions::{ObjectKey, StorableObject, StorableObjectType},
        SpecState, SpecTransaction,
    },
};
use serde::{Deserialize, Serialize};

use paperclip::actix::Apiv2Schema;

/// Replica information
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Replica {
    /// Current state of the replica.
    pub state: Option<ReplicaState>,
    /// Desired replica specification.
    pub spec: ReplicaSpec,
}

/// Runtime state of a replica.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReplicaState {
    /// Replica information.
    pub replica: mbus::Replica,
    /// State of the replica.
    pub state: mbus::ReplicaState,
}

/// Key used by the store to uniquely identify a ReplicaState structure.
pub struct ReplicaStateKey(ReplicaId);

impl ObjectKey for ReplicaStateKey {
    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::ReplicaState
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl StorableObject for ReplicaState {
    type Key = ReplicaStateKey;

    fn key(&self) -> Self::Key {
        ReplicaStateKey(self.replica.uuid.clone())
    }
}

/// User specification of a replica.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub struct ReplicaSpec {
    /// uuid of the replica
    pub uuid: ReplicaId,
    /// The size that the replica should be.
    pub size: u64,
    /// The pool that the replica should live on.
    pub pool: PoolId,
    /// Protocol used for exposing the replica.
    pub share: Protocol,
    /// Thin provisioning.
    pub thin: bool,
    /// The state that the replica should eventually achieve.
    pub state: ReplicaSpecState,
    /// Managed by our control plane
    pub managed: bool,
    /// Owner Resource
    pub owners: ReplicaOwners,
    /// Update in progress
    #[serde(skip)]
    pub updating: bool,
    /// Record of the operation in progress
    pub operation: Option<ReplicaOperationState>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub struct ReplicaOperationState {
    /// Record of the operation
    pub operation: ReplicaOperation,
    /// Result of the operation
    pub result: Option<bool>,
}

impl SpecTransaction<ReplicaOperation> for ReplicaSpec {
    fn pending_op(&self) -> bool {
        self.operation.is_some()
    }

    fn commit_op(&mut self) {
        if let Some(op) = self.operation.clone() {
            match op.operation {
                ReplicaOperation::Unknown => unreachable!(),
                ReplicaOperation::Create => {
                    self.state = SpecState::Created(mbus::ReplicaState::Online);
                }
                ReplicaOperation::Destroy => {
                    self.state = SpecState::Deleted;
                }
                ReplicaOperation::Share(share) => {
                    self.share = share.into();
                }
                ReplicaOperation::Unshare => {
                    self.share = Protocol::Off;
                }
            }
        }
        self.clear_op();
    }

    fn clear_op(&mut self) {
        self.operation = None;
        self.updating = false;
    }

    fn start_op(&mut self, operation: ReplicaOperation) {
        self.updating = true;
        self.operation = Some(ReplicaOperationState {
            operation,
            result: None,
        })
    }

    fn set_op_result(&mut self, result: bool) {
        if let Some(op) = &mut self.operation {
            op.result = Some(result);
        }
        self.updating = false;
    }
}

/// Available Replica Operations
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub enum ReplicaOperation {
    Unknown,
    Create,
    Destroy,
    Share(ReplicaShareProtocol),
    Unshare,
}

impl Default for ReplicaOperation {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Key used by the store to uniquely identify a ReplicaSpec structure.
pub struct ReplicaSpecKey(ReplicaId);

impl ObjectKey for ReplicaSpecKey {
    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::ReplicaSpec
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl From<&ReplicaId> for ReplicaSpecKey {
    fn from(id: &ReplicaId) -> Self {
        ReplicaSpecKey(id.clone())
    }
}

impl StorableObject for ReplicaSpec {
    type Key = ReplicaSpecKey;

    fn key(&self) -> Self::Key {
        ReplicaSpecKey(self.uuid.clone())
    }
}

impl From<&ReplicaSpec> for mbus::Replica {
    fn from(replica: &ReplicaSpec) -> Self {
        Self {
            node: NodeId::default(),
            uuid: replica.uuid.clone(),
            pool: replica.pool.clone(),
            thin: replica.thin,
            size: replica.size,
            share: replica.share.clone(),
            uri: "".to_string(),
            state: mbus::ReplicaState::Unknown,
        }
    }
}

/// State of the Replica Spec
pub type ReplicaSpecState = SpecState<mbus::ReplicaState>;

impl From<&CreateReplica> for ReplicaSpec {
    fn from(request: &CreateReplica) -> Self {
        Self {
            uuid: request.uuid.clone(),
            size: request.size,
            pool: request.pool.clone(),
            share: request.share.clone(),
            thin: request.thin,
            state: ReplicaSpecState::Creating,
            managed: request.managed,
            owners: request.owners.clone(),
            updating: false,
            operation: None,
        }
    }
}
impl PartialEq<CreateReplica> for ReplicaSpec {
    fn eq(&self, other: &CreateReplica) -> bool {
        let mut other = ReplicaSpec::from(other);
        other.state = self.state.clone();
        other.updating = self.updating;
        &other == self
    }
}
impl PartialEq<mbus::Replica> for ReplicaSpec {
    fn eq(&self, _other: &mbus::Replica) -> bool {
        true
    }
}
