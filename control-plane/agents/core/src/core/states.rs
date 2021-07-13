use common_lib::types::v0::{
    message_bus::{Nexus, NexusId, Pool, PoolId, Replica, ReplicaId},
    store::{nexus::NexusState, pool::PoolState, replica::ReplicaState},
};
use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use super::resource_map::ResourceMap;
use parking_lot::Mutex;

/// Locked Resource States
#[derive(Default, Clone, Debug)]
pub(crate) struct ResourceStatesLocked(Arc<RwLock<ResourceStates>>);

impl ResourceStatesLocked {
    pub(crate) fn new() -> Self {
        ResourceStatesLocked::default()
    }
}

impl Deref for ResourceStatesLocked {
    type Target = Arc<RwLock<ResourceStates>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Resource States
#[derive(Default, Debug)]
pub(crate) struct ResourceStates {
    /// Todo: Add runtime state information for nodes.
    nexuses: ResourceMap<NexusId, NexusState>,
    pools: ResourceMap<PoolId, PoolState>,
    replicas: ResourceMap<ReplicaId, ReplicaState>,
}

impl ResourceStates {
    /// Update the various resource states.
    pub(crate) fn update(&mut self, pools: Vec<Pool>, replicas: Vec<Replica>, nexuses: Vec<Nexus>) {
        self.replicas.update(Self::states_from_mbus(replicas));
        self.pools.update(Self::states_from_mbus(pools));
        self.nexuses.update(Self::states_from_mbus(nexuses));
    }

    /// Returns a vector of nexus states.
    pub(crate) fn get_nexus_states(&self) -> Vec<NexusState> {
        Self::cloned_inner_states(self.nexuses.to_vec())
    }

    /// Returns a vector of pool states.
    pub(crate) fn get_pool_states(&self) -> Vec<PoolState> {
        Self::cloned_inner_states(self.pools.to_vec())
    }

    /// Returns a vector of replica states.
    pub(crate) fn get_replica_states(&self) -> Vec<ReplicaState> {
        Self::cloned_inner_states(self.replicas.to_vec())
    }

    /// Converts a vector of mbus types into a vector of resource state types.
    fn states_from_mbus<M, S>(mbus_resource: Vec<M>) -> Vec<S>
    where
        S: From<M>,
    {
        mbus_resource.into_iter().map(S::from).collect()
    }

    /// Takes a vector of resources protected by an 'Arc' and 'Mutex' and returns a vector of
    /// unprotected resources.
    fn cloned_inner_states<S>(locked_states: Vec<Arc<Mutex<S>>>) -> Vec<S>
    where
        S: Clone,
    {
        locked_states.iter().map(|s| s.lock().clone()).collect()
    }
}
