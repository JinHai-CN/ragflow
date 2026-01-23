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

use base64::prelude::*;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::Pkcs1v15Encrypt;
use rsa::RsaPrivateKey;
use std::fs;

/// Decrypt a password encrypted with RSA PKCS1 v1.5 using the private key.
/// The input is expected to be base64-encoded ciphertext.
/// Returns the decrypted plaintext string.
pub fn decrypt_password(encrypted_password: &str) -> Result<String, anyhow::Error> {
    // Load private key from conf/private.pem
    let project_base = std::env::current_dir()?;
    let private_key_path = project_base.join("conf").join("private.pem");
    
    if !private_key_path.exists() {
        return Err(anyhow::anyhow!(
            "Private key file not found at {}",
            private_key_path.display()
        ));
    }
    
    let pem_content = fs::read_to_string(&private_key_path)?;
    let private_key = RsaPrivateKey::from_pkcs1_pem(&pem_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
    
    // Decode base64 ciphertext
    let ciphertext = BASE64_STANDARD.decode(encrypted_password)?;
    
    // Decrypt using PKCS1v15 encryption padding scheme
    let decrypted_bytes = private_key
        .decrypt(Pkcs1v15Encrypt, &ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    
    // Convert to string
    let plaintext = String::from_utf8(decrypted_bytes)?;
    Ok(plaintext)
}

/// Placeholder utility function.
pub fn placeholder() -> &'static str {
    "utils placeholder"
}