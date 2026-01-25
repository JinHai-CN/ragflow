/*
 * Copyright (c) 2026 Infiniflow, Inc. All rights reserved.
 *
 * PROPRIETARY AND CONFIDENTIAL
 *
 * This software is the proprietary property of Infiniflow, Inc. and is
 * protected by copyright and other intellectual property laws.
 *
 * RESTRICTIONS:
 * - You may NOT redistribute, sell, lease, or sublicense this software.
 * - You may NOT use this software to provide commercial hosting services
 *   (SaaS/PaaS) without explicit written permission.
 * - You may NOT reverse-engineer, decompile, or disassemble this software.
 * - You may NOT remove or alter this copyright notice.
 *
 * VIOLATION:
 * Any unauthorized use, reproduction, or distribution of this software
 * may result in severe civil and criminal penalties, and will be prosecuted
 * to the maximum extent possible under applicable law.
 *
 * THIS SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED.
 */

//! Utility functions for the RAGFlow API server.

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use openssl::pkey::PKey;
use openssl::pkcs5::scrypt as openssl_scrypt;
use scrypt::{
    scrypt,
    Params,
};

/// Decrypt a password encrypted with RSA PKCS1 v1.5 using the private key.
/// The input is expected to be base64-encoded ciphertext.
/// Returns the decrypted plaintext string.
pub fn decrypt_password(encrypted_password: &str, passphrase: &str) -> Result<String, anyhow::Error> {
    // Load private key from conf/private.pem
    let project_base = std::env::current_dir()?;
    let pem_file_path = project_base.join("conf").join("private.pem");
    if !pem_file_path.exists() {
        return Err(anyhow::anyhow!(
            "Private key file not found at {}",
            pem_file_path.display()
        ));
    }

    let pem_content = std::fs::read_to_string(pem_file_path)
        .map_err(|e| anyhow!("Failed to read private key file: {}", e))?;

    // 2. Load private key using passphrase (this is the key step!)
    let private_key = match PKey::private_key_from_pem_passphrase(
        pem_content.as_bytes(),
        passphrase.as_bytes()
    ) {
        Ok(key) => key,
        Err(_) => return Err(anyhow!("Fail to decrypt password!")),
    };

    // 3. Get RSA object
    let rsa = private_key.rsa()
        .map_err(|e| anyhow!("Invalid RSA key: {}", e))?;

    // 4. Base64 decode
    let encrypted_data = general_purpose::STANDARD.decode(encrypted_password)
        .map_err(|e| anyhow!("Invalid base64 data: {}", e))?;

    // 5. RSA decryption (PKCS#1 v1.5)
    let mut decrypted = vec![0; rsa.size() as usize];
    match rsa.private_decrypt(
        &encrypted_data,
        &mut decrypted,
        openssl::rsa::Padding::PKCS1
    ) {
        Ok(len) => {
            decrypted.truncate(len);
            // 6. Convert to UTF-8 string
            String::from_utf8(decrypted)
                .map_err(|e| anyhow!("Decrypted data is not valid UTF-8: {}", e))
        }
        Err(_) => Err(anyhow!("Fail to decrypt password!")),
    }
}

/// check_password_hash supporting multiple algorithms
pub fn check_password_hash(pwhash: &str, password: &str) -> bool {
    // Split hash string
    let parts: Vec<&str> = pwhash.split('$').collect();
    if parts.len() < 3 {
        return false;
    }

    let method = parts[0];
    if method.starts_with("scrypt") {
        check_scrypt_hash_internal(pwhash, password).unwrap_or(false)
    } else {
        // Other algorithms
        false
    }
}

/// Internal scrypt check implementation
fn check_scrypt_hash_internal(pwhash: &str, password: &str) -> Result<bool> {
    let parts: Vec<&str> = pwhash.split('$').collect();

    let params_str = parts[0];
    let salt_b64 = parts[1];
    let stored_hash_b64 = parts[2];

    // Parse parameters
    let param_parts: Vec<&str> = params_str.split(':').collect();
    if param_parts.len() != 4 {
        return Err(anyhow!("Invalid scrypt parameters"));
    }

    let n: u32 = param_parts[1].parse()?;
    let r: u32 = param_parts[2].parse()?;
    let p: u32 = param_parts[3].parse()?;

    // Decode
    let salt = general_purpose::STANDARD.decode(salt_b64)?;
    let stored_hash = general_purpose::STANDARD.decode(stored_hash_b64)?;

    // Recalculate
    if stored_hash.len() < 10 {
        return Err(anyhow!("Invalid hash length"));
    }

    let total_len = stored_hash.len();
    
    // Calculate maxmem value same as werkzeug: 132 * n * r * p
    let maxmem = 132 * n as u64 * r as u64 * p as u64;
    
    // Use openssl::pkcs5::scrypt function, includes maxmem parameter
    let mut computed_hash = vec![0u8; total_len];
    
    openssl_scrypt(
        password.as_bytes(),
        &salt,
        n as u64,
        r as u64,
        p as u64,
        maxmem,
        &mut computed_hash
    )?;
    
    Ok(constant_time_compare(&computed_hash, &stored_hash))
}

fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

fn check_pbkdf2_hash_internal(_pwhash: &str, _password: &str) -> Result<bool> {
    // PBKDF2 implementation omitted
    Ok(false)
}

fn check_salted_hash_internal(_pwhash: &str, _password: &str) -> Result<bool> {
    // Salted hash implementation omitted
    Ok(false)
}

