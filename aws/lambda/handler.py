import json
import boto3
import os
import logging
from datetime import datetime, timezone
from prompts import get_system_prompt, format_context

# Setup logging
logger = logging.getLogger()
logger.setLevel(os.environ.get('LOG_LEVEL', 'INFO'))

# Initialize Bedrock client (lazy — created on first use to reduce cold start if health-checked)
_bedrock_client = None

def get_bedrock_client():
    global _bedrock_client
    if _bedrock_client is None:
        _bedrock_client = boto3.client(
            service_name='bedrock-runtime',
            region_name=os.environ.get('AWS_REGION', 'ap-south-1')
        )
    return _bedrock_client

MODEL_ID = os.environ.get('BEDROCK_MODEL_ID', 'anthropic.claude-3-sonnet-20240229-v1:0')
MAX_DIFF_LENGTH = 4000  # Limit diff content to avoid token explosion
MAX_REQUEST_BODY_SIZE = 128_000  # 128KB max request body
VALID_REQUEST_TYPES = ['explain', 'error', 'recommend', 'commit_suggestion', 'learn', 'review', 'merge_resolve', 'merge_strategy']


def validate_request(body: dict) -> tuple:
    """Validate incoming request."""
    request_type = body.get('type', 'explain')
    
    if request_type not in VALID_REQUEST_TYPES:
        return False, f"Invalid type '{request_type}'. Must be one of: {VALID_REQUEST_TYPES}"
    
    if request_type == 'error' and not body.get('error'):
        return False, "Error type requires 'error' field with the Git error message"
    
    if request_type == 'recommend' and not body.get('query'):
        return False, "Recommend type requires 'query' field describing what you want to do"
    
    return True, ""


def build_response(status_code: int, success: bool, data: dict = None, error: str = None) -> dict:
    """Build standardized API response."""
    body = {
        'success': success,
        'timestamp': datetime.now(timezone.utc).isoformat(),
        'model': MODEL_ID
    }
    
    if success and data:
        body['response'] = data
    elif error:
        body['error'] = error
    
    return {
        'statusCode': status_code,
        'headers': {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Headers': 'Content-Type,Authorization,x-api-key',
            'Access-Control-Allow-Methods': 'GET,POST,OPTIONS'
        },
        'body': json.dumps(body)
    }


def lambda_handler(event, context):
    """Main Lambda handler for AI Mentor requests."""
    try:
        path = event.get('path', '')
        http_method = event.get('httpMethod', '')
        
        logger.info(f"Request: {http_method} {path}")
        
        # Handle health check
        if path == '/health' or path.endswith('/health'):
            return build_response(200, True, data={
                'status': 'healthy',
                'model': MODEL_ID,
                'version': '1.0.0'
            })
        
        # Handle OPTIONS preflight
        if http_method == 'OPTIONS':
            return build_response(200, True, {'message': 'OK'})
        
        # Check request body size
        raw_body = event.get('body', '')
        if isinstance(raw_body, str) and len(raw_body) > MAX_REQUEST_BODY_SIZE:
            logger.warning(f"Request body too large: {len(raw_body)} bytes")
            return build_response(413, False, error=f"Request body too large (max {MAX_REQUEST_BODY_SIZE // 1024}KB)")

        # Parse request body
        if isinstance(raw_body, str):
            body = json.loads(raw_body)
        else:
            body = raw_body if raw_body else event
        
        logger.info(f"Request type: {body.get('type', 'explain')}")
        
        # Validate request
        is_valid, error_msg = validate_request(body)
        if not is_valid:
            logger.warning(f"Invalid request: {error_msg}")
            return build_response(400, False, error=error_msg)
        
        request_type = body.get('type', 'explain')
        repo_context = body.get('context', {})
        user_query = body.get('query', '')
        error_message = body.get('error', '')
        
        # Build the prompt based on request type
        if request_type == 'explain':
            response = handle_explain(repo_context, user_query)
        elif request_type == 'error':
            response = handle_error(error_message, repo_context)
        elif request_type == 'recommend':
            response = handle_recommend(repo_context, user_query)
        elif request_type == 'commit_suggestion':
            response = handle_commit_suggestion(repo_context)
        elif request_type == 'learn':
            response = handle_learn(repo_context, user_query)
        elif request_type == 'review':
            response = handle_review(repo_context, user_query)
        elif request_type == 'merge_resolve':
            response = handle_merge_resolve(repo_context, user_query)
        elif request_type == 'merge_strategy':
            response = handle_merge_strategy(repo_context, user_query)
        else:
            response = handle_explain(repo_context, user_query)
        
        logger.info(f"Request processed successfully: {request_type}")
        return build_response(200, True, data=response)
        
    except json.JSONDecodeError as e:
        logger.error(f"JSON decode error: {str(e)}")
        return build_response(400, False, error="Invalid JSON in request body")
        
    except Exception as e:
        logger.error(f"Error processing request: {str(e)}", exc_info=True)
        return build_response(500, False, error="Internal server error. Please try again later.")


def invoke_bedrock(system_prompt: str, user_message: str) -> str:
    """Invoke Amazon Bedrock with Claude model."""
    logger.info(f"Invoking Bedrock model: {MODEL_ID}")
    
    client = get_bedrock_client()
    
    request_body = {
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 1024,
        "temperature": 0.7,
        "system": system_prompt,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ]
    }
    
    response = client.invoke_model(
        modelId=MODEL_ID,
        contentType='application/json',
        accept='application/json',
        body=json.dumps(request_body)
    )
    
    response_body = json.loads(response['body'].read())
    
    # Log token usage for cost tracking
    usage = response_body.get('usage', {})
    logger.info(f"Token usage - Input: {usage.get('input_tokens', 0)}, Output: {usage.get('output_tokens', 0)}")
    
    return response_body['content'][0]['text']


def invoke_bedrock_stream(system_prompt: str, user_message: str) -> str:
    """Invoke Amazon Bedrock with streaming for faster first token."""
    logger.info(f"Invoking Bedrock model (streaming): {MODEL_ID}")
    
    client = get_bedrock_client()
    
    request_body = {
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 1024,
        "temperature": 0.7,
        "system": system_prompt,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ]
    }
    
    response = client.invoke_model_with_response_stream(
        modelId=MODEL_ID,
        contentType='application/json',
        accept='application/json',
        body=json.dumps(request_body)
    )
    
    full_response = ""
    for event in response['body']:
        chunk = json.loads(event['chunk']['bytes'])
        if chunk['type'] == 'content_block_delta':
            full_response += chunk['delta'].get('text', '')
    
    return full_response


def handle_explain(repo_context: dict, query: str) -> dict:
    """Handle repository state explanation requests."""
    system_prompt = get_system_prompt('explain')
    context_str = format_context(repo_context)
    
    user_message = f"""
Repository Context:
{context_str}

User Question: {query if query else "Explain the current repository state."}
"""
    
    explanation = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'explanation',
        'content': explanation
    }


def handle_error(error_message: str, repo_context: dict) -> dict:
    """Handle Git error translation requests."""
    system_prompt = get_system_prompt('error')
    context_str = format_context(repo_context)
    
    user_message = f"""
Git Error Message:
{error_message}

Repository Context:
{context_str}

Please explain this error and suggest how to fix it.
"""
    
    explanation = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'error_explanation',
        'original_error': error_message,
        'content': explanation
    }


def handle_recommend(repo_context: dict, query: str) -> dict:
    """Handle operation recommendation requests."""
    system_prompt = get_system_prompt('recommend')
    context_str = format_context(repo_context)
    
    user_message = f"""
Repository Context:
{context_str}

User wants to: {query}

Recommend the safest approach.
"""
    
    recommendation = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'recommendation',
        'content': recommendation
    }


def handle_commit_suggestion(repo_context: dict) -> dict:
    """Handle commit message suggestion requests."""
    system_prompt = get_system_prompt('commit')
    
    staged_files = repo_context.get('staged_files', [])
    diff_stats = repo_context.get('diff_stats', {})
    diff_content = repo_context.get('diff', '')
    
    user_message = f"""
Staged Files: {', '.join(staged_files) if staged_files else 'None specified'}
Diff Statistics:
- Files changed: {diff_stats.get('files_changed', len(staged_files))}
- Insertions: {diff_stats.get('insertions', 0)}
- Deletions: {diff_stats.get('deletions', 0)}
{f"Diff Preview: {diff_content[:MAX_DIFF_LENGTH]}..." if diff_content else ""}

Suggest a concise, conventional commit message.
"""
    
    suggestion = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'commit_suggestion',
        'content': suggestion
    }


def handle_learn(repo_context: dict, topic: str) -> dict:
    """Handle Git learning/tutorial requests."""
    system_prompt = get_system_prompt('learn')
    context_str = format_context(repo_context)
    
    user_message = f"""
Repository Context:
{context_str}

Topic to learn about: {topic if topic else "basic Git workflow"}

Provide a beginner-friendly explanation with practical examples.
"""
    
    explanation = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'learning',
        'topic': topic,
        'content': explanation
    }


def handle_review(repo_context: dict, query: str) -> dict:
    """Handle code diff review requests."""
    system_prompt = get_system_prompt('review')
    context_str = format_context(repo_context)
    
    diff_content = repo_context.get('diff', '')
    staged_files = repo_context.get('staged_files', [])
    
    user_message = f"""
Repository Context:
{context_str}

Files Under Review: {', '.join(staged_files) if staged_files else 'Unknown'}

Diff Content:
{diff_content[:MAX_DIFF_LENGTH] if diff_content else 'No diff provided'}

{f"Reviewer Notes: {query}" if query else "Review this diff for issues and improvements."}
"""
    
    review = invoke_bedrock_stream(system_prompt, user_message)
    
    return {
        'type': 'review',
        'content': review
    }


def handle_merge_resolve(repo_context: dict, query: str) -> dict:
    """Handle merge conflict resolution requests."""
    system_prompt = get_system_prompt('merge_resolve')
    context_str = format_context(repo_context)
    
    conflict_diff = repo_context.get('conflict_diff', '')
    conflict_files = repo_context.get('conflict_files', [])
    
    user_message = f"""
Repository Context:
{context_str}

Conflicted Files: {', '.join(conflict_files) if conflict_files else 'Unknown'}

Conflict Content (with markers):
{conflict_diff[:MAX_DIFF_LENGTH] if conflict_diff else 'No conflict content provided'}

{f"Developer Notes: {query}" if query else "Analyze this merge conflict and recommend the best resolution."}
"""
    
    resolution = invoke_bedrock_stream(system_prompt, user_message)
    
    return {
        'type': 'merge_resolution',
        'content': resolution
    }


def handle_merge_strategy(repo_context: dict, query: str) -> dict:
    """Handle merge strategy recommendation requests."""
    system_prompt = get_system_prompt('merge_strategy')
    context_str = format_context(repo_context)
    
    user_message = f"""
Repository Context:
{context_str}

{f"Developer Question: {query}" if query else "What is the safest strategy to integrate these branches?"}

Please recommend the best merge/rebase strategy given the current repository state.
"""
    
    recommendation = invoke_bedrock(system_prompt, user_message)
    
    return {
        'type': 'merge_strategy',
        'content': recommendation
    }
