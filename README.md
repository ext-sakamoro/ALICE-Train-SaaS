# ALICE-Train SaaS

ML model training job API powered by ALICE-Train backpropagation framework

## License

AGPL-3.0

## Architecture

```
Frontend :3000  -->  API Gateway :8080  -->  Core Engine :8081
```

| Layer | Port | Technology |
|-------|------|-----------|
| Frontend | 3000 | Next.js 14, Tailwind CSS |
| API Gateway | 8080 | Rust, Axum |
| Core Engine | 8081 | Rust, Axum, ALICE-Train |

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST /api/v1/train` | 訓練ジョブ開始（モデル設定・データセット指定） |
| `GET  /api/v1/jobs` | 訓練ジョブ一覧・ステータス取得 |
| `GET  /api/v1/jobs/{id}/metrics` | 訓練メトリクス（loss, accuracy）取得 |
| `GET`  | `/health` | ヘルスチェック |

## Quick Start

```bash
cd services/core-engine
cargo run --release
curl http://localhost:8081/health
```

## Author

Moroya Sakamoto
