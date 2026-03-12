//! ALICE SaaS — Core Engine ライブラリ
//!
//! ヘルスチェック・データ処理などのビジネスロジックを提供する。

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ---------------------------------------------------------------------------
// 定数
// ---------------------------------------------------------------------------

/// パッケージバージョン（Cargo.tomlから取得）
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 処理対象データの最大プレビュー長
pub const MAX_PREVIEW_LEN: usize = 100;

/// デフォルトリッスンポート
pub const DEFAULT_PORT: u16 = 8081;

// ---------------------------------------------------------------------------
// ヘルスチェック関連
// ---------------------------------------------------------------------------

/// ヘルスレスポンス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: f64,
}

/// ヘルスレスポンスを構築する
pub fn build_health_response(uptime: Duration) -> HealthResponse {
    HealthResponse {
        status: "ok".to_string(),
        version: PKG_VERSION.to_string(),
        uptime_seconds: uptime.as_secs_f64(),
    }
}

/// ステータスが正常かどうかを判定する
pub fn is_healthy(resp: &HealthResponse) -> bool {
    resp.status == "ok"
}

// ---------------------------------------------------------------------------
// 稼働時間ユーティリティ
// ---------------------------------------------------------------------------

/// 秒数を人間が読みやすい形式に変換する
pub fn format_uptime(seconds: f64) -> String {
    if seconds < 0.0 {
        return "0s".to_string();
    }
    let total_secs = seconds as u64;
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, mins, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

// ---------------------------------------------------------------------------
// リクエスト/レスポンス型
// ---------------------------------------------------------------------------

/// 処理リクエスト
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ProcessRequest {
    pub data: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

/// 処理レスポンス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessResponse {
    pub result: String,
    pub latency_ms: f64,
}

// ---------------------------------------------------------------------------
// データ処理ロジック
// ---------------------------------------------------------------------------

/// 入力データのプレビュー文字列を生成する（最大 MAX_PREVIEW_LEN 文字）
pub fn truncate_preview(data: &str) -> &str {
    let end = data
        .char_indices()
        .nth(MAX_PREVIEW_LEN)
        .map(|(i, _)| i)
        .unwrap_or(data.len());
    &data[..end]
}

/// データ処理エラー
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessError {
    /// 入力データが空
    EmptyInput,
    /// 入力データが大きすぎる（バイト数）
    InputTooLarge(usize),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::EmptyInput => write!(f, "入力データが空です"),
            ProcessError::InputTooLarge(size) => {
                write!(f, "入力データが大きすぎます: {} bytes", size)
            }
        }
    }
}

/// 入力データの最大サイズ（10MB）
pub const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

/// データを処理し、結果文字列を返す（スタブ実装）
///
/// - 空データの場合は `ProcessError::EmptyInput` を返す
/// - `MAX_INPUT_SIZE` 超過時は `ProcessError::InputTooLarge` を返す
pub fn process_data(data: &str, options: &serde_json::Value) -> Result<String, ProcessError> {
    if data.is_empty() {
        return Err(ProcessError::EmptyInput);
    }
    let size = data.len();
    if size > MAX_INPUT_SIZE {
        return Err(ProcessError::InputTooLarge(size));
    }

    let preview = truncate_preview(data);

    // オプションに "uppercase" が true ならば大文字変換
    let processed = if options
        .get("uppercase")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        format!("[processed] {}", preview.to_uppercase())
    } else {
        format!("[stub] processed: {}", preview)
    };

    Ok(processed)
}

