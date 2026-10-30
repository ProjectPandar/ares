#!/usr/bin/env python3
"""Transfer in-run Native outputs without duplicating signed Nixpkgs closures."""

import argparse
import json
import lzma
import shutil
import subprocess
from pathlib import Path


OFFICIAL_CACHE = "https://cache.nixos.org"


def path_info(paths):
    return json.loads(subprocess.check_output([
        "nix", "path-info", "--json", "--json-format", "1", *paths,
    ], text=True))


def export(cache, roots):
    closure = path_info(["--recursive", *roots])
    native = {
        path: info["narHash"]
        for path, info in closure.items()
        if path in roots or not any(
            signature.startswith("cache.nixos.org-1:")
            for signature in info["signatures"]
        )
    }
    cache.mkdir(parents=True, exist_ok=True)
    # A binary-cache copy requires every reference to be present in that cache.
    # Store exports let the signed dependencies remain with their own authority.
    with subprocess.Popen([
        "nix-store", "--export", *sorted(native),
    ], stdout=subprocess.PIPE) as process:
        with lzma.open(cache / "native.export.xz", "wb", preset=6) as archive:
            shutil.copyfileobj(process.stdout, archive, 1024 * 1024)
        if process.wait() != 0:
            raise RuntimeError("Native store export failed")
    (cache / "manifest.json").write_text(json.dumps({
        "roots": roots,
        "native": native,
        "official": sorted(closure.keys() - native.keys()),
    }, indent=2) + "\n")
    print(f"Native cache: {len(native)} paths, "
          f"{(cache / 'native.export.xz').stat().st_size} compressed bytes")


def restore(cache, variants):
    manifests = {
        variant: json.loads((cache / variant / "manifest.json").read_text())
        for variant in variants
    }
    official = sorted({
        path for manifest in manifests.values() for path in manifest["official"]
    })
    if official:
        subprocess.run([
            "nix", "build", "--no-link", "--max-jobs", "0",
            "--substituters", OFFICIAL_CACHE, *official,
        ], check=True)
    for variant, manifest in manifests.items():
        with subprocess.Popen([
            "nix-store", "--import",
        ], stdin=subprocess.PIPE) as process:
            with lzma.open(cache / variant / "native.export.xz", "rb") as archive:
                shutil.copyfileobj(archive, process.stdin, 1024 * 1024)
            process.stdin.close()
            if process.wait() != 0:
                raise RuntimeError(f"Native store import failed: {variant}")
        actual = path_info(list(manifest["native"]))
        for path, expected in manifest["native"].items():
            if actual[path]["narHash"] != expected:
                raise RuntimeError(f"Native cache hash mismatch: {path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    publish = commands.add_parser("export")
    publish.add_argument("cache", type=Path)
    publish.add_argument("roots", nargs="+")
    consume = commands.add_parser("import")
    consume.add_argument("cache", type=Path)
    consume.add_argument("variants", help="JSON array of producer variants")
    args = parser.parse_args()
    match args.command:
        case "export":
            export(args.cache, args.roots)
        case "import":
            restore(args.cache, json.loads(args.variants))
