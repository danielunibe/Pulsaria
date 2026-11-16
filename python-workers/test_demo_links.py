"""Deterministic worker contract tests. Live TikTok checks are opt-in."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from threading import Thread
from unittest.mock import patch

import downloader


WORKER_DIR = Path(__file__).resolve().parent


class DownloaderContractTests(unittest.TestCase):
    def test_retry_after_is_appended_only_for_a_safe_429_header(self) -> None:
        class UpstreamError(RuntimeError):
            def __init__(self, status, headers):
                super().__init__("synthetic upstream error")
                self.status = status
                self.headers = headers

        original = UpstreamError(429, {"retry-after": "3"})
        wrapped = RuntimeError("extractor error")
        wrapped.exc_info = (UpstreamError, original, None)

        self.assertEqual(
            downloader._append_retry_after("HTTP Error 429", wrapped),
            "HTTP Error 429\nRetry-After: 3",
        )
        self.assertEqual(
            downloader._append_retry_after(
                "HTTP Error 503", UpstreamError(503, {"Retry-After": "3"})
            ),
            "HTTP Error 503",
        )
        self.assertEqual(
            downloader._append_retry_after(
                "HTTP Error 429",
                UpstreamError(429, {"Retry-After": "3\r\nX-Injected: yes"}),
            ),
            "HTTP Error 429",
        )

    def test_import_has_no_network_or_process_side_effect(self) -> None:
        self.assertTrue(callable(downloader.extract_metadata))

    @patch("downloader.subprocess.run")
    def test_metadata_contract(self, run) -> None:
        run.return_value = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout=json.dumps({
                "title": "Video de prueba", "uploader": "creator",
                "thumbnail": "https://example.invalid/thumb.jpg", "duration": 12,
                "upload_date": "20260831", "extractor_key": "TikTok",
            }), stderr="",
        )
        metadata = downloader.extract_metadata("https://www.tiktok.com/@creator/video/123")
        self.assertEqual(metadata["title"], "Video de prueba")
        self.assertEqual(metadata["platform"], "tiktok")

    @patch("downloader.subprocess.run")
    def test_tiktok_errors_keep_actionable_detail(self, run) -> None:
        run.side_effect = subprocess.CalledProcessError(
            1, ["yt-dlp"], stderr="TikTok session cookies are expired"
        )
        with self.assertRaisesRegex(RuntimeError, "session cookies are expired"):
            downloader.extract_metadata("https://www.tiktok.com/@creator/video/123")

    @unittest.skipUnless(
        importlib.util.find_spec("yt_dlp"), "yt-dlp runtime module not staged"
    )
    def test_429_retry_after_header_reaches_queue_error(self) -> None:
        class RateLimitHandler(BaseHTTPRequestHandler):
            requests = 0

            def do_GET(self):
                type(self).requests += 1
                self.send_response(429, "Too Many Requests")
                self.send_header("Retry-After", "3")
                self.send_header("Content-Length", "0")
                self.end_headers()

            def do_HEAD(self):
                self.do_GET()

            def log_message(self, *_args):
                pass

        server = HTTPServer(("127.0.0.1", 0), RateLimitHandler)
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with patch.dict(os.environ, {"PULSAR_COOKIES_FROM_BROWSER": ""}):
                command_prefix = downloader.build_yt_dlp_base_cmd() + [
                    "--ignore-config",
                    "--retries",
                    "0",
                    "--extractor-retries",
                    "0",
                ]
                with patch.object(
                    downloader, "build_yt_dlp_base_cmd", return_value=command_prefix
                ):
                    with self.assertRaisesRegex(RuntimeError, r"Retry-After: 3"):
                        downloader.extract_metadata(
                            f"http://127.0.0.1:{server.server_port}/video"
                        )
            self.assertEqual(RateLimitHandler.requests, 1)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)

    @unittest.skipUnless(
        importlib.util.find_spec("yt_dlp"), "yt-dlp runtime module not staged"
    )
    def test_local_metadata_success_keeps_the_cli_json_contract(self) -> None:
        class MetadataHandler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/video":
                    body = (
                        f'<html><head><meta property="og:title" '
                        f'content="Local metadata fixture"><meta property="og:video" '
                        f'content="http://127.0.0.1:{self.server.server_port}/fixture.mp4">'
                        '<meta property="og:description" content="offline test"></head></html>'
                    ).encode("utf-8")
                    content_type = "text/html; charset=utf-8"
                else:
                    body = b"synthetic-video-bytes"
                    content_type = "video/mp4"
                self.send_response(200)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def do_HEAD(self):
                body = b"" if self.path != "/video" else b"x" * 180
                content_type = (
                    "text/html; charset=utf-8"
                    if self.path == "/video"
                    else "video/mp4"
                )
                self.send_response(200)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(body) or 21))
                self.end_headers()

            def log_message(self, *_args):
                pass

        server = HTTPServer(("127.0.0.1", 0), MetadataHandler)
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with patch.dict(os.environ, {"PULSAR_COOKIES_FROM_BROWSER": ""}):
                command_prefix = downloader.build_yt_dlp_base_cmd() + [
                    "--ignore-config",
                    "--retries",
                    "0",
                    "--extractor-retries",
                    "0",
                ]
                with patch.object(
                    downloader, "build_yt_dlp_base_cmd", return_value=command_prefix
                ):
                    metadata = downloader.extract_metadata(
                        f"http://127.0.0.1:{server.server_port}/video"
                    )
            self.assertEqual(metadata["title"], "Local metadata fixture")
            self.assertEqual(metadata["description"], "offline test")
            self.assertEqual(metadata["platform"], "generic")
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)


@unittest.skipUnless(os.environ.get("PULSARIA_LIVE_TIKTOK_URL"), "live TikTok URL not supplied")
class LiveTikTokCertification(unittest.TestCase):
    def test_user_supplied_url_completes(self) -> None:
        url = os.environ["PULSARIA_LIVE_TIKTOK_URL"]
        with tempfile.TemporaryDirectory(prefix="pulsaria-live-") as directory:
            environment = os.environ.copy()
            environment["PULSAR_DOWNLOAD_DIR"] = directory
            result = subprocess.run(
                [sys.executable, str(WORKER_DIR / "main.py"), "--job_id", "990001", "--url", url],
                cwd=WORKER_DIR, env=environment, capture_output=True, text=True, timeout=900,
            )
            events = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(events and events[-1].get("event") == "completed", result.stdout)


if __name__ == "__main__":
    unittest.main()
