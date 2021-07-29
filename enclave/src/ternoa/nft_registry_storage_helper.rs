/*
    Copyright 2019 Supercomputing Systems AG
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
use codec::{Decode, Encode};
use log::*;
use my_node_primitives::{BlockNumber, NFTId};
use std::collections::HashMap;
use std::fs;
use std::vec::Vec;

use super::nft_registry::{Error, NFTData, NFTRegistry, Result};

use crate::io as SgxIo;

/// helper struct to encode / decode hashmap
/// and finally store it in SgxFs
#[derive(Debug, Encode, Decode)]
pub struct NFTRegistryStorageHelper {
    registry: Vec<(NFTId, NFTData)>,
    block_number: BlockNumber,
}

impl NFTRegistryStorageHelper {
    fn create_from_registry(hashmap_registry: &NFTRegistry) -> Self {
        let vec_registry: Vec<(NFTId, NFTData)> =
            hashmap_registry.registry.clone().into_iter().collect();
        NFTRegistryStorageHelper {
            block_number: hashmap_registry.block_number,
            registry: vec_registry,
        }
    }

    fn recover_registry(&self) -> NFTRegistry {
        let mut recovered_map: HashMap<NFTId, NFTData> = HashMap::new();
        let mut ids: Vec<NFTId> = Vec::new();
        for data_point in self.registry.clone() {
            recovered_map.insert(data_point.0, data_point.1);
            ids.push(data_point.0);
        }
        NFTRegistry {
            block_number: self.block_number,
            registry: recovered_map,
            nft_ids: ids,
        }
    }

    /// save NFT Registry into SgxFs
    pub fn seal(path: &str, hashmap_registry: &NFTRegistry) -> Result<()> {
        debug!("backup registry state");
        if fs::copy(path, format!("{}.1", path)).is_err() {
            warn!("could not backup previous registry state");
        };
        debug!(
            "Seal Nft Registry State. Current state: {:?}",
            hashmap_registry
        );
        let helper = NFTRegistryStorageHelper::create_from_registry(hashmap_registry);
        match SgxIo::seal(helper.encode().as_slice(), path) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::SgxIoSealError(e)),
        }
    }

    /// load NFT Registry from SgxFs
    pub fn unseal(path: &str) -> Result<NFTRegistry> {
        let encoded = SgxIo::unseal(path).map_err(Error::SgxIoUnsealError)?;
        let registry_codec = NFTRegistryStorageHelper::decode(&mut encoded.as_slice())
            .map_err(|_| Error::DecodeError)?;
        Ok(registry_codec.recover_registry())
    }
}
pub mod test {
    use super::*;

    use fs::File;
    use my_node_primitives::nfts::NFTDetails;
    use my_node_primitives::AccountId;

    pub fn test_recover_registry() {
        //given
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let details_two = NFTDetails::new(vec![10, 10], 100, true);
        let owner = dummy_account();
        let nft_data_one = NFTData::new(owner.clone(), details, false, true);
        let nft_data_two = NFTData::new(owner, details_two, false, false);
        let mut vec_map: Vec<(NFTId, NFTData)> = Vec::new();
        let pair_one = (3, nft_data_one);
        let pair_two = (10, nft_data_two);
        vec_map.push(pair_one.clone());
        vec_map.push(pair_two.clone());
        let helper = NFTRegistryStorageHelper {
            registry: vec_map,
            block_number: 13,
        };

        // when
        let registry = NFTRegistryStorageHelper::recover_registry(&helper);

        // then
        assert_eq!(pair_one.1, *registry.registry.get(&pair_one.0).unwrap());
        assert_eq!(pair_two.1, *registry.registry.get(&pair_two.0).unwrap());
        assert_eq!(registry.block_number, helper.block_number);
        assert_eq!(registry.nft_ids.len(), 2);
    }

    pub fn test_create_from_registry() {
        //given
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let details_two = NFTDetails::new(vec![10, 10], 100, true);
        let owner = dummy_account();
        let nft_data_one = NFTData::new(owner.clone(), details, false, true);
        let nft_data_two = NFTData::new(owner, details_two, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(1, nft_data_one);
        hash_map.insert(2, nft_data_two);
        let registry = NFTRegistry::new(100, hash_map.clone(), vec![10, 2, 10, 4]);

        // when
        let helper = NFTRegistryStorageHelper::create_from_registry(&registry);

        // then
        let retrieved_key_one = helper.registry[0].0;
        let retrieved_key_two = helper.registry[1].0;

        assert_eq!(
            helper.registry[0].1,
            *hash_map.get(&retrieved_key_one).unwrap()
        );
        assert_eq!(
            helper.registry[1].1,
            *hash_map.get(&retrieved_key_two).unwrap()
        );
        assert_eq!(registry.block_number, helper.block_number);
    }

    pub fn test_recover_from_create_from_registry() {
        //given
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let details_two = NFTDetails::new(vec![10, 10], 100, true);
        let owner = dummy_account();
        let nft_data_one = NFTData::new(owner.clone(), details, false, true);
        let nft_data_two = NFTData::new(owner, details_two, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(1, nft_data_one);
        hash_map.insert(2, nft_data_two);
        let registry = NFTRegistry::new(100, hash_map, vec![10, 2, 10, 4]);
        let helper = NFTRegistryStorageHelper::create_from_registry(&registry);

        // when
        let recovered_registry = NFTRegistryStorageHelper::recover_registry(&helper);

        // then
        assert_eq!(registry, recovered_registry);
    }

    pub fn test_seal_creates_file() {
        //given
        let path = "hello_sealed_file";
        // when
        NFTRegistryStorageHelper::seal(path, &NFTRegistry::default()).unwrap();

        // then
        assert!(File::open(path).is_ok());

        // clean up
        fs::remove_file(path).unwrap();
        fs::remove_file(format!("{}.1", path)).unwrap();
    }

    pub fn test_seal_creates_backup_file() {
        //given
        let path = "hello_sealed_backup_file";
        let backup_path = "hello_sealed_backup_file.1";
        // when
        NFTRegistryStorageHelper::seal(path, &NFTRegistry::default()).unwrap();

        // then
        let file = fs::read(path).unwrap();
        let backup = fs::read(backup_path).unwrap();
        assert_eq!(file, backup);

        // clean up
        fs::remove_file(path).unwrap();
        fs::remove_file(backup_path).unwrap();
    }

    pub fn test_unseal_works() {
        // given
        let path = "hello_unseal";
        let registry = NFTRegistry::new(3, HashMap::default(), vec![10, 20]);
        NFTRegistryStorageHelper::seal(path, &registry).unwrap();

        // when
        let unsealed = NFTRegistryStorageHelper::unseal(path).unwrap();

        // then
        assert_eq!(unsealed, registry);

        // clean up
        fs::remove_file(path).unwrap();
        fs::remove_file(format!("{}.1", path)).unwrap();
    }

    fn dummy_account() -> AccountId {
        AccountId::from([
            212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133,
            88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 163, 127,
        ])
    }
}
