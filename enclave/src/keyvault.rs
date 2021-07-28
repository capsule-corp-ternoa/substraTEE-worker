use crate::io;
use crate::keyvault::Error::KeyvaultError;
use core::convert::TryFrom;
use derive_more::{Display, From};
use log::*;
use my_node_primitives::NFTId;
use sharks::Share;
use sp_core::crypto::AccountId32;
use std::fs;
use std::path::PathBuf;
use std::prelude::v1::*;
use std::vec::Vec;

#[derive(Debug, Display, From)]
pub enum Error {
    /// Wrapping of io error to keyvault error
    IoError(std::io::Error),
    ///Wrapping of Keyvault error
    KeyvaultError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub const STORAGE_PATH: &str = "keyshare";

pub trait NftAccess {
    fn is_authorized(&self, owner: AccountId32, nft_id: NFTId) -> bool;
}

pub struct KeyvaultStorage {
    path: PathBuf,
}

impl KeyvaultStorage {
    fn new() -> Self {
        KeyvaultStorage {
            path: PathBuf::from(STORAGE_PATH),
        }
    }
    pub fn set_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    ///Store share on disk in sealed file
    pub fn provision(&self, owner: AccountId32, nft_id: NFTId, share: Share) -> Result<()> {
        if !self.is_authorized(owner.clone(), nft_id) {
            return Err(KeyvaultError(format!(
                "Provision of {} is non authorized for this owner: {:?}.",
                nft_id, owner
            )));
        }
        self.seal(nft_id, share)?;
        Ok(())
    }

    ///Check if share for NFTId is in store
    pub fn check(&self, owner: AccountId32, nft_id: NFTId) -> bool {
        //TODO Authorization owner
        if !self.is_authorized(owner, nft_id) {
            return false;
        }
        let file_name = self.nft_sealed_file_path(nft_id);
        file_name.is_file()
    }

    ///Get the share from store for this NFTId
    pub fn get(&self, owner: AccountId32, nft_id: NFTId) -> Option<Share> {
        //TODO Authorization owner
        if !&self.is_authorized(owner.clone(), nft_id) {
            error!(
                "get of {} is non authorized for this owner: {:?}.",
                nft_id, owner
            );
            return None;
        }
        match self.unseal(nft_id) {
            Ok(share) => Some(share),
            Err(e) => {
                error!("No share found for {:?} and NFT {}: {:?}", owner, nft_id, e);
                None
            }
        }
    }

    ///Filename should encode the NFTId
    fn nft_sealed_file_path(&self, nft_id: NFTId) -> PathBuf {
        let name = format!("{}_Nft", nft_id);
        let mut p = PathBuf::from(&self.path);
        p.push(name);
        p.set_extension("bin");
        p
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        if !&self.path.is_dir() {
            fs::create_dir_all(&self.path)?
        }
        Ok(())
    }

    ///Seal the share for NFTId
    fn seal(&self, nft_id: NFTId, share: Share) -> Result<()> {
        let filepath = self.nft_sealed_file_path(nft_id);
        if filepath.is_file() {
            warn!(
                "You will override an already existing sealed for {}!",
                nft_id
            );
        } else {
            //Directory will not be created by the seal method, so create it if it doesn't exist
            self.ensure_dir_exists()?;
        }

        match io::seal(Vec::from(&share).as_slice(), &filepath.to_string_lossy()) {
            Ok(_r) => Ok(()),
            Err(e) => {
                return Err(KeyvaultError(format!("Cannot seal {} : {:?}", nft_id, e)));
            }
        }
    }

    ///Unseal the share for NFTId
    fn unseal(&self, nft_id: NFTId) -> Result<Share> {
        let filepath = self.nft_sealed_file_path(nft_id);
        let share_bytes = match io::unseal(&filepath.to_string_lossy()) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(KeyvaultError(format!("Cannot unseal {} : {:?}", nft_id, e)));
            }
        };
        Share::try_from(share_bytes.as_slice())
            .map_err(|e| KeyvaultError(format!("Cannot unseal share'{}' : error {}", nft_id, e)))
    }
}

impl NftAccess for KeyvaultStorage {
    fn is_authorized(&self, _owner: AccountId32, _nft_id: NFTId) -> bool {
        //TODO Authorization owner
        true
    }
}

pub mod test {
    use super::*;

