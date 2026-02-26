//! DevOps Info Service - Rust Implementation
//!
//! A web application providing detailed information about itself and its runtime environment.
//!
//! ## Features
//! - GET `/` - Returns comprehensive service and system information
//! - GET `/health` - Health check endpoint for monitoring
//! - Configurable via environment variables
//! - Built with Actix-web for async performance

use std::env;
use std::time::{Duration, SystemTime};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use chrono::Utc;
use serde::Serialize;
use sysinfo::System;

// Service start time (captured at startup)
static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

#[derive(Serialize)]
struct ServiceStatistics {
    service: ServiceInfo,
    system: SystemInfo,
    runtime: RuntimeInfo,
    request: RequestInfo,
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize)]
struct ServiceInfo {
    name: String,
    version: String,
    description: String,
    framework: String,
    language: String,
}

#[derive(Serialize)]
struct SystemInfo {
    hostname: String,
    platform: String,
    platform_version: String,
    architecture: String,
    cpu_count: usize,
    rust_version: String,
    total_memory: u64,
    used_memory: u64,
}

#[derive(Serialize)]
struct RuntimeInfo {
    uptime_seconds: u64,
    uptime_human: String,
    current_time: String,
    timezone: String,
}

#[derive(Serialize)]
struct RequestInfo {
    client_ip: String,
    user_agent: String,
    method: String,
    path: String,
}

#[derive(Serialize)]
struct Endpoint {
    path: String,
    method: String,
    description: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    uptime_seconds: u64,
}

/// Get system information using sysinfo crate
fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    SystemInfo {
        hostname,
        platform: env::consts::OS.to_string(),
        platform_version: System::long_os_version().unwrap_or_else(|| "unknown".to_string()),
        architecture: env::consts::ARCH.to_string(),
        cpu_count: sys.cpus().len(),
        rust_version: env!("CARGO_PKG_VERSION").to_string(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
    }
}

/// Calculate uptime since service start
fn get_uptime() -> RuntimeInfo {
    let start_time = START_TIME.get().expect("Start time not set");
    let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
    let seconds = duration.as_secs();

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let remaining_seconds = seconds % 60;

    RuntimeInfo {
        uptime_seconds: seconds,
        uptime_human: format!(
            "{} hours, {} minutes, {} seconds",
            hours, minutes, remaining_seconds
        ),
        current_time: Utc::now().to_rfc3339(),
        timezone: "UTC".to_string(),
    }
}

/// Extract request information
fn get_request_info(req: &HttpRequest) -> RequestInfo {
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let user_agent = req
        .headers()
        .get("user-agent")
        .map(|h| h.to_str().unwrap_or("unknown"))
        .unwrap_or("unknown")
        .to_string();

    RequestInfo {
        client_ip,
        user_agent,
        method: req.method().to_string(),
        path: req.path().to_string(),
    }
}

/// Main endpoint handler
async fn index(req: HttpRequest) -> Result<HttpResponse> {
    let service_info = ServiceStatistics {
        service: ServiceInfo {
            name: "devops-info-service".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "DevOps course info service - Rust implementation".to_string(),
            framework: "Actix-web".to_string(),
            language: "Rust".to_string(),
        },
        system: get_system_info(),
        runtime: get_uptime(),
        request: get_request_info(&req),
        endpoints: vec![
            Endpoint {
                path: "/".to_string(),
                method: "GET".to_string(),
                description: "Service information".to_string(),
            },
            Endpoint {
                path: "/health".to_string(),
                method: "GET".to_string(),
                description: "Health check".to_string(),
            },
        ],
    };

    Ok(HttpResponse::Ok().json(service_info))
}

/// Health check endpoint handler
async fn health() -> HttpResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        uptime_seconds: get_uptime().uptime_seconds,
    };

    HttpResponse::Ok().json(response)
}

/// Metrics endpoint handler (for future Prometheus integration)
async fn metrics() -> HttpResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let metrics = format!(
        "# HELP devops_info_service_info Service information\n\
        # TYPE devops_info_service_info gauge\n\
        devops_info_service_info{{version=\"{}\", language=\"rust\", framework=\"actix-web\"}} 1\n\
        \n\
        # HELP system_cpu_count Number of CPUs\n\
        # TYPE system_cpu_count gauge\n\
        system_cpu_count {}\n\
        \n\
        # HELP system_memory_total Total system memory in bytes\n\
        # TYPE system_memory_total gauge\n\
        system_memory_total {}\n\
        \n\
        # HELP system_memory_used Used system memory in bytes\n\
        # TYPE system_memory_used gauge\n\
        system_memory_used {}\n\
        \n\
        # HELP service_uptime_seconds Service uptime in seconds\n\
        # TYPE service_uptime_seconds gauge\n\
        service_uptime_seconds {}\n",
        env!("CARGO_PKG_VERSION"),
        sys.cpus().len(),
        sys.total_memory(),
        sys.used_memory(),
        get_uptime().uptime_seconds
    );

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize start time
    START_TIME.set(SystemTime::now()).unwrap();

    // Configure from environment variables with defaults
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    println!("🚀 Starting DevOps Info Service (Rust)");
    println!("📡 Listening on: http://{}", bind_address);
    println!("🔧 Framework: Actix-web");
    println!("⚙️  Environment:");
    println!("   - HOST: {}", host);
    println!("   - PORT: {}", port);

    // Start HTTP server
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .route("/metrics", web::get().to(metrics))
    })
    .bind(&bind_address)?
    .run()
    .await
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().route("/", web::get().to(index))).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["service"]["name"], "devops-info-service");
        assert_eq!(json["service"]["framework"], "Actix-web");
        assert_eq!(json["service"]["language"], "Rust");
    }

    #[actix_web::test]
    async fn test_health_get() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert!(json["timestamp"].is_string());
        assert!(json["uptime_seconds"].is_number());
    }
}
