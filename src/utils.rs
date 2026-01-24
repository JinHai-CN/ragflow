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

/// 支持多种算法的check_password_hash
pub fn check_password_hash(pwhash: &str, password: &str) -> bool {
    // 分割哈希字符串
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

/// 内部scrypt检查实现
fn check_scrypt_hash_internal(pwhash: &str, password: &str) -> Result<bool> {
    let parts: Vec<&str> = pwhash.split('$').collect();

    let params_str = parts[0];
    let salt_b64 = parts[1];
    let stored_hash_b64 = parts[2];

    // 解析参数
    let param_parts: Vec<&str> = params_str.split(':').collect();
    if param_parts.len() != 4 {
        return Err(anyhow!("Invalid scrypt parameters"));
    }

    let n: u32 = param_parts[1].parse()?;
    let log_n: u8 = n.ilog2() as u8;
    let r: u32 = param_parts[2].parse()?;
    let p: u32 = param_parts[3].parse()?;

    // 解码
    let salt = general_purpose::STANDARD.decode(salt_b64)?;
    let stored_hash = general_purpose::STANDARD.decode(stored_hash_b64)?;

    // 重新计算
    let params = Params::new(log_n, r, p, stored_hash.len())?;
    let mut computed_hash = vec![0u8; stored_hash.len()];

    scrypt(
        password.as_bytes(),
        &salt,
        &params,
        &mut computed_hash
    )?;

    // 安全比较
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
    // PBKDF2实现略
    Ok(false)
}

fn check_salted_hash_internal(_pwhash: &str, _password: &str) -> Result<bool> {
    // 加盐哈希实现略
    Ok(false)
}

/// Placeholder utility function.
pub fn placeholder() -> &'static str {
    "utils placeholder"
}