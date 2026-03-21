"""
Tests for the Google Drive publisher (v1.0).
All Drive API calls and credentials are mocked — no real service account required.
"""

from __future__ import annotations

import base64
import json
import os
import pytest
from pathlib import Path
from unittest.mock import MagicMock, patch


# ---------------------------------------------------------------------------
# _get_credentials
# ---------------------------------------------------------------------------

def test_get_credentials_raises_without_env_var():
    from forma.publisher.google_drive import _get_credentials

    with patch.dict(os.environ, {}, clear=True):
        # Remove the key if it exists
        env = {k: v for k, v in os.environ.items() if k != "GOOGLE_SERVICE_ACCOUNT_JSON"}
        with patch.dict(os.environ, env, clear=True):
            with pytest.raises(EnvironmentError, match="GOOGLE_SERVICE_ACCOUNT_JSON"):
                _get_credentials()


def _make_fake_sa_json() -> str:
    """Return a base64-encoded fake service account JSON."""
    data = {
        "type": "service_account",
        "project_id": "test-project",
        "private_key_id": "key-id",
        "private_key": "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Z3VS5JJcds3xHn/ygWep4PAtEsHAQ==\n-----END RSA PRIVATE KEY-----\n",
        "client_email": "test@test-project.iam.gserviceaccount.com",
        "client_id": "123456789",
        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
        "token_uri": "https://oauth2.googleapis.com/token",
    }
    return base64.b64encode(json.dumps(data).encode()).decode()


def test_get_credentials_reads_from_env():
    fake_b64 = _make_fake_sa_json()
    mock_creds = MagicMock()

    with patch.dict(os.environ, {"GOOGLE_SERVICE_ACCOUNT_JSON": fake_b64}):
        with patch("google.oauth2.service_account.Credentials.from_service_account_info", return_value=mock_creds) as mock_factory:
            from importlib import reload
            import forma.publisher.google_drive as module
            creds = module._get_credentials()

    assert mock_factory.called
    assert creds is mock_creds


# ---------------------------------------------------------------------------
# upload_file — create path
# ---------------------------------------------------------------------------

def _setup_drive_mock(existing_files=None):
    """Return a mock Drive service with configurable existing files."""
    mock_service = MagicMock()
    mock_files = mock_service.files.return_value

    # list() response
    mock_files.list.return_value.execute.return_value = {
        "files": existing_files or []
    }

    # create() response
    mock_files.create.return_value.execute.return_value = {"id": "new-file-id"}

    # update() response
    mock_files.update.return_value.execute.return_value = {}

    return mock_service


def test_upload_file_creates_new_file(tmp_path):
    from forma.publisher.google_drive import upload_file

    pdf = tmp_path / "slides.pdf"
    pdf.write_bytes(b"%PDF-1.4 test")

    mock_service = _setup_drive_mock(existing_files=[])
    fake_b64 = _make_fake_sa_json()
    mock_creds = MagicMock()

    with patch.dict(os.environ, {"GOOGLE_SERVICE_ACCOUNT_JSON": fake_b64}):
        with patch("google.oauth2.service_account.Credentials.from_service_account_info", return_value=mock_creds):
            with patch("googleapiclient.discovery.build", return_value=mock_service):
                file_id = upload_file(pdf, "folder-123", filename="slides.pdf")

    assert file_id == "new-file-id"
    mock_service.files.return_value.create.assert_called_once()
    mock_service.files.return_value.update.assert_not_called()


def test_upload_file_updates_existing_file(tmp_path):
    from forma.publisher.google_drive import upload_file

    pdf = tmp_path / "slides.pdf"
    pdf.write_bytes(b"%PDF-1.4 test")

    existing = [{"id": "existing-id", "name": "slides.pdf"}]
    mock_service = _setup_drive_mock(existing_files=existing)
    fake_b64 = _make_fake_sa_json()
    mock_creds = MagicMock()

    with patch.dict(os.environ, {"GOOGLE_SERVICE_ACCOUNT_JSON": fake_b64}):
        with patch("google.oauth2.service_account.Credentials.from_service_account_info", return_value=mock_creds):
            with patch("googleapiclient.discovery.build", return_value=mock_service):
                file_id = upload_file(pdf, "folder-123", filename="slides.pdf")

    assert file_id == "existing-id"
    mock_service.files.return_value.update.assert_called_once_with(
        fileId="existing-id",
        media_body=mock_service.files.return_value.update.call_args.kwargs["media_body"],
    )
    mock_service.files.return_value.create.assert_not_called()


def test_upload_file_uses_local_name_when_no_filename(tmp_path):
    from forma.publisher.google_drive import upload_file

    pdf = tmp_path / "myreport.pdf"
    pdf.write_bytes(b"%PDF-1.4 test")

    mock_service = _setup_drive_mock()
    fake_b64 = _make_fake_sa_json()
    mock_creds = MagicMock()

    with patch.dict(os.environ, {"GOOGLE_SERVICE_ACCOUNT_JSON": fake_b64}):
        with patch("google.oauth2.service_account.Credentials.from_service_account_info", return_value=mock_creds):
            with patch("googleapiclient.discovery.build", return_value=mock_service):
                upload_file(pdf, "folder-123")

    # list() should have been called with the local filename
    list_call_kwargs = mock_service.files.return_value.list.call_args
    assert "myreport.pdf" in str(list_call_kwargs)
