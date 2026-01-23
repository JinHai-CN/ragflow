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
use std::time::Duration;
use actix_web::{get, web, Error, HttpResponse, Responder};
use serde_json::json;

/// System ping endpoint
// #[get("/v1/system/ping")]
// pub async fn ping() -> impl Responder {
//     HttpResponse::Ok().json(json!({
//         "code": 0,
//         "message": "success",
//         "data": "pong"
//     }))
// }

fn read_large_file(path: &str) -> std::io::Result<String> {
    // println!("read large: {}", path);
    std::thread::sleep(Duration::from_secs(20));
    Ok("File context".to_string())
}
/// System ping endpoint
#[get("/v1/system/ping")]
pub async fn ping() -> Result<HttpResponse, Error> {

    let result2 = web::block(move || {
        read_large_file("/path/to/large/file.txt")
    }).await??;

    // println!("result2: {}", result2);
    Ok(HttpResponse::Ok().json(json!({
        "code": 0,
        "message": format!("{}",result2),
        "data": "pong"
    })))
}