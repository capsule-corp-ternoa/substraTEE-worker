/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use crate::{
	node_api_factory::CreateNodeApi,
	ocall_bridge::{
		bridge_api::{
			GetOCallBridgeComponents, IpfsBridge, RemoteAttestationBridge, WorkerOnChainBridge,
		},
		ipfs_ocall::IpfsOCall,
		remote_attestation_ocall::RemoteAttestationOCall,
		worker_on_chain_ocall::WorkerOnChainOCall,
	},
};
use itp_enclave_api::remote_attestation::RemoteAttestationCallBacks;
use std::sync::Arc;

/// Concrete implementation, should be moved out of the OCall Bridge, into the worker
/// since the OCall bridge itself should not know any concrete types to ensure
/// our dependency graph is worker -> ocall bridge
pub struct OCallBridgeComponentFactory<NodeApi, EnclaveApi> {
	node_api_factory: Arc<NodeApi>,
	enclave_api: Arc<EnclaveApi>,
}

impl<NodeApi, EnclaveApi> OCallBridgeComponentFactory<NodeApi, EnclaveApi> {
	pub fn new(node_api_factory: Arc<NodeApi>, enclave_api: Arc<EnclaveApi>) -> Self {
		OCallBridgeComponentFactory { node_api_factory, enclave_api }
	}
}

impl<NodeApi, EnclaveApi> GetOCallBridgeComponents
	for OCallBridgeComponentFactory<NodeApi, EnclaveApi>
where
	NodeApi: CreateNodeApi + 'static,
	EnclaveApi: RemoteAttestationCallBacks + 'static,
{
	fn get_ra_api(&self) -> Arc<dyn RemoteAttestationBridge> {
		Arc::new(RemoteAttestationOCall::new(self.enclave_api.clone()))
	}

	fn get_oc_api(&self) -> Arc<dyn WorkerOnChainBridge> {
		Arc::new(WorkerOnChainOCall::new(self.node_api_factory.clone()))
	}

	fn get_ipfs_api(&self) -> Arc<dyn IpfsBridge> {
		Arc::new(IpfsOCall {})
	}
}
