# SAM / CloudFormation (Infrastructure as Code)

## What It Is
AWS Serverless Application Model (SAM) is an extension of CloudFormation that simplifies defining serverless resources (Lambda functions, API Gateway, etc.). The template defines the entire infrastructure as YAML code.

## Why My Project Uses It
We need repeatable, one-command deployment of the AI backend. SAM turns our Lambda + API Gateway + IAM + CloudWatch setup into a single `template.yaml` file that anyone can deploy with `sam deploy`.

## Where It Appears in My Project
- `aws/infrastructure/template.yaml` (178 lines) — The complete infrastructure definition
- `aws/deploy.sh` (87 lines) — Wrapper script for `sam build` + `sam deploy`

## How It Works Internally
1. `sam build` packages the Lambda code (handler.py + prompts.py + dependencies)
2. `sam deploy` creates/updates a CloudFormation stack in AWS, provisioning:
   - **Lambda function**: Python 3.12, 512MB memory, 60s timeout, X-Ray tracing
   - **API Gateway**: REST API with CORS headers, API key authentication
   - **Usage Plan**: 5,000 requests/month, 10 req/sec, burst 20
   - **IAM Policy**: Least-privilege — only `bedrock:InvokeModel` on the specific model ARN
   - **CloudWatch Alarms**: 4 alarms for errors, latency, throttles, and 5xx rates
3. Outputs include the API endpoint URL, health endpoint, function ARN, and API key retrieval command

## Key Concepts I Must Know
- **Infrastructure as Code**: The entire backend is defined in one YAML file — version-controllable, auditable
- **API Key authentication**: Every request must include `x-api-key` header
- **Least privilege IAM**: The Lambda can ONLY call Bedrock's InvokeModel on the configured model
- **Usage plan**: 5,000 req/month prevents accidental cost explosions
- **Monitoring**: 4 CloudWatch alarms with `notBreaching` for missing data

## How My Code Uses It (Annotated)
```yaml
# aws/infrastructure/template.yaml — Key resource definitions
ZitAIMentorFunction:
  Type: AWS::Serverless::Function
  Properties:
    FunctionName: !Sub zit-ai-mentor-${Environment}  # dev or prod
    Runtime: python3.12
    Handler: handler.lambda_handler
    MemorySize: 512                                   # MB
    Timeout: 60                                       # seconds
    Policies:
      - Effect: Allow
        Action: bedrock:InvokeModel                   # Only this action
        Resource: !Sub 'arn:aws:bedrock:${AWS::Region}::foundation-model/${BedrockModelId}'
```

## What Could Go Wrong
- **Region support**: Not all regions have Bedrock. Default is `ap-south-1` (Mumbai) — relevant for Indian developers
- **IAM permissions**: Deployer needs CloudFormation, Lambda, API Gateway, IAM permissions
- **Cold starts**: First invocation after idle has ~1-2s extra latency
- **Cost**: Lambda free tier includes 1M requests/month; Bedrock charges per token (~$0.003/request)

## Judge-Ready One-Liner
"Our entire AI backend — Lambda, API Gateway, IAM, monitoring alarms — is defined in a single YAML template, deployable with one command, following infrastructure-as-code best practices."
