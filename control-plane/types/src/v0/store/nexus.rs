//! Definition of nexus types that can be saved to the persistent store.

use crate::v0::{
    message_bus::{
        mbus,
        mbus::{
            ChildState, ChildUri, CreateNexus, DestroyNexus, NexusId, NexusShareProtocol, NodeId,
            Protocol, VolumeId,
        },
    },
    store::{
        definitions::{ObjectKey, StorableObject, StorableObjectType},
        SpecState, SpecTransaction,
    },
};

use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

/// Nexus information
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Nexus {
    /// Current state of the nexus.
    pub state: Option<mbus::NexusState>,
    /// Desired nexus specification.
    pub spec: NexusSpec,
}

/// Runtime state of the nexus.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NexusState {
    /// Nexus information.
    pub nexus: mbus::Nexus,
}

/// Key used by the store to uniquely identify a NexusState structure.
pub struct NexusStateKey(NexusId);

impl From<&NexusId> for NexusStateKey {
    fn from(id: &NexusId) -> Self {
        Self(id.clone())
    }
}

impl ObjectKey for NexusStateKey {
    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::NexusState
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl StorableObject for NexusState {
    type Key = NexusStateKey;

    fn key(&self) -> Self::Key {
        NexusStateKey(self.nexus.uuid.clone())
    }
}

/// State of the Nexus Spec
pub type NexusSpecState = SpecState<mbus::NexusState>;

/// User specification of a nexus.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub struct NexusSpec {
    /// Nexus Id
    pub uuid: NexusId,
    /// Node where the nexus should live.
    pub node: NodeId,
    /// List of children.
    pub children: Vec<ChildUri>,
    /// Size of the nexus.
    pub size: u64,
    /// The state the nexus should eventually reach.
    pub state: NexusSpecState,
    /// Share Protocol
    pub share: Protocol,
    /// Managed by our control plane
    pub managed: bool,
    /// Volume which owns this nexus, if any
    pub owner: Option<VolumeId>,
    /// Update of the state in progress
    #[serde(skip)]
    pub updating: bool,
    /// Record of the operation in progress
    pub operation: Option<NexusOperationState>,
}

/// Operation State for a Nexus spec resource
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub struct NexusOperationState {
    /// Record of the operation
    pub operation: NexusOperation,
    /// Result of the operation
    pub result: Option<bool>,
}

impl SpecTransaction<NexusOperation> for NexusSpec {
    fn pending_op(&self) -> bool {
        self.operation.is_some()
    }

    fn commit_op(&mut self) {
        if let Some(op) = self.operation.clone() {
            match op.operation {
                NexusOperation::Unknown => unimplemented!(),
                NexusOperation::Destroy => {
                    self.state = SpecState::Deleted;
                }
                NexusOperation::Create => {
                    self.state = SpecState::Created(mbus::NexusState::Online);
                }
                NexusOperation::Share(share) => {
                    self.share = share.into();
                }
                NexusOperation::Unshare => {
                    self.share = Protocol::Off;
                }
                NexusOperation::AddChild(uri) => self.children.push(uri),
                NexusOperation::RemoveChild(uri) => self.children.retain(|c| c != &uri),
            }
        }
        self.clear_op();
    }

    fn clear_op(&mut self) {
        self.operation = None;
        self.updating = false;
    }

    fn start_op(&mut self, operation: NexusOperation) {
        self.updating = true;
        self.operation = Some(NexusOperationState {
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

/// Available Nexus Operations
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub enum NexusOperation {
    Unknown,
    Create,
    Destroy,
    Share(NexusShareProtocol),
    Unshare,
    AddChild(ChildUri),
    RemoveChild(ChildUri),
}

impl Default for NexusOperation {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Key used by the store to uniquely identify a NexusSpec structure.
pub struct NexusSpecKey(NexusId);

impl From<&NexusId> for NexusSpecKey {
    fn from(id: &NexusId) -> Self {
        Self(id.clone())
    }
}

impl ObjectKey for NexusSpecKey {
    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::NexusSpec
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl StorableObject for NexusSpec {
    type Key = NexusSpecKey;

    fn key(&self) -> Self::Key {
        NexusSpecKey(self.uuid.clone())
    }
}

impl From<&CreateNexus> for NexusSpec {
    fn from(request: &CreateNexus) -> Self {
        Self {
            uuid: request.uuid.clone(),
            node: request.node.clone(),
            children: request.children.clone(),
            size: request.size,
            state: NexusSpecState::Creating,
            share: Protocol::Off,
            managed: request.managed,
            owner: request.owner.clone(),
            updating: false,
            operation: None,
        }
    }
}

impl PartialEq<CreateNexus> for NexusSpec {
    fn eq(&self, other: &CreateNexus) -> bool {
        let mut other = NexusSpec::from(other);
        other.state = self.state.clone();
        other.updating = self.updating;
        &other == self
    }
}
impl PartialEq<mbus::Nexus> for NexusSpec {
    fn eq(&self, _other: &mbus::Nexus) -> bool {
        true
    }
}

impl From<&NexusSpec> for mbus::Nexus {
    fn from(nexus: &NexusSpec) -> Self {
        Self {
            node: nexus.node.clone(),
            uuid: nexus.uuid.clone(),
            size: nexus.size,
            state: mbus::NexusState::Unknown,
            children: nexus
                .children
                .iter()
                .map(|uri| mbus::Child {
                    uri: uri.clone(),
                    state: ChildState::Unknown,
                    rebuild_progress: None,
                })
                .collect(),
            device_uri: "".to_string(),
            rebuilds: 0,
            share: nexus.share.clone(),
        }
    }
}

impl From<NexusSpec> for DestroyNexus {
    fn from(nexus: NexusSpec) -> Self {
        Self {
            node: nexus.node,
            uuid: nexus.uuid,
        }
    }
}