    //owner for test
    const ALICE_ENCODED: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];

    ///Tests
    pub fn create_keyvault_storage_works() {
        let storage = KeyvaultStorage::new();
        assert_eq!(storage.path, PathBuf::from(STORAGE_PATH));
    }

    pub fn set_filename_and_path_works() {
        let dir = PathBuf::from("test_set_path");

        let storage = KeyvaultStorage::new().set_path(dir.clone());

        assert_eq!(storage.path, dir);
    }

    pub fn test_ensure_dir_exists_creates_new_if_not_existing() {
        let dir = PathBuf::from("test_creates_dir");
        let storage = KeyvaultStorage::new().set_path(dir.clone());
        storage.ensure_dir_exists().unwrap();

        assert!(dir.is_dir());

        //clean up
        fs::remove_dir_all(dir).unwrap();
    }

    pub fn test_sealed_file_name_contains_nftid() {
        let dir = PathBuf::from("test_sealed_file_name");
        let storage = KeyvaultStorage::new().set_path(dir.clone());
        let file = storage.nft_sealed_file_path(197);
        let name = file.file_name().unwrap().to_str().unwrap();

        assert!(name.contains("197"));
    }

    pub fn test_seal_create_file() {
        let dir = PathBuf::from("test_seal_create_file");
        let share_bytes = Vec::from("hello");
        let share = Share::try_from(share_bytes.as_slice()).unwrap();
        let nft_id = 365;
        let storage = KeyvaultStorage::new().set_path(dir.clone());
        storage.seal(nft_id, share).unwrap();
        let file = PathBuf::from("test_seal_create_file/365_Nft.bin");

        assert!(file.is_file());

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    pub fn test_share_saved_in_sealed_file() {
        let dir = PathBuf::from("test_share_saved");
        let share_bytes = Vec::from("hello");
        let share = Share::try_from(share_bytes.as_slice()).unwrap();
        let nft_id = 365;
        let storage = KeyvaultStorage::new().set_path(dir.clone());
        storage.seal(nft_id, share).unwrap();

        let read_share = storage.unseal(nft_id).unwrap();
        let read_share_bytes = Vec::from(&read_share);
        for i in 1..5 {
            assert_eq!(share_bytes[i], read_share_bytes[i]);
        }

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    ///Can we override a seal file?
    pub fn test_seal_override_existing_sealed_file() {
        let dir = PathBuf::from("test_override_file");
        let share = Share::try_from("hello".as_bytes()).unwrap();
        let storage = KeyvaultStorage::new().set_path(dir.clone());

        let nft_id = 5870;
        storage.seal(nft_id, share).unwrap();

        let new_share_bytes = Vec::from("hello_world");
        let new_share = Share::try_from(new_share_bytes.as_slice()).unwrap();
        storage.seal(nft_id, new_share).unwrap();

        let read_share = storage.unseal(nft_id).unwrap();
        let read_share_bytes = Vec::from(&read_share);
        for i in 1..11 {
            assert_eq!(new_share_bytes[i], read_share_bytes[i]);
        }

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    pub fn test_unseal_fails_when_no_file_exists() {
        let dir = PathBuf::from("unseal_fails_no_file");
        let storage = KeyvaultStorage::new().set_path(dir.clone());
        let file = dir.join("365_Nft.bin");

        assert!(!file.is_file());

        assert!(storage.unseal(365).is_err());
    }

    pub fn test_provision_fails_when_no_nft_owner() {
        //TODO
    }

    pub fn test_check_is_false_when_no_nft_owner() {
        //TODO
    }

    pub fn test_get_fails_when_no_nft_owner() {
        //TODO
    }

    pub fn test_provision_store_share_in_sealed_file() {
        let nft_id = 5880;
        let author = AccountId32::from(ALICE_ENCODED);
        let share_bytes = Vec::from("new_test_share_5875");
        let share = Share::try_from(share_bytes.as_slice()).unwrap();
        let storage = KeyvaultStorage::new();
        storage.provision(author, nft_id, share).unwrap();

        let file_name = storage.nft_sealed_file_path(nft_id);
        assert!(file_name.is_file());

        let read_share = storage.unseal(nft_id).unwrap();
        let read_share_bytes = Vec::from(&read_share);
        for i in 1..18 {
            assert_eq!(share_bytes[i], read_share_bytes[i]);
        }

        //Clean-up
        fs::remove_file(file_name).unwrap();
    }

    pub fn test_check_is_true_when_sealed_file() {
        let nft_id = 5890;
        let storage = KeyvaultStorage::new();
        let file_name = storage.nft_sealed_file_path(nft_id);
        let author = AccountId32::from(ALICE_ENCODED);
        let new_share = Share::try_from("hello_world".as_bytes()).unwrap();

        storage.seal(nft_id, new_share).unwrap();

        assert!(storage.check(author, nft_id));

        //Clean-up
        fs::remove_file(file_name).unwrap();
    }

    pub fn test_check_is_false_when_no_sealed_file() {
        let nft_id = 6000;
        let author = AccountId32::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new();
        assert!(!storage.check(author, nft_id));
    }

    pub fn test_get_none_when_nft_not_in_store() {
        let nft_id = 6010;
        let author = AccountId32::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new();
        assert! {storage.get(author, nft_id).is_none()};
    }

    pub fn test_get_the_valid_stored_share() {
        let nft_id = 6020;
        let owner = AccountId32::from(ALICE_ENCODED);
        let share_bytes = Vec::from("new_test_share_6020");
        let share = Share::try_from(share_bytes.as_slice()).unwrap();
        let storage = KeyvaultStorage::new();
        storage.provision(owner.clone(), nft_id, share).unwrap();

        let read_share = storage.get(owner, nft_id).unwrap();
        let read_share_bytes = Vec::from(&read_share);
        for i in 1..18 {
            assert_eq!(share_bytes[i], read_share_bytes[i]);
        }

        //Clean-up
        let file_name = storage.nft_sealed_file_path(nft_id);
        fs::remove_file(file_name).unwrap();
    }
}
