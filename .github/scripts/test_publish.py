"""Regression checks for release identity and safe retry behavior."""

import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import publish


class PublishTests(unittest.TestCase):
    version = "4.1.0"
    commit = "0123456789abcdef0123456789abcdef01234567"

    def package(self, dirty=False):
        source = json.dumps({"git": {"sha1": self.commit, "dirty": dirty}}).encode()
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:gz") as archive:
            member = tarfile.TarInfo(f"mime-infer-{self.version}/.cargo_vcs_info.json")
            member.size = len(source)
            archive.addfile(member, io.BytesIO(source))
        package = stream.getvalue()
        metadata = {"num": self.version, "checksum": hashlib.sha256(package).hexdigest()}
        return package, metadata

    def test_accepts_the_tagged_package(self):
        package, metadata = self.package()
        self.assertEqual(
            publish.verify_package(package, metadata, self.version, self.commit),
            metadata["checksum"],
        )

    def test_rejects_changed_package_bytes(self):
        package, metadata = self.package()
        with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
            publish.verify_package(package + b"changed", metadata, self.version, self.commit)

    def test_rejects_another_source_commit(self):
        package, metadata = self.package()
        with self.assertRaisesRegex(RuntimeError, "different source commit"):
            publish.verify_package(package, metadata, self.version, "f" * 40)

    def test_rejects_dirty_source(self):
        package, metadata = self.package(dirty=True)
        with self.assertRaisesRegex(RuntimeError, "different source commit"):
            publish.verify_package(package, metadata, self.version, self.commit)

    def test_rejects_another_version(self):
        package, metadata = self.package()
        with self.assertRaisesRegex(RuntimeError, "unexpected version"):
            publish.verify_package(package, metadata, "4.2.0", self.commit)

    def test_retry_uses_existing_registry_package_without_publishing(self):
        package, metadata = self.package()
        with tempfile.TemporaryDirectory() as destination:
            arguments = ["publish.py", self.version, self.commit, destination]
            with (
                patch("sys.argv", arguments),
                patch.object(publish, "registry_version", return_value=metadata),
                patch.object(publish, "fetch", return_value=package),
                patch.object(publish.subprocess, "run") as run,
            ):
                publish.main()
            run.assert_not_called()
            asset = Path(destination) / f"mime-infer-{self.version}.crate"
            self.assertEqual(asset.read_bytes(), package)
            self.assertEqual(
                asset.with_suffix(".crate.sha256").read_text(encoding="utf-8"),
                f"{metadata['checksum']}  {asset.name}\n",
            )


if __name__ == "__main__":
    unittest.main()
