# Zit AI Mentor — AWS Backend

Production AI backend for zit's AI Mentor feature. Runs on AWS Lambda with Amazon Bedrock (Claude 3 Sonnet).

## Architecture

```
zit CLI (Rust) ──HTTPS──▶ API Gateway ──▶ Lambda (Python 3.12) ──▶ Amazon Bedrock (Claude 3 Sonnet)
                          │                                          
                          ├─ POST /mentor   (AI requests)
                          └─ GET  /health   (connectivity check)
```

- **API Gateway**: REST API with API key auth, usage plan (5,000 req/month, 10 req/sec burst)
- **Lambda**: Python 3.12, 512 MB, 60s timeout, X-Ray tracing
- **Bedrock**: Claude 3 Sonnet (`anthropic.claude-3-sonnet-20240229-v1:0`)
- **Security**: Scoped IAM (Bedrock invoke only for the specific model), API key required

## Prerequisites

1. **AWS CLI** configured with credentials (`aws configure`)
2. **AWS SAM CLI** (`brew install aws-sam-cli`)
3. **Python 3.12** (`brew install python@3.12`) — or Docker for container-based builds
4. **Bedrock access**: Anthropic models auto-activate on first use. Your IAM user needs `aws-marketplace:ViewSubscriptions` and `aws-marketplace:Subscribe` permissions for the first invocation.

## Project Structure

```
aws/
├── deploy.sh               # One-command deploy script
├── README.md               # This file
├── lambda/
│   ├── handler.py          # Lambda handler (request routing, Bedrock calls)
│   ├── prompts.py          # System prompts per request type
│   └── requirements.txt    # Python deps (boto3)
├── infrastructure/
│   └── template.yaml       # SAM/CloudFormation template
└── tests/
    ├── test_handler.py     # 27 unit tests
    └── pytest.ini
```

## Deployment

### Quick deploy

```bash
cd aws
chmod +x deploy.sh
./deploy.sh
```

The script:
1. Builds the Lambda with SAM (uses Docker if Python 3.12 isn't on PATH)
2. Deploys the CloudFormation stack
3. Prints the API endpoint and instructions to retrieve the API key

### Environment variables

Create `aws/.env` to override defaults:

```bash
AWS_REGION=ap-south-1                                      # Default
ENVIRONMENT=dev                                            # Default
BEDROCK_MODEL_ID=anthropic.claude-3-sonnet-20240229-v1:0   # Default
```

### Get API credentials after deploy

```bash
# API endpoint (from stack outputs)
aws cloudformation describe-stacks \
  --stack-name zit-ai-mentor-dev \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' \
  --output text --region ap-south-1

# API key
aws apigateway get-api-keys \
  --include-values \
  --query 'items[?contains(stageKeys, `*/dev`)].value' \
  --output text --region ap-south-1
```

### Configure zit

**Option A — Config file** (`~/.config/zit/config.toml`):

```toml
[ai]
enabled = true
endpoint = "https://xxx.execute-api.ap-south-1.amazonaws.com/dev/mentor"
api_key = "your-api-key"
timeout_secs = 30
```

**Option B — Environment variables**:

```bash
export ZIT_AI_ENDPOINT="https://xxx.execute-api.ap-south-1.amazonaws.com/dev/mentor"
export ZIT_AI_API_KEY="your-api-key"
```

## API Reference

### `GET /health`

Health check — no request body needed.

```bash
curl -H "x-api-key: $API_KEY" "$ENDPOINT/../health"
```

Response:
```json
{
  "success": true,
  "model": "anthropic.claude-3-sonnet-20240229-v1:0",
  "response": { "status": "healthy", "version": "1.0.0" }
}
```

### `POST /mentor`

All AI requests. Headers: `Content-Type: application/json`, `x-api-key: <key>`.

Body size limit: **128 KB**.

#### Request types

| Type | Required fields | Description |
|------|----------------|-------------|
| `explain` | `context` | Explain repo state |
| `commit_suggestion` | `context` (with staged files + diff) | Suggest commit messages |
| `error` | `error`, `context` | Explain a git error |
| `recommend` | `query`, `context` | Recommend a git operation |
| `learn` | `query` | General git learning |

#### Example: Explain

```json
{
  "type": "explain",
  "context": {
    "branch": "main",
    "staged_files": ["src/main.rs"],
    "unstaged_files": ["README.md"]
  },
  "query": "What should I do next?"
}
```

#### Example: Commit suggestion

```json
{
  "type": "commit_suggestion",
  "context": {
    "branch": "feature",
    "staged_files": ["src/auth.rs", "src/login.rs"],
    "diff_stats": { "files_changed": 2, "insertions": 150, "deletions": 20 },
    "diff": "+fn authenticate() { ... }"
  }
}
```

#### Example: Error explanation

```json
{
  "type": "error",
  "error": "error: Your local changes would be overwritten by merge.",
  "context": { "branch": "feature", "unstaged_files": ["config.json"] }
}
```

#### Example: Recommendation

```json
{
  "type": "recommend",
  "query": "I want to undo my last 3 commits",
  "context": { "branch": "main" }
}
```

#### Response format

```json
{
  "success": true,
  "timestamp": "2026-02-26T06:19:09.449Z",
  "model": "anthropic.claude-3-sonnet-20240229-v1:0",
  "response": {
    "type": "explanation",
    "content": "Your repository is in a clean state..."
  }
}
```

## Testing

```bash
cd aws
python3 -m pytest tests/ -v    # 27 tests
```

Tests cover: request validation, response building, health check, CORS, error handling, prompt generation — all without AWS credentials.

## Client Integration

The Rust AI client ([src/ai/client.rs](../src/ai/client.rs)) handles:

- **Retry with backoff**: 2 retries, 500ms → 1s exponential backoff
- **Error classification**: Distinguishes transient (5xx, timeout, DNS) from permanent (4xx) errors
- **Diff truncation**: Caps diff content at 4,000 chars to avoid token explosion
- **Non-blocking**: All AI calls run in background threads via `mpsc` channels
- **Request body limit**: Lambda rejects requests > 128 KB

## Cost Estimates

| Model | Input (per 1K tokens) | Output (per 1K tokens) | Typical request |
|-------|----------------------|----------------------|-----------------|
| Claude 3 Haiku | $0.00025 | $0.00125 | ~$0.001 |
| Claude 3 Sonnet | $0.003 | $0.015 | ~$0.01 |

Default quota: 5,000 requests/month. Estimated cost with Sonnet: **~$50/month**.

## Troubleshooting

### "Access Denied" on Bedrock

Your IAM user (or Lambda role) needs `aws-marketplace:ViewSubscriptions` and `aws-marketplace:Subscribe` for first-time Anthropic model activation. Add the permissions, wait ~90 seconds for IAM propagation.

### SAM build fails: "python3.12 not found"

Install Python 3.12 (`brew install python@3.12`) or the deploy script will automatically use Docker (`--use-container`).

### Cold start latency

First request after idle may take 2-3s (Lambda cold start). Subsequent requests: ~500ms–1s.

### Check Lambda logs

```bash
aws logs tail /aws/lambda/zit-ai-mentor-dev --region ap-south-1 --follow
```

## Cleanup

Delete all AWS resources:

```bash
aws cloudformation delete-stack --stack-name zit-ai-mentor-dev --region ap-south-1
```