/// Placeholder utility function.
pub fn placeholder() -> &'static str {
    "utils placeholder"
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine};

    fn decode_scrypt_hash(pwhash: &str) -> (u8, u32, u32, Vec<u8>, Vec<u8>) {
        let parts: Vec<&str> = pwhash.split('$').collect();
        let params_str = parts[0];
        let salt_b64 = parts[1];
        let hash_b64 = parts[2];

        let param_parts: Vec<&str> = params_str.split(':').collect();
        let n: u32 = param_parts[1].parse().unwrap();
        let log_n: u8 = n.ilog2() as u8;
        let r: u32 = param_parts[2].parse().unwrap();
        let p: u32 = param_parts[3].parse().unwrap();

        let salt = general_purpose::STANDARD.decode(salt_b64).unwrap();
        let hash = general_purpose::STANDARD.decode(hash_b64).unwrap();
        (log_n, r, p, salt, hash)
    }

    #[test]
    fn test_scrypt_long_hash_wrong_password() {
        // Example hash with dk_len=96 (base64 decoded length)
        let pwhash = "scrypt:32768:8:1$Hs10WRCoINjDniMY$e5c0adc4564c62ff75b9b45dd5a4d078184c7a1e27f25af13a876e40406cd936966d913aa6dd038bf1b2800a9b8190605f75430353ad3e7d2afd0a659a06a28d";
        let password = "wrongpassword";
        // Should return Ok(false) because password doesn't match, but not error
        let result = check_scrypt_hash_internal(pwhash, password);
        assert!(matches!(result, Ok(false)));
    }

    #[test]
    fn test_scrypt_long_hash_correct_password() {
        // Hash generated by Python's werkzeug with password "testpassword123"
        // Updated with new werkzeug hash from generate_scrypt_hashes.py
        // let pwhash = "scrypt:32768:8:1$Vrx6vtc131Sbhmjc$f8861ae8088c7dbd48263fcf343c7570c265beb46117dd84d97c90e49662c9429cd8a109dc9686a7a085ee89d1e4a2f6d9b91a3beaa91e628e32fbccce523bf6";
        let pwhash = "scrypt:32768:8:1$epfMHVVrbNbcm9mJ$3a65d4587838e8686d140d21784c95452e016a2b1e232da598e101302cd22dbceb3a6dbe89311ecd4c7e37c60f4535fa917d861de9819a6fa5c6985c67965377";
        let password = "testpassword123";
        // Should return Ok(true) because password matches
        let result = check_scrypt_hash_internal(pwhash, password);
        assert!(matches!(result, Ok(true)));
    }

    #[test]
    fn debug_scrypt_hash() {
        let pwhash = "scrypt:32768:8:1$Vrx6vtc131Sbhmjc$f8861ae8088c7dbd48263fcf343c7570c265beb46117dd84d97c90e49662c9429cd8a109dc9686a7a085ee89d1e4a2f6d9b91a3beaa91e628e32fbccce523bf6";
        let password = "testpassword123";
        let (log_n, r, p, salt, stored_hash) = decode_scrypt_hash(pwhash);
        println!("log_n={}, r={}, p={}", log_n, r, p);
        println!("salt len={}, hash len={}", salt.len(), stored_hash.len());
        println!("stored hash first 32 bytes: {:02x?}", &stored_hash[..32.min(stored_hash.len())]);

        // Use Params::new_with_output_len with valid length 64
        let params = Params::new_with_output_len(log_n, r, p, 64).unwrap();
        let mut computed_hash = vec![0u8; stored_hash.len()];
        scrypt(password.as_bytes(), &salt, &params, &mut computed_hash).unwrap();
        println!("computed hash first 32 bytes: {:02x?}", &computed_hash[..32.min(computed_hash.len())]);
        println!("hash match? {}", constant_time_compare(&computed_hash, &stored_hash));
    }

    #[test]
    fn test_scrypt_python_generated_hash_64() {
        // 64-byte hash generated by Python, using n=1024 to avoid memory constraints
        let pwhash = "scrypt:1024:8:1$Zml4ZWRfc2FsdF8xMjM0NTY=$fqX8whFZ0EDlWZVBt0xqlwGpRi43ldrE9P3zI/RjMBA7dhK/D6IEtK4ERNh90iXJIPdaMS8Ff9nWHDiBmVR8Kw==";
        let password = "testpassword123";
        
        // Verify with Rust
        let result = check_scrypt_hash_internal(pwhash, password);
        assert!(matches!(result, Ok(true)), "64-byte hash generated by Python should be verified by Rust");
    }
    
    #[test]
    fn test_scrypt_python_generated_hash_96() {
        // 96-byte hash generated by Python, using n=1024 to avoid memory constraints
        let pwhash = "scrypt:1024:8:1$Zml4ZWRfc2FsdF8xMjM0NTY=$fqX8whFZ0EDlWZVBt0xqlwGpRi43ldrE9P3zI/RjMBA7dhK/D6IEtK4ERNh90iXJIPdaMS8Ff9nWHDiBmVR8K6SLhG5asz1Q0WeQ5gCjJOlW7sypFbJUiYF4qiAlBd5z";
        let password = "testpassword123";

        // Verify with Rust
        let result = check_scrypt_hash_internal(pwhash, password);
        assert!(matches!(result, Ok(true)), "96-byte hash generated by Python should be verified by Rust");
    }


}