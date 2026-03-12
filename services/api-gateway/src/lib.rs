//! ALICE SaaS — API Gateway ライブラリ
//!
//! main.rs から抽出したテスト可能なビジネスロジック

use serde::Serialize;

/// ヘルスチェック応答の構造体
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GatewayHealth {
    pub status: String,
    pub version: String,
}

/// ヘルスチェック応答を構築する
pub fn build_health_response(version: &str) -> GatewayHealth {
    GatewayHealth {
        status: "ok".to_string(),
        version: version.to_string(),
    }
}

/// ゲートウェイの設定値
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    /// ゲートウェイがリッスンするポート
    pub port: u16,
    /// プロキシ先のアップストリームURL（スキーム + ホスト + ポート）
    pub upstream_url: String,
    /// リクエストボディの最大サイズ（バイト）
    pub max_body_size: usize,
}

/// デフォルトのリッスンポート
pub const DEFAULT_PORT: u16 = 8080;
/// デフォルトのアップストリームURL
pub const DEFAULT_UPSTREAM_URL: &str = "http://localhost:8081";
/// デフォルトの最大ボディサイズ（10 MiB）
pub const DEFAULT_MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            upstream_url: DEFAULT_UPSTREAM_URL.to_string(),
            max_body_size: DEFAULT_MAX_BODY_SIZE,
        }
    }
}

impl GatewayConfig {
    /// 環境変数からゲートウェイ設定を読み込む
    ///
    /// - `GATEWAY_PORT`: リッスンポート（デフォルト: 8080）
    /// - `UPSTREAM_URL`: プロキシ先URL（デフォルト: http://localhost:8081）
    /// - `MAX_BODY_SIZE`: 最大ボディサイズ（バイト、デフォルト: 10485760）
    pub fn from_env() -> Self {
        let port = std::env::var("GATEWAY_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);

        let upstream_url =
            std::env::var("UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM_URL.to_string());

        let max_body_size = std::env::var("MAX_BODY_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_BODY_SIZE);

        Self {
            port,
            upstream_url,
            max_body_size,
        }
    }
}

/// プロキシ先の完全なURLを構築する
///
/// `upstream_base` にパスとクエリ文字列を結合して返す。
/// 末尾スラッシュの重複を防ぐため、upstream_base の末尾スラッシュは除去する。
pub fn build_proxy_url(upstream_base: &str, path: &str, query: Option<&str>) -> String {
    let base = upstream_base.trim_end_matches('/');
    let query_part = match query {
        Some(q) if !q.is_empty() => format!("?{q}"),
        _ => String::new(),
    };
    format!("{base}{path}{query_part}")
}

/// ボディサイズが制限内かどうかを検証する
///
/// 制限内なら `Ok(())`、超過なら `Err` とエラーメッセージを返す。
pub fn validate_body_size(size: usize, max_size: usize) -> Result<(), String> {
    if size > max_size {
        Err(format!(
            "ボディサイズ超過: {size} バイト（上限: {max_size} バイト）"
        ))
    } else {
        Ok(())
    }
}

/// SocketAddr を構築するヘルパー
pub fn build_listen_addr(port: u16) -> std::net::SocketAddr {
    std::net::SocketAddr::from(([0, 0, 0, 0], port))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ヘルスチェック応答テスト =====

    #[test]
    fn test_health_response_status_is_ok() {
        let h = build_health_response("1.0.0");
        assert_eq!(h.status, "ok");
    }

    #[test]
    fn test_health_response_version() {
        let h = build_health_response("0.1.0");
        assert_eq!(h.version, "0.1.0");
    }

    #[test]
    fn test_health_response_empty_version() {
        let h = build_health_response("");
        assert_eq!(h.version, "");
        assert_eq!(h.status, "ok");
    }

    #[test]
    fn test_health_response_serializes_to_json() {
        let h = build_health_response("0.1.0");
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
    }

    #[test]
    fn test_health_response_equality() {
        let a = build_health_response("1.0.0");
        let b = build_health_response("1.0.0");
        assert_eq!(a, b);
    }

    #[test]
    fn test_health_response_inequality() {
        let a = build_health_response("1.0.0");
        let b = build_health_response("2.0.0");
        assert_ne!(a, b);
    }

    // ===== プロキシURL構築テスト =====

    #[test]
    fn test_proxy_url_simple_path() {
        let url = build_proxy_url("http://localhost:8081", "/api/v1/trains", None);
        assert_eq!(url, "http://localhost:8081/api/v1/trains");
    }

    #[test]
    fn test_proxy_url_with_query() {
        let url = build_proxy_url("http://localhost:8081", "/search", Some("q=test&limit=10"));
        assert_eq!(url, "http://localhost:8081/search?q=test&limit=10");
    }

    #[test]
    fn test_proxy_url_root_path() {
        let url = build_proxy_url("http://localhost:8081", "/", None);
        assert_eq!(url, "http://localhost:8081/");
    }

    #[test]
    fn test_proxy_url_empty_query() {
        let url = build_proxy_url("http://localhost:8081", "/path", Some(""));
        assert_eq!(url, "http://localhost:8081/path");
    }

    #[test]
    fn test_proxy_url_upstream_trailing_slash() {
        // アップストリームURLの末尾スラッシュを除去して重複を防ぐ
        let url = build_proxy_url("http://localhost:8081/", "/api", None);
        assert_eq!(url, "http://localhost:8081/api");
    }

    #[test]
    fn test_proxy_url_deep_nested_path() {
        let url = build_proxy_url("http://upstream:9090", "/a/b/c/d/e", Some("deep=true"));
        assert_eq!(url, "http://upstream:9090/a/b/c/d/e?deep=true");
    }

    #[test]
    fn test_proxy_url_special_characters_in_query() {
        let url = build_proxy_url(
            "http://localhost:8081",
            "/search",
            Some("q=hello%20world&lang=ja"),
        );
        assert_eq!(url, "http://localhost:8081/search?q=hello%20world&lang=ja");
    }

    #[test]
    fn test_proxy_url_no_port_in_upstream() {
        let url = build_proxy_url("http://core-engine", "/health", None);
        assert_eq!(url, "http://core-engine/health");
    }

    #[test]
    fn test_proxy_url_https_upstream() {
        let url = build_proxy_url("https://api.example.com", "/v2/data", Some("format=json"));
        assert_eq!(url, "https://api.example.com/v2/data?format=json");
    }

    #[test]
    fn test_proxy_url_query_none_vs_some_empty() {
        // None と Some("") はどちらもクエリなしとして扱う
        let a = build_proxy_url("http://host", "/path", None);
        let b = build_proxy_url("http://host", "/path", Some(""));
        assert_eq!(a, b);
    }

    // ===== ボディサイズ検証テスト =====

    #[test]
    fn test_validate_body_size_within_limit() {
        assert!(validate_body_size(1000, DEFAULT_MAX_BODY_SIZE).is_ok());
    }

    #[test]
    fn test_validate_body_size_exactly_at_limit() {
        // 上限ちょうどはOK
        assert!(validate_body_size(DEFAULT_MAX_BODY_SIZE, DEFAULT_MAX_BODY_SIZE).is_ok());
    }

    #[test]
    fn test_validate_body_size_exceeds_limit() {
        assert!(validate_body_size(DEFAULT_MAX_BODY_SIZE + 1, DEFAULT_MAX_BODY_SIZE).is_err());
    }

    #[test]
    fn test_validate_body_size_zero() {
        assert!(validate_body_size(0, DEFAULT_MAX_BODY_SIZE).is_ok());
    }

    #[test]
    fn test_validate_body_size_zero_limit() {
        // 上限0バイトの場合、0バイトはOK、1バイトはNG
        assert!(validate_body_size(0, 0).is_ok());
        assert!(validate_body_size(1, 0).is_err());
    }

    #[test]
    fn test_validate_body_size_error_message_contains_sizes() {
        let err = validate_body_size(2000, 1000).unwrap_err();
        assert!(err.contains("2000"));
        assert!(err.contains("1000"));
    }

    #[test]
    fn test_validate_body_size_custom_limit() {
        let limit = 512;
        assert!(validate_body_size(512, limit).is_ok());
        assert!(validate_body_size(513, limit).is_err());
    }

    // ===== GatewayConfig テスト =====

    #[test]
    fn test_config_default_port() {
        let cfg = GatewayConfig::default();
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn test_config_default_upstream() {
        let cfg = GatewayConfig::default();
        assert_eq!(cfg.upstream_url, "http://localhost:8081");
    }

    #[test]
    fn test_config_default_max_body_size() {
        let cfg = GatewayConfig::default();
        assert_eq!(cfg.max_body_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_config_constants() {
        assert_eq!(DEFAULT_PORT, 8080);
        assert_eq!(DEFAULT_UPSTREAM_URL, "http://localhost:8081");
        assert_eq!(DEFAULT_MAX_BODY_SIZE, 10_485_760);
    }

    #[test]
    fn test_config_from_env_defaults() {
        // 環境変数未設定時はデフォルト値になることを確認
        // （他テストとの競合を避けるため、設定済み変数を除去）
        std::env::remove_var("GATEWAY_PORT");
        std::env::remove_var("UPSTREAM_URL");
        std::env::remove_var("MAX_BODY_SIZE");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.port, DEFAULT_PORT);
        assert_eq!(cfg.upstream_url, DEFAULT_UPSTREAM_URL);
        assert_eq!(cfg.max_body_size, DEFAULT_MAX_BODY_SIZE);
    }

    #[test]
    fn test_config_from_env_custom_port() {
        std::env::set_var("GATEWAY_PORT", "9090");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.port, 9090);
        std::env::remove_var("GATEWAY_PORT");
    }

    #[test]
    fn test_config_from_env_custom_upstream() {
        std::env::set_var("UPSTREAM_URL", "http://backend:3000");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.upstream_url, "http://backend:3000");
        std::env::remove_var("UPSTREAM_URL");
    }

    #[test]
    fn test_config_from_env_custom_max_body() {
        std::env::set_var("MAX_BODY_SIZE", "1048576");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.max_body_size, 1_048_576);
        std::env::remove_var("MAX_BODY_SIZE");
    }

    #[test]
    fn test_config_from_env_invalid_port_falls_back() {
        // 不正な値の場合はデフォルトにフォールバック
        std::env::set_var("GATEWAY_PORT", "not_a_number");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.port, DEFAULT_PORT);
        std::env::remove_var("GATEWAY_PORT");
    }

    #[test]
    fn test_config_from_env_invalid_body_size_falls_back() {
        std::env::set_var("MAX_BODY_SIZE", "abc");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.max_body_size, DEFAULT_MAX_BODY_SIZE);
        std::env::remove_var("MAX_BODY_SIZE");
    }

    #[test]
    fn test_config_from_env_port_zero() {
        // ポート0は有効な値（OS任意割り当て）
        std::env::set_var("GATEWAY_PORT", "0");
        let cfg = GatewayConfig::from_env();
        assert_eq!(cfg.port, 0);
        std::env::remove_var("GATEWAY_PORT");
    }

    // ===== SocketAddr 構築テスト =====

    #[test]
    fn test_build_listen_addr_default() {
        let addr = build_listen_addr(8080);
        assert_eq!(addr.port(), 8080);
        assert!(addr.ip().is_unspecified());
    }

    #[test]
    fn test_build_listen_addr_custom_port() {
        let addr = build_listen_addr(3000);
        assert_eq!(addr.port(), 3000);
    }

    #[test]
    fn test_build_listen_addr_zero() {
        let addr = build_listen_addr(0);
        assert_eq!(addr.port(), 0);
    }
}
