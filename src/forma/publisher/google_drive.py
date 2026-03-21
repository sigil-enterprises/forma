"""
Google Drive publisher.

Uploads rendered artifacts to a Drive folder using a service account.
Credentials are read from GOOGLE_SERVICE_ACCOUNT_JSON (base64-encoded JSON)
— never written to disk.
"""

from __future__ import annotations

import base64
import json
import os
from pathlib import Path

from rich.console import Console

console = Console()


def _get_credentials():
    from google.oauth2 import service_account

    raw = os.environ.get("GOOGLE_SERVICE_ACCOUNT_JSON")
    if not raw:
        raise OSError(
            "GOOGLE_SERVICE_ACCOUNT_JSON is not set. "
            "Set it to the base64-encoded service account JSON."
        )

    decoded = base64.b64decode(raw)
    info = json.loads(decoded)

    return service_account.Credentials.from_service_account_info(
        info,
        scopes=["https://www.googleapis.com/auth/drive"],
    )


def upload_file(
    local_path: Path,
    folder_id: str,
    *,
    filename: str | None = None,
    mime_type: str = "application/pdf",
) -> str:
    """
    Upload a file to Google Drive, replacing any existing file with the same name.
    Returns the Drive file ID.
    """
    from googleapiclient.discovery import build
    from googleapiclient.http import MediaFileUpload

    credentials = _get_credentials()
    service = build("drive", "v3", credentials=credentials)

    name = filename or local_path.name

    # Check if a file with this name already exists in the folder
    response = (
        service.files()
        .list(
            q=f"name='{name}' and '{folder_id}' in parents and trashed=false",
            fields="files(id, name)",
        )
        .execute()
    )
    existing = response.get("files", [])

    media = MediaFileUpload(str(local_path), mimetype=mime_type, resumable=True)

    if existing:
        file_id = existing[0]["id"]
        service.files().update(fileId=file_id, media_body=media).execute()
        console.print(f"[green]↑ Updated[/green] {name} in Drive ({folder_id})")
    else:
        metadata = {"name": name, "parents": [folder_id]}
        result = (
            service.files()
            .create(body=metadata, media_body=media, fields="id")
            .execute()
        )
        file_id = result["id"]
        console.print(f"[green]↑ Uploaded[/green] {name} to Drive ({folder_id})")

    return file_id
