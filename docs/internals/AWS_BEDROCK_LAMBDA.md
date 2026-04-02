# Amazon Bedrock + AWS Lambda

## What It Is
Amazon Bedrock is AWS's managed AI service that provides access to foundation models (like Claude 3 Sonnet). AWS Lambda is a serverless compute service that runs code in response to events without provisioning servers.

## Why My Project Uses It
Zit's AI Mentor feature needs a backend to process natural language requests. We deploy a Python Lambda function that receives requests from the Rust client, constructs prompts, and calls Bedrock's Claude 3 Sonnet model. This gives us: (1) no server management, (2) pay-per-use pricing, (3) access to state-of-the-art Claude models via AWS.

## Where It Appears in My Project
- `aws/lambda/handler.py` (410 lines) — The Lambda function code
- `aws/lambda/prompts.py` (216 lines) — System prompts for each request type
- `aws/infrastructure/template.yaml` (178 lines) — SAM/CloudFormation IaC template
- `aws/deploy.sh` (87 lines) — One-command deployment script
- `src/ai/client.rs` lines 257–308: `call_bedrock()` — client-side Bedrock path
- `src/ai/provider.rs` lines 109–239: `BedrockProvider` struct

## How It Works Internally
1. **Client sends request**: Rust client POSTs a JSON `MentorRequest` to the API Gateway URL with an `x-api-key` header.
2. **API Gateway authenticates**: Validates the API key and forwards the request to Lambda.
3. **Lambda processes**: `handler.py` parses the request type, selects the system prompt, formats the repo context, and calls Bedrock.
4. **Bedrock invokes Claude**: The `invoke_model()` API sends the prompt to Claude 3 Sonnet and returns the completion.
5. **Response flows back**: Lambda wraps the AI response in a standardized JSON envelope; API Gateway returns it to the client.
6. **Streaming for reviews**: `invoke_model_with_response_stream()` is used for code reviews and merge resolutions for faster first-token latency.

## Key Concepts I Must Know
- **Serverless**: No servers to manage; Lambda scales automatically from 0 to thousands of concurrent invocations
- **Cold start**: First invocation takes ~1–2 seconds to initialize the Python runtime and boto3 client. Subsequent invocations reuse the warm container.
- **IAM permissions**: The Lambda function has fine-grained permissions — only `bedrock:InvokeModel` on the specific model ARN
- **API Gateway + API Keys**: Authentication is done via API keys with a usage plan (5,000 requests/month, 10 req/sec, burst 20)
- **CloudWatch Alarms**: 4 alarms monitor errors, latency (p90 > 30s), throttles, and 5xx rates
- **Model ID**: `anthropic.claude-3-sonnet-20240229-v1:0` — configurable via environment variable
- **Max tokens**: Limited to 1024 tokens per response to control costs
- **Diff truncation**: Both client and Lambda cap diff content at 4,000 characters to avoid token explosion

## How My Code Uses It (Annotated)
```python
# aws/lambda/handler.py — The Lambda function
def invoke_bedrock(system_prompt: str, user_message: str) -> str:
    client = get_bedrock_client()          # Lazy boto3 client init
    request_body = {
        "anthropic_version": "bedrock-2023-05-31",  # Required Bedrock version header
        "max_tokens": 1024,                # Cap response length
        "temperature": 0.7,                # Balance creativity vs accuracy
        "system": system_prompt,           # Role-specific prompt (from prompts.py)
        "messages": [{"role": "user", "content": user_message}]  # The actual question
    }
    response = client.invoke_model(        # Call Bedrock
        modelId=MODEL_ID,
        contentType='application/json',
        body=json.dumps(request_body)
    )
    response_body = json.loads(response['body'].read())
    return response_body['content'][0]['text']  # Extract the AI response
```

## What Could Go Wrong
- **Cold start latency**: First request after idle period takes 1–2s extra
- **Token limits**: Very large diffs can exceed the 1024 output token limit, causing truncated responses
- **Region availability**: Bedrock with Claude 3 Sonnet is only available in certain AWS regions (we use `ap-south-1`)
- **Cost**: Each request costs ~$0.003–0.01 depending on input/output tokens
- **Rate limiting**: Usage plan limits to 5,000 requests/month; exceeding this returns 429 errors

## Judge-Ready One-Liner
"Our AI backend is a serverless Python Lambda function on AWS that calls Claude 3 Sonnet through Amazon Bedrock — it scales automatically, costs fractions of a cent per request, and requires zero server management."
