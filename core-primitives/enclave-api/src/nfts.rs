use crate::{error::Error, Enclave, EnclaveResult};
use frame_support::ensure;
use itp_enclave_api_ffi as ffi;
use sgx_types::sgx_status_t;
use sp_runtime::AccountId32;

pub trait NFTs: Send + Sync + 'static {
	fn store_nft_data(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()>;
	fn update_nft_data(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()>;
}

impl NFTs for Enclave {
	fn store_nft_data(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let p_owner_id = AsRef::<[u8]>::as_ref(&owner_id).as_ptr();

		let res = unsafe { ffi::store_nft_data(self.eid, &mut retval, nft_id, p_owner_id) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}

	fn update_nft_data(&self, nft_id: u32, new_owner_id: AccountId32) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let p_owner_id = AsRef::<[u8]>::as_ref(&new_owner_id).as_ptr();

		let res = unsafe { ffi::update_nft_data(self.eid, &mut retval, nft_id, p_owner_id) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}
}
