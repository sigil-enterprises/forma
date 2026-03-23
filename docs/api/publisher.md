# Publisher API Reference

The `forma.publisher` package uploads rendered artifacts to Google Drive using a service account. Credentials are never written to disk.

---

## upload_file()

```python
def upload_file(
    local_path: Path,
    folder_id: str,
    *,
    filename: str | None = None,
    mime_type: str = "application/pdf",
) -> str
```

Upload a file to a Google Drive folder. If a file with the same name already exists in the folder, it is updated in place (preserving its Drive file ID and share links). If no such file exists, a new file is created.

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `local_path` | Path to the local file to upload. |
| `folder_id` | Google Drive folder ID. Found in the folder's URL: `drive.google.com/drive/folders/<folder_id>`. |
| `filename` | Override the uploaded filename. Defaults to `local_path.name`. |
| `mime_type` | MIME type of the file. Defaults to `"application/pdf"`. |

**Returns:** the Drive file ID of the created or updated file.

**Raises:** `OSError` if `GOOGLE_SERVICE_ACCOUNT_JSON` is not set.

```python
from forma.publisher.google_drive import upload_file
from pathlib import Path

file_id = upload_file(
    Path("var/builds/acme/slides.pdf"),
    folder_id="1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs",
    filename="ACME-Proposal-Slides.pdf",
)
```

---

## Credentials

`GOOGLE_SERVICE_ACCOUNT_JSON` must be set to the **base64-encoded** service account JSON. Credentials are decoded in memory and passed directly to `google-auth` — no temporary file is written to disk.

To encode a service account key:

```bash
base64 -w0 service-account.json
```

Set the scope to `https://www.googleapis.com/auth/drive` when creating the service account key. The service account must be granted Editor access to the target Drive folder.

The `_get_credentials()` helper is an internal function that handles decoding and instantiating `google.oauth2.service_account.Credentials`. It is not part of the public API.

---

## Security notes

- **No disk writes:** credentials are decoded from the environment variable directly into a Python dict and passed to `service_account.Credentials.from_service_account_info()`. The JSON key file never touches the filesystem during upload.
- **Upsert semantics:** the uploader queries the folder before uploading. If a file with the same name exists, it calls `files().update()` to replace the content while keeping the same Drive file ID — existing share links continue to work.
- **Resumable uploads:** `MediaFileUpload` is instantiated with `resumable=True`, which is safe for large PDFs and allows the Drive API to retry partial uploads.
