"""Publish a crate once and obtain verified release assets from crates.io."""

import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import time
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def fetch(url):
    request = Request(url, headers={"User-Agent": "mime-infer-release"})
    with urlopen(request, timeout=60) as response:
        return response.read()


def registry_version(version):
    try:
        data = fetch(f"https://crates.io/api/v1/crates/mime-infer/{version}")
    except HTTPError as error:
        if error.code == 404:
            return None
        raise
    return json.loads(data)["version"]


def verify_package(package, metadata, version, commit):
    if metadata["num"] != version:
        raise RuntimeError("Registry returned an unexpected version")
    checksum = hashlib.sha256(package).hexdigest()
    if checksum != metadata["checksum"]:
        raise RuntimeError("Registry package checksum mismatch")
    with tarfile.open(fileobj=io.BytesIO(package), mode="r:gz") as archive:
        member = archive.extractfile(f"mime-infer-{version}/.cargo_vcs_info.json")
        if member is None:
            raise RuntimeError("Registry package has no source commit information")
        source = json.load(member)
    if source["git"]["sha1"] != commit or source["git"].get("dirty", False):
        raise RuntimeError("This version was published from a different source commit")
    return checksum


def main():
    version, commit, destination = sys.argv[1:]
    metadata = registry_version(version)
    if metadata is None:
        if not os.environ.get("CARGO_REGISTRY_TOKEN"):
            raise RuntimeError("Configure the CARGO_REGISTRY_TOKEN Actions secret")
        subprocess.run(["cargo", "publish", "--locked"], check=True)
        for _ in range(12):
            metadata = registry_version(version)
            if metadata is not None:
                break
            time.sleep(5)
        if metadata is None:
            raise RuntimeError("Published version is not visible yet; rerun the workflow")

    package = fetch(f"https://crates.io/api/v1/crates/mime-infer/{version}/download")
    checksum = verify_package(package, metadata, version, commit)
    output = Path(destination)
    output.mkdir(parents=True, exist_ok=True)
    asset = output / f"mime-infer-{version}.crate"
    asset.write_bytes(package)
    asset.with_suffix(".crate.sha256").write_text(
        f"{checksum}  {asset.name}\n", encoding="utf-8"
    )
    print(f"Verified mime-infer {version} from {commit}")


if __name__ == "__main__":
    main()