/// `ProcessRequest` を受け取り `ProcessResponse` を構築する
pub fn handle_process_request(req: &ProcessRequest) -> Result<ProcessResponse, ProcessError> {
    let start = std::time::Instant::now();
    let result = process_data(&req.data, &req.options)?;
    Ok(ProcessResponse {
        result,
        latency_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

// ---------------------------------------------------------------------------
// バリデーションユーティリティ
// ---------------------------------------------------------------------------

/// ポート番号が有効範囲（1〜65535）かを検証する
pub fn validate_port(port: u16) -> bool {
    port > 0
}

/// リクエストデータのバリデーション
pub fn validate_request(req: &ProcessRequest) -> Result<(), ProcessError> {
    if req.data.is_empty() {
        return Err(ProcessError::EmptyInput);
    }
    if req.data.len() > MAX_INPUT_SIZE {
        return Err(ProcessError::InputTooLarge(req.data.len()));
    }
    Ok(())
}

// ===========================================================================
// テスト
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // ヘルスレスポンス構築テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_health_response_zero_uptime() {
        // 起動直後（稼働時間0秒）のレスポンスが正しいこと
        let resp = build_health_response(Duration::from_secs(0));
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.version, PKG_VERSION);
        assert_eq!(resp.uptime_seconds, 0.0);
    }

    #[test]
    fn test_health_response_with_uptime() {
        // 稼働時間が正しく反映されること
        let resp = build_health_response(Duration::from_secs(3600));
        assert_eq!(resp.uptime_seconds, 3600.0);
    }

    #[test]
    fn test_health_response_fractional_uptime() {
        // ミリ秒精度の稼働時間が正しく反映されること
        let resp = build_health_response(Duration::from_millis(1500));
        assert!((resp.uptime_seconds - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_health_response_large_uptime() {
        // 長期間稼働（30日）でも正しく計算されること
        let thirty_days = Duration::from_secs(30 * 24 * 3600);
        let resp = build_health_response(thirty_days);
        assert_eq!(resp.uptime_seconds, 2_592_000.0);
    }

    #[test]
    fn test_health_response_version_matches_pkg() {
        // バージョンがCargo.tomlと一致すること
        let resp = build_health_response(Duration::from_secs(0));
        assert_eq!(resp.version, "0.1.0");
    }

    #[test]
    fn test_is_healthy_ok() {
        // status="ok"なら健全と判定されること
        let resp = build_health_response(Duration::from_secs(0));
        assert!(is_healthy(&resp));
    }

    #[test]
    fn test_is_healthy_not_ok() {
        // status="ok"以外なら非健全と判定されること
        let resp = HealthResponse {
            status: "degraded".to_string(),
            version: PKG_VERSION.to_string(),
            uptime_seconds: 0.0,
        };
        assert!(!is_healthy(&resp));
    }

    #[test]
    fn test_health_response_serialization() {
        // ヘルスレスポンスがJSONシリアライズ可能なこと
        let resp = build_health_response(Duration::from_secs(42));
        let json_str = serde_json::to_string(&resp).unwrap();
        assert!(json_str.contains("\"status\":\"ok\""));
        assert!(json_str.contains("\"uptime_seconds\":42.0"));
    }

    #[test]
    fn test_health_response_deserialization() {
        // JSONからデシリアライズ可能なこと
        let json_str = r#"{"status":"ok","version":"0.1.0","uptime_seconds":10.5}"#;
        let resp: HealthResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.uptime_seconds, 10.5);
    }

    // -----------------------------------------------------------------------
    // 稼働時間フォーマットテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_uptime_seconds_only() {
        // 秒のみの表示
        assert_eq!(format_uptime(45.0), "45s");
    }

    #[test]
    fn test_format_uptime_minutes() {
        // 分+秒の表示
        assert_eq!(format_uptime(125.0), "2m 5s");
    }

    #[test]
    fn test_format_uptime_hours() {
        // 時+分+秒の表示
        assert_eq!(format_uptime(3661.0), "1h 1m 1s");
    }

    #[test]
    fn test_format_uptime_days() {
        // 日+時+分+秒の表示
        assert_eq!(format_uptime(90061.0), "1d 1h 1m 1s");
    }

    #[test]
    fn test_format_uptime_zero() {
        // ゼロ秒
        assert_eq!(format_uptime(0.0), "0s");
    }

    #[test]
    fn test_format_uptime_negative() {
        // 負数はゼロとして扱われること
        assert_eq!(format_uptime(-10.0), "0s");
    }

    #[test]
    fn test_format_uptime_large_value() {
        // 365日相当
        let one_year = 365.0 * 86400.0;
        let result = format_uptime(one_year);
        assert!(result.starts_with("365d"));
    }

    // -----------------------------------------------------------------------
    // truncate_preview テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncate_preview_short_string() {
        // 短い文字列はそのまま返ること
        assert_eq!(truncate_preview("hello"), "hello");
    }

    #[test]
    fn test_truncate_preview_exact_limit() {
        // ちょうど100文字は切り詰められないこと
        let data = "a".repeat(100);
        assert_eq!(truncate_preview(&data).len(), 100);
    }

    #[test]
    fn test_truncate_preview_over_limit() {
        // 100文字を超える場合は切り詰められること
        let data = "b".repeat(200);
        assert_eq!(truncate_preview(&data).len(), 100);
    }

    #[test]
    fn test_truncate_preview_empty() {
        // 空文字列はそのまま返ること
        assert_eq!(truncate_preview(""), "");
    }

    #[test]
    fn test_truncate_preview_unicode() {
        // マルチバイト文字が文字単位で切り詰められること
        let data = "あ".repeat(101);
        let preview = truncate_preview(&data);
        assert_eq!(preview.chars().count(), 100);
    }

    // -----------------------------------------------------------------------
    // process_data テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_process_data_normal() {
        // 通常のデータが処理されること
        let result = process_data("test input", &json!({})).unwrap();
        assert_eq!(result, "[stub] processed: test input");
    }

    #[test]
    fn test_process_data_empty_input() {
        // 空入力でエラーが返ること
        let err = process_data("", &json!({})).unwrap_err();
        assert_eq!(err, ProcessError::EmptyInput);
    }

    #[test]
    fn test_process_data_large_input_within_limit() {
        // 制限内の大きなデータが処理されること
        let data = "x".repeat(1000);
        let result = process_data(&data, &json!({})).unwrap();
        assert!(result.starts_with("[stub] processed: "));
        // プレビューは100文字に切り詰められていること
        assert!(result.len() <= "[stub] processed: ".len() + MAX_PREVIEW_LEN);
    }

    #[test]
    fn test_process_data_exceeds_max_size() {
        // 10MBを超えるデータでエラーが返ること
        let data = "z".repeat(MAX_INPUT_SIZE + 1);
        let err = process_data(&data, &json!({})).unwrap_err();
        assert_eq!(err, ProcessError::InputTooLarge(MAX_INPUT_SIZE + 1));
    }

    #[test]
    fn test_process_data_uppercase_option() {
        // uppercaseオプションで大文字変換されること
        let result = process_data("hello world", &json!({"uppercase": true})).unwrap();
        assert_eq!(result, "[processed] HELLO WORLD");
    }

    #[test]
    fn test_process_data_uppercase_false() {
        // uppercase=falseではスタブ処理のままであること
        let result = process_data("hello", &json!({"uppercase": false})).unwrap();
        assert_eq!(result, "[stub] processed: hello");
    }

    #[test]
    fn test_process_data_no_options() {
        // オプション未指定（null）でもスタブ処理されること
        let result = process_data("data", &serde_json::Value::Null).unwrap();
        assert_eq!(result, "[stub] processed: data");
    }

    #[test]
    fn test_process_data_single_char() {
        // 1文字の入力が処理されること
        let result = process_data("a", &json!({})).unwrap();
        assert_eq!(result, "[stub] processed: a");
    }

    // -----------------------------------------------------------------------
    // handle_process_request テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_handle_process_request_ok() {
        // 正常なリクエストが処理されること
        let req = ProcessRequest {
            data: "test".to_string(),
            options: json!({}),
        };
        let resp = handle_process_request(&req).unwrap();
        assert!(resp.result.contains("test"));
        assert!(resp.latency_ms >= 0.0);
    }

    #[test]
    fn test_handle_process_request_empty() {
        // 空データリクエストでエラーが返ること
        let req = ProcessRequest {
            data: String::new(),
            options: json!({}),
        };
        let err = handle_process_request(&req).unwrap_err();
        assert_eq!(err, ProcessError::EmptyInput);
    }

    #[test]
    fn test_handle_process_request_latency_positive() {
        // レイテンシが非負であること
        let req = ProcessRequest {
            data: "benchmark".to_string(),
            options: json!({}),
        };
        let resp = handle_process_request(&req).unwrap();
        assert!(resp.latency_ms >= 0.0);
    }

    // -----------------------------------------------------------------------
    // バリデーションテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_request_valid() {
        // 有効なリクエストがOKを返すこと
        let req = ProcessRequest {
            data: "valid".to_string(),
            options: json!({}),
        };
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn test_validate_request_empty() {
        // 空データリクエストがエラーを返すこと
        let req = ProcessRequest {
            data: String::new(),
            options: json!({}),
        };
        assert_eq!(
            validate_request(&req).unwrap_err(),
            ProcessError::EmptyInput
        );
    }

    #[test]
    fn test_validate_port_valid() {
        // 有効なポート番号
        assert!(validate_port(8081));
        assert!(validate_port(1));
        assert!(validate_port(65535));
    }

    #[test]
    fn test_validate_port_default() {
        // デフォルトポートが有効であること
        assert!(validate_port(DEFAULT_PORT));
    }

    // -----------------------------------------------------------------------
    // ProcessError 表示テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_process_error_display_empty() {
        // 空入力エラーの表示
        let err = ProcessError::EmptyInput;
        assert_eq!(format!("{}", err), "入力データが空です");
    }

    #[test]
    fn test_process_error_display_too_large() {
        // サイズ超過エラーの表示
        let err = ProcessError::InputTooLarge(999);
        assert_eq!(format!("{}", err), "入力データが大きすぎます: 999 bytes");
    }

    // -----------------------------------------------------------------------
    // シリアライズ/デシリアライズテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_process_request_deserialize() {
        // JSONからProcessRequestへのデシリアライズ
        let json_str = r#"{"data":"hello"}"#;
        let req: ProcessRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.data, "hello");
        assert_eq!(req.options, serde_json::Value::Null);
    }

    #[test]
    fn test_process_request_deserialize_with_options() {
        // オプション付きJSONからのデシリアライズ
        let json_str = r#"{"data":"hello","options":{"uppercase":true}}"#;
        let req: ProcessRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.data, "hello");
        assert_eq!(req.options["uppercase"], true);
    }

    #[test]
    fn test_process_response_serialize() {
        // ProcessResponseのJSONシリアライズ
        let resp = ProcessResponse {
            result: "done".to_string(),
            latency_ms: 1.5,
        };
        let json_str = serde_json::to_string(&resp).unwrap();
        assert!(json_str.contains("\"result\":\"done\""));
        assert!(json_str.contains("\"latency_ms\":1.5"));
    }

    // -----------------------------------------------------------------------
    // 定数テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_constants() {
        // 定数値が期待通りであること
        assert_eq!(MAX_PREVIEW_LEN, 100);
        assert_eq!(DEFAULT_PORT, 8081);
        assert_eq!(MAX_INPUT_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_pkg_version_not_empty() {
        // パッケージバージョンが空でないこと
        assert!(!PKG_VERSION.is_empty());
    }
}
