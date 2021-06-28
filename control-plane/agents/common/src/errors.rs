use mbus_api::{message_bus::v0::BusError, ErrorChain, ReplyError, ReplyErrorKind, ResourceKind};
use snafu::{Error, Snafu};
use tonic::Code;
use types::v0::{
    message_bus::mbus::{Filter, NodeId, PoolId, ReplicaId},
    store::definitions::StoreError,
};

/// Common error type for send/receive
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
#[allow(missing_docs)]
pub enum SvcError {
    #[snafu(display("Failed to get node '{}' from the node agent", node))]
    BusGetNode { node: String, source: BusError },
    #[snafu(display("Failed to get nodes from the node agent"))]
    BusGetNodes { source: BusError },
    #[snafu(display("Node '{}' is not online", node))]
    NodeNotOnline { node: NodeId },
    #[snafu(display("No available online nodes"))]
    NoNodes {},
    #[snafu(display(
        "Timed out after '{:?}' attempting to connect to node '{}' via gRPC endpoint '{}'",
        timeout,
        node_id,
        endpoint
    ))]
    GrpcConnectTimeout {
        node_id: String,
        endpoint: String,
        timeout: std::time::Duration,
    },
    #[snafu(display("Failed to connect to node via gRPC"))]
    GrpcConnect { source: tonic::transport::Error },
    #[snafu(display("Node '{}' has invalid gRPC URI '{}'", node_id, uri))]
    GrpcConnectUri {
        node_id: String,
        uri: String,
        source: http::uri::InvalidUri,
    },
    #[snafu(display(
        "gRPC request '{}' for '{}' failed with '{}'",
        request,
        resource.to_string(),
        source
    ))]
    GrpcRequestError {
        resource: ResourceKind,
        request: String,
        source: tonic::Status,
    },
    #[snafu(display("Node '{}' not found", node_id))]
    NodeNotFound { node_id: NodeId },
    #[snafu(display("Pool '{}' not found", pool_id))]
    PoolNotFound { pool_id: PoolId },
    #[snafu(display("Nexus '{}' not found", nexus_id))]
    NexusNotFound { nexus_id: String },
    #[snafu(display("{} '{}' not found", kind.to_string(), id))]
    NotFound { kind: ResourceKind, id: String },
    #[snafu(display("{} '{}' is still being created..", kind.to_string(), id))]
    PendingCreation { kind: ResourceKind, id: String },
    #[snafu(display("Child '{}' not found in Nexus '{}'", child, nexus))]
    ChildNotFound { nexus: String, child: String },
    #[snafu(display("Child '{}' already exists in Nexus '{}'", child, nexus))]
    ChildAlreadyExists { nexus: String, child: String },
    #[snafu(display("Volume '{}' not found", vol_id))]
    VolumeNotFound { vol_id: String },
    #[snafu(display("Volume '{}' not published", vol_id))]
    VolumeNotPublished { vol_id: String },
    #[snafu(display(
        "Volume '{}' is already published on node '{}' with protocol '{}'",
        vol_id,
        node,
        protocol
    ))]
    VolumeAlreadyPublished {
        vol_id: String,
        node: String,
        protocol: String,
    },
    #[snafu(display("Replica '{}' not found", replica_id))]
    ReplicaNotFound { replica_id: ReplicaId },
    #[snafu(display("{} '{}' is already shared over {}", kind.to_string(), id, share))]
    AlreadyShared {
        kind: ResourceKind,
        id: String,
        share: String,
    },
    #[snafu(display("{} '{}' is not shared", kind.to_string(), id))]
    NotShared { kind: ResourceKind, id: String },
    #[snafu(display("Invalid filter value: {:?}", filter))]
    InvalidFilter { filter: Filter },
    #[snafu(display("Operation failed due to insufficient resources"))]
    NotEnoughResources { source: NotEnough },
    #[snafu(display("Failed to deserialise JsonRpc response"))]
    JsonRpcDeserialise { source: serde_json::Error },
    #[snafu(display(
        "Json RPC call failed for method '{}' with parameters '{}'. Error {}",
        method,
        params,
        error,
    ))]
    JsonRpc {
        method: String,
        params: String,
        error: String,
    },
    #[snafu(display("Internal error: {}", details))]
    Internal { details: String },
    #[snafu(display("Message Bus error"))]
    MBusError { source: mbus_api::Error },
    #[snafu(display("Invalid Arguments"))]
    InvalidArguments {},
    #[snafu(display("Multiple nexuses not supported"))]
    MultipleNexuses {},
    #[snafu(display("Storage Error: {}", source.to_string()))]
    Store { source: StoreError },
    #[snafu(display("Storage Error: {} Config for Resource id {} not committed to the store", kind.to_string(), id))]
    StoreSave { kind: ResourceKind, id: String },
    #[snafu(display("Watch Config Not Found"))]
    WatchNotFound {},
    #[snafu(display("{} Resource to be watched does not exist", kind.to_string()))]
    WatchResourceNotFound { kind: ResourceKind },
    #[snafu(display("Watch Already Exists"))]
    WatchAlreadyExists {},
    #[snafu(display("Conflicts with existing operation - please retry"))]
    Conflict {},
    #[snafu(display("Pending deletion - please retry"))]
    Deleting {},
    #[snafu(display(
        "Retried creation of resource id {} kind {} with different parameters",
        id,
        kind.to_string()
    ))]
    ReCreateMismatch { id: String, kind: ResourceKind },
    #[snafu(display("{} Resource id {} needs to be reconciled. Please retry", kind.to_string(), id))]
    NotReady { kind: ResourceKind, id: String },
    #[snafu(display("{} Resource id {} still in still use", kind.to_string(), id))]
    InUse { kind: ResourceKind, id: String },
    #[snafu(display("{} Resource id {} already exists", kind.to_string(), id))]
    AlreadyExists { kind: ResourceKind, id: String },
}

