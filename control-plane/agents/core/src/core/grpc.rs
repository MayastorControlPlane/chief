use crate::node::service::NodeCommsTimeout;
use common::errors::{GrpcConnect, GrpcConnectUri, SvcError};
use common_lib::{rpc::mayastor_client::MayastorClient, types::v0::message_bus::mbus::NodeId};
use snafu::ResultExt;
use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
    sync::Arc,
};
use tonic::transport::Channel;

/// Context with a gRPC client and a lock to serialize mutating gRPC calls
#[derive(Clone)]
pub(crate) struct GrpcContext {
    /// gRPC CRUD lock
    lock: Arc<tokio::sync::Mutex<()>>,
    /// node identifier
    node: NodeId,
    /// gRPC URI endpoint
    endpoint: tonic::transport::Endpoint,
    /// gRPC connect and request timeouts
    comms_timeouts: NodeCommsTimeout,
}

impl GrpcContext {
    pub(crate) fn new(
        lock: Arc<tokio::sync::Mutex<()>>,
        node: &NodeId,
        endpoint: &str,
        comms_timeouts: &NodeCommsTimeout,
    ) -> Result<Self, SvcError> {
        let uri = format!("http://{}", endpoint);
        let uri = http::uri::Uri::from_str(&uri).context(GrpcConnectUri {
            node_id: node.to_string(),
            uri: uri.clone(),
        })?;
        let endpoint = tonic::transport::Endpoint::from(uri).timeout(comms_timeouts.request());

        Ok(Self {
            node: node.clone(),
            lock,
            endpoint,
            comms_timeouts: comms_timeouts.clone(),
        })
    }
    pub(crate) async fn lock(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.lock.clone().lock_owned().await
    }
    pub(crate) async fn connect(&self) -> Result<GrpcClient, SvcError> {
        GrpcClient::new(self).await
    }
    pub(crate) async fn connect_locked(&self) -> Result<GrpcClientLocked, SvcError> {
        GrpcClientLocked::new(self).await
    }
}

/// Wrapper over all gRPC Clients types
#[derive(Clone)]
pub(crate) struct GrpcClient {
    context: GrpcContext,
    /// gRPC Mayastor Client
    pub(crate) client: MayaClient,
}
pub(crate) type MayaClient = MayastorClient<Channel>;
impl GrpcClient {
    pub(crate) async fn new(context: &GrpcContext) -> Result<Self, SvcError> {
        let client = match tokio::time::timeout(
            context.comms_timeouts.connect(),
            MayaClient::connect(context.endpoint.clone()),
        )
        .await
        {
            Err(_) => Err(SvcError::GrpcConnectTimeout {
                node_id: context.node.to_string(),
                endpoint: format!("{:?}", context.endpoint),
                timeout: context.comms_timeouts.connect(),
            }),
            Ok(client) => Ok(client.context(GrpcConnect)?),
        }?;

        Ok(Self {
            context: context.clone(),
            client,
        })
    }
}

/// Wrapper over all gRPC Clients types with implicit locking for serialization
pub(crate) struct GrpcClientLocked {
    /// gRPC auto CRUD guard lock
    _lock: tokio::sync::OwnedMutexGuard<()>,
    client: GrpcClient,
}
impl GrpcClientLocked {
    pub(crate) async fn new(context: &GrpcContext) -> Result<Self, SvcError> {
        let client = GrpcClient::new(context).await?;

        Ok(Self {
            _lock: context.lock().await,
            client,
        })
    }
}

impl Deref for GrpcClientLocked {
    type Target = GrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
impl DerefMut for GrpcClientLocked {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
