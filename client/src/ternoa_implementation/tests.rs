use crate::ternoa_implementation::cipher::{decrypt, encrypt, recover_or_generate_encryption_key};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

const KEYFILE_EXT: &str = "aes256";
const CIPHERTEXT_EXT: &str = "ciphertext";
const DECRYPTED_EXT: &str = "decrypted";
const TEST_TEXT: &str = "I'm nobody! Who are you?\nAre you nobody, too?";

fn plaintext_input(dir_path: &Path) -> std::io::Result<PathBuf> {
    let file_path = dir_path.join("input.txt");
    let mut test_file = File::create(file_path.clone()).unwrap();
    write!(test_file, "{}", TEST_TEXT).unwrap();
    Ok(file_path)
}

fn decrypted_text(decrypted_file_path: &str) -> String {
    let decrypted_read_result = fs::read_to_string(decrypted_file_path);
    let text = decrypted_read_result.ok().unwrap();
    text
}

#[test]
fn verify_recover_encryption_key() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //When
    let key = recover_or_generate_encryption_key(&key_path).unwrap();
    let recovered_key = recover_or_generate_encryption_key(&key_path).unwrap();
    //Then
    assert!(key_path.exists());
    assert_eq!(recovered_key.0, key.0);
    assert_eq!(recovered_key.1, key.1);
    //Clean
    dir.close();
}

#[test]
fn verify_generate_encryption_key() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //When
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    //Then
    assert_eq!(aes.0.len(), 32);
    assert_eq!(aes.1.len(), 16);
    assert!(key_path.exists());

    dir.close();
}

#[test]
fn verify_encrypt_generate_key_when_no_key_passed() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("input.".to_owned() + KEYFILE_EXT);
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();

    //When
    encrypt(inputfile_path.to_str().unwrap(), None).unwrap();

    // Then
    assert!(key_path.exists()); //A key has been generated
    dir.close();
}

#[test]
fn verify_encrypt_without_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();

    //When
    let result = encrypt(inputfile_path.to_str().unwrap(), None);

    // Then
    assert!(result.is_ok());
    assert!(ciphertext_path.exists());
    dir.close();
}

#[test]
fn verify_decrypt_without_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();
    encrypt(inputfile_path.to_str().unwrap(), None).unwrap();

    //When
    let result = decrypt(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_ok());
    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_encrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    //generate key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();

    //When
    let result = encrypt(test_file_path.to_str().unwrap(), Some(aes.clone()));
    //Then
    assert!(result.is_ok());
    assert!(ciphertext_path.exists());

    dir.close();
}

#[test]
fn verify_decrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    //generate key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    //encrypt
    encrypt(test_file_path.to_str().unwrap(), Some(aes.clone())).unwrap();

    //When
    let result = decrypt(ciphertext_path.to_str().unwrap(), Some(aes));

    //Then
    assert!(result.is_ok());
    assert!(decrypted_path.exists());
    //Check content
    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_decrypt_fails_without_keyfile() {
    //Given
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("tmp");
    let mut test_file = File::create(file_path.clone()).unwrap();
    writeln!(test_file, "blablabla").unwrap();

    let ciphertext_path = file_path.join("tmp.").join(CIPHERTEXT_EXT);

    encrypt(file_path.to_str().unwrap(), None).unwrap();

    let key_path = file_path.join("tmp.").join(KEYFILE_EXT);
    fs::remove_file(key_path);

    //When
    let result = decrypt(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_err());
    dir.close();
}

#[test]
fn verify_decrypt_fails_with_diff_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    let other_key_path = dir.path().join("keyfile2.".to_owned() + KEYFILE_EXT);
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();

    //encrypt with one key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    encrypt(test_file_path.to_str().unwrap(), Some(aes)).unwrap();

    //When
    //decrypt with another key
    let other_aes = recover_or_generate_encryption_key(&other_key_path).unwrap();
    decrypt(ciphertext_path.to_str().unwrap(), Some(other_aes)).unwrap();

    //Then
    assert!(decrypted_path.exists());
    //Decrypted file isn't valid
    let decrypted_read_result = fs::read_to_string(decrypted_path.to_str().unwrap());
    assert!(decrypted_read_result.is_err());

    dir.close();
}

#[test]
fn verify_decrypt_without_passing_key_fails_when_encrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();
    //encrypt with one key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    encrypt(inputfile_path.to_str().unwrap(), Some(aes)).unwrap();

    //When
    let result = decrypt(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_err());
    dir.close();
}