impl From<StoreError> for SvcError {
    fn from(source: StoreError) -> Self {
        SvcError::Store { source }
    }
}

impl From<mbus_api::Error> for SvcError {
    fn from(source: mbus_api::Error) -> Self {
        Self::MBusError { source }
    }
}

impl From<NotEnough> for SvcError {
    fn from(source: NotEnough) -> Self {
        Self::NotEnoughResources { source }
    }
}

impl From<SvcError> for ReplyError {
    fn from(error: SvcError) -> Self {
        #[allow(deprecated)]
        let desc: &String = &error.description().to_string();
        let error_str = error.full_string();
        match error {
            SvcError::StoreSave { kind, .. } => ReplyError {
                kind: ReplyErrorKind::FailedPersist,
                resource: kind,
                source: desc.to_string(),
                extra: error_str,
            },
            SvcError::NotShared { kind, .. } => ReplyError {
                kind: ReplyErrorKind::NotShared,
                resource: kind,
                source: desc.to_string(),
                extra: error_str,
            },
            SvcError::AlreadyShared { kind, .. } => ReplyError {
                kind: ReplyErrorKind::AlreadyShared,
                resource: kind,
                source: desc.to_string(),
                extra: error_str,
            },
            SvcError::ChildNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Child,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::ChildAlreadyExists { .. } => ReplyError {
                kind: ReplyErrorKind::AlreadyExists,
                resource: ResourceKind::Child,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::InUse { kind, id } => ReplyError {
                kind: ReplyErrorKind::Conflict,
                resource: kind,
                source: desc.to_string(),
                extra: format!("id: {}", id),
            },
            SvcError::AlreadyExists { kind, id } => ReplyError {
                kind: ReplyErrorKind::AlreadyExists,
                resource: kind,
                source: desc.to_string(),
                extra: format!("id: {}", id),
            },
            SvcError::NotReady { ref kind, .. } => ReplyError {
                kind: ReplyErrorKind::Unavailable,
                resource: kind.clone(),
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::Conflict { .. } => ReplyError {
                kind: ReplyErrorKind::Conflict,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::Deleting { .. } => ReplyError {
                kind: ReplyErrorKind::Deleting,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::ReCreateMismatch { id: _, ref kind } => ReplyError {
                kind: ReplyErrorKind::Conflict,
                resource: kind.clone(),
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::BusGetNode { source, .. } => source,
            SvcError::BusGetNodes { source } => source,
            SvcError::GrpcRequestError {
                source,
                request,
                resource,
            } => grpc_to_reply_error(SvcError::GrpcRequestError {
                source,
                request,
                resource,
            }),

            SvcError::InvalidArguments { .. } => ReplyError {
                kind: ReplyErrorKind::InvalidArgument,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },

            SvcError::NodeNotOnline { .. } => ReplyError {
                kind: ReplyErrorKind::FailedPrecondition,
                resource: ResourceKind::Node,
                source: desc.to_string(),
                extra: error.full_string(),
            },

            SvcError::NoNodes { .. } => ReplyError {
                kind: ReplyErrorKind::FailedPrecondition,
                resource: ResourceKind::Node,
                source: desc.to_string(),
                extra: error.full_string(),
            },

            SvcError::GrpcConnectTimeout { .. } => ReplyError {
                kind: ReplyErrorKind::Timeout,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },

            SvcError::GrpcConnectUri { .. } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },

            SvcError::GrpcConnect { source } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: source.to_string(),
            },

            SvcError::NotEnoughResources { .. } => ReplyError {
                kind: ReplyErrorKind::ResourceExhausted,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::JsonRpcDeserialise { .. } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::JsonGrpc,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::Store { .. } => ReplyError {
                kind: ReplyErrorKind::FailedPersist,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::JsonRpc { .. } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::JsonGrpc,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::NodeNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Node,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::PoolNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Pool,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::ReplicaNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Replica,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::NexusNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Nexus,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::NotFound { ref kind, .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: kind.clone(),
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::PendingCreation { ref kind, .. } => ReplyError {
                kind: ReplyErrorKind::FailedPrecondition,
                resource: kind.clone(),
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::VolumeNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Volume,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::VolumeNotPublished { .. } => ReplyError {
                kind: ReplyErrorKind::NotPublished,
                resource: ResourceKind::Volume,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::VolumeAlreadyPublished { .. } => ReplyError {
                kind: ReplyErrorKind::AlreadyPublished,
                resource: ResourceKind::Volume,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::WatchResourceNotFound { kind } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: kind,
                source: desc.to_string(),
                extra: error_str,
            },
            SvcError::WatchNotFound { .. } => ReplyError {
                kind: ReplyErrorKind::NotFound,
                resource: ResourceKind::Watch,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::WatchAlreadyExists { .. } => ReplyError {
                kind: ReplyErrorKind::AlreadyExists,
                resource: ResourceKind::Watch,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::InvalidFilter { .. } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::Internal { .. } => ReplyError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
            SvcError::MBusError { source } => source.into(),
            SvcError::MultipleNexuses { .. } => ReplyError {
                kind: ReplyErrorKind::InvalidArgument,
                resource: ResourceKind::Unknown,
                source: desc.to_string(),
                extra: error.full_string(),
            },
        }
    }
}

fn grpc_to_reply_error(error: SvcError) -> ReplyError {
    match error {
        SvcError::GrpcRequestError {
            source,
            request,
            resource,
        } => {
            let kind = match source.code() {
                Code::Ok => ReplyErrorKind::Internal,
                Code::Cancelled => ReplyErrorKind::Internal,
                Code::Unknown => ReplyErrorKind::Internal,
                Code::InvalidArgument => ReplyErrorKind::InvalidArgument,
                Code::DeadlineExceeded => ReplyErrorKind::DeadlineExceeded,
                Code::NotFound => ReplyErrorKind::NotFound,
                Code::AlreadyExists => ReplyErrorKind::AlreadyExists,
                Code::PermissionDenied => ReplyErrorKind::PermissionDenied,
                Code::ResourceExhausted => ReplyErrorKind::ResourceExhausted,
                Code::FailedPrecondition => ReplyErrorKind::FailedPrecondition,
                Code::Aborted => ReplyErrorKind::Aborted,
                Code::OutOfRange => ReplyErrorKind::OutOfRange,
                Code::Unimplemented => ReplyErrorKind::Unimplemented,
                Code::Internal => ReplyErrorKind::Internal,
                Code::Unavailable => ReplyErrorKind::Unavailable,
                Code::DataLoss => ReplyErrorKind::Internal,
                Code::Unauthenticated => ReplyErrorKind::Unauthenticated,
                Code::__NonExhaustive => ReplyErrorKind::Internal,
            };
            let extra = format!("{}::{}", request, source.to_string());
            ReplyError {
                kind,
                resource,
                source: "SvcError::GrpcRequestError".to_string(),
                extra,
            }
        }
        _ => unreachable!("Expected a GrpcRequestError!"),
    }
}

/// Not enough resources available
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum NotEnough {
    #[snafu(display("Not enough suitable pools available, {}/{}", have, need))]
    OfPools { have: u64, need: u64 },
    #[snafu(display("Not enough replicas available, {}/{}", have, need))]
    OfReplicas { have: u64, need: u64 },
    #[snafu(display("Not enough nexuses available, {}/{}", have, need))]
    OfNexuses { have: u64, need: u64 },
}
