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

/// Placeholder utility function.
pub fn placeholder() -> &'static str {
    "utils placeholder"
}