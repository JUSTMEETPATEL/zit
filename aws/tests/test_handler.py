"""Tests for zit AI Mentor Lambda handler."""

import json
import pytest
from unittest.mock import patch, MagicMock

# Import handler from lambda directory
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'lambda'))

from handler import (
    lambda_handler,
    validate_request,
    build_response,
    handle_commit_suggestion,
    VALID_REQUEST_TYPES,
)
from prompts import get_system_prompt, format_context


# ─── Validation Tests ───────────────────────────────────────────

class TestValidateRequest:
    def test_valid_explain_request(self):
        ok, msg = validate_request({"type": "explain"})
        assert ok is True
        assert msg == ""

    def test_valid_commit_suggestion(self):
        ok, msg = validate_request({"type": "commit_suggestion"})
        assert ok is True

    def test_invalid_type(self):
        ok, msg = validate_request({"type": "invalid_type"})
        assert ok is False
        assert "Invalid type" in msg

    def test_error_type_without_error_field(self):
        ok, msg = validate_request({"type": "error"})
        assert ok is False
        assert "'error' field" in msg

    def test_error_type_with_error_field(self):
        ok, msg = validate_request({"type": "error", "error": "fatal: not a repo"})
        assert ok is True

    def test_recommend_type_without_query(self):
        ok, msg = validate_request({"type": "recommend"})
        assert ok is False
        assert "'query' field" in msg

    def test_recommend_type_with_query(self):
        ok, msg = validate_request({"type": "recommend", "query": "undo last commit"})
        assert ok is True

    def test_default_type_is_explain(self):
        ok, msg = validate_request({})
        assert ok is True


# ─── Response Builder Tests ─────────────────────────────────────

class TestBuildResponse:
    def test_success_response(self):
        resp = build_response(200, True, data={"content": "hello"})
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["success"] is True
        assert body["response"]["content"] == "hello"
        assert "timestamp" in body
        assert "model" in body

    def test_error_response(self):
        resp = build_response(400, False, error="bad request")
        body = json.loads(resp["body"])
        assert body["success"] is False
        assert body["error"] == "bad request"

    def test_cors_headers(self):
        resp = build_response(200, True)
        assert resp["headers"]["Access-Control-Allow-Origin"] == "*"
        assert "Content-Type" in resp["headers"]


# ─── Health Check Tests ─────────────────────────────────────────

class TestHealthCheck:
    def test_health_endpoint(self):
        event = {"path": "/health", "httpMethod": "GET"}
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["response"]["status"] == "healthy"

    def test_health_endpoint_with_prefix(self):
        event = {"path": "/dev/health", "httpMethod": "GET"}
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200

    def test_options_preflight(self):
        event = {"path": "/mentor", "httpMethod": "OPTIONS"}
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200


# ─── Lambda Handler Tests ───────────────────────────────────────

class TestLambdaHandler:
    def test_invalid_json_body(self):
        event = {"path": "/mentor", "httpMethod": "POST", "body": "not json"}
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 400
        body = json.loads(resp["body"])
        assert "Invalid JSON" in body["error"]

    def test_invalid_request_type(self):
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({"type": "hack"})
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 400

    @patch("handler.invoke_bedrock")
    def test_explain_request(self, mock_bedrock):
        mock_bedrock.return_value = "Your repo is clean."
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({
                "type": "explain",
                "context": {"branch": "main"},
                "query": "What is happening?"
            })
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["success"] is True
        assert body["response"]["type"] == "explanation"
        assert body["response"]["content"] == "Your repo is clean."

    @patch("handler.invoke_bedrock")
    def test_commit_suggestion_request(self, mock_bedrock):
        mock_bedrock.return_value = "feat: add user authentication"
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({
                "type": "commit_suggestion",
                "context": {
                    "staged_files": ["src/auth.rs"],
                    "diff_stats": {"files_changed": 1, "insertions": 50, "deletions": 0},
                    "diff": "+pub fn login() {\n+    // OAuth flow\n+}"
                }
            })
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["response"]["type"] == "commit_suggestion"
        assert "feat:" in body["response"]["content"]

    @patch("handler.invoke_bedrock")
    def test_error_request(self, mock_bedrock):
        mock_bedrock.return_value = "You have uncommitted changes."
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({
                "type": "error",
                "error": "error: Your local changes would be overwritten",
                "context": {"branch": "main"}
            })
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["response"]["type"] == "error_explanation"
        assert body["response"]["original_error"] == "error: Your local changes would be overwritten"

    @patch("handler.invoke_bedrock")
    def test_learn_request(self, mock_bedrock):
        mock_bedrock.return_value = "A branch is like a parallel universe."
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({
                "type": "learn",
                "query": "what are branches?"
            })
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 200
        body = json.loads(resp["body"])
        assert body["response"]["type"] == "learning"

    @patch("handler.invoke_bedrock", side_effect=Exception("Bedrock timeout"))
    def test_bedrock_failure(self, mock_bedrock):
        event = {
            "path": "/mentor",
            "httpMethod": "POST",
            "body": json.dumps({"type": "explain", "context": {}})
        }
        resp = lambda_handler(event, None)
        assert resp["statusCode"] == 500
        body = json.loads(resp["body"])
        assert body["success"] is False
        assert "Internal server error" in body["error"]

    def test_dict_body_passthrough(self):
        """Test when API Gateway passes body as dict (test invocation)."""
        with patch("handler.invoke_bedrock", return_value="ok"):
            event = {
                "path": "/mentor",
                "httpMethod": "POST",
                "body": {"type": "explain", "context": {}}
            }
            resp = lambda_handler(event, None)
            assert resp["statusCode"] == 200


# ─── Prompt Tests ───────────────────────────────────────────────

class TestPrompts:
    def test_all_prompt_types_exist(self):
        for ptype in VALID_REQUEST_TYPES:
            prompt_key = "commit" if ptype == "commit_suggestion" else ptype
            prompt = get_system_prompt(prompt_key)
            assert len(prompt) > 50, f"Prompt for '{ptype}' is too short"

    def test_unknown_prompt_falls_back(self):
        prompt = get_system_prompt("nonexistent")
        default_prompt = get_system_prompt("explain")
        assert prompt == default_prompt

    def test_format_context_empty(self):
        result = format_context({})
        assert result == "No context provided"

    def test_format_context_full(self):
        ctx = {
            "branch": "main",
            "upstream": "origin/main",
            "ahead": 2,
            "behind": 1,
            "staged_files": ["file1.rs", "file2.rs"],
            "unstaged_files": ["README.md"],
            "has_conflicts": True,
            "detached_head": True,
            "recent_commits": ["abc1234 fix: typo", "def5678 feat: login"]
        }
        result = format_context(ctx)
        assert "main" in result
        assert "origin/main" in result
        assert "+2/-1" in result
        assert "file1.rs" in result
        assert "MERGE CONFLICTS" in result
        assert "DETACHED HEAD" in result
        assert "fix: typo" in result

    def test_format_context_partial(self):
        result = format_context({"branch": "develop"})
        assert "develop" in result
        assert "CONFLICTS" not in result
