#!/usr/bin/env python3
"""Check all packages for version mismatches and other issues"""

import os
import re
from pathlib import Path

# Expected versions from workspace Cargo.toml
WORKSPACE_VERSIONS = {
    "serde": "1.0",
    "serde_json": "1.0",
    "bincode": "1.3",
    "sha2": "0.10",
    "hex": "0.4",
    "reqwest": "0.11",
    "rdkafka": "0.36",
    "tonic": "0.10",
    "tonic-build": "0.10",
    "prost": "0.12",
    "prost-types": "0.12",
    "oxigraph": "0.5",
    "ahash": "0.8",
    "tera": "1.19",
    "anyhow": "1.0",
    "miette": "7.6",
    "regorus": "0.4",
    "rand": "0.8",
    "uuid": "1.0",
    "chrono": "0.4",
    "rayon": "1.11",
    "futures": "0.3",
    "criterion": "0.5",
    "proptest": "1.0",
    "tempfile": "3.10",
    "tokio-stream": "0.1",
    "tokio-test": "0.4",
    "tracing": "0.1",
    "tracing-subscriber": "0.3",
    "tracing-opentelemetry": "0.32",
    "opentelemetry-semantic-conventions": "0.15",
    "toml": "0.8",
    "sled": "0.34",
    "git2": "0.18",
    "clap": "4.5",
    "clap-noun-verb": "3.4.0",
    "lru": "0.16",
    "blake3": "1.8",
    "thiserror": "2.0",
    "tokio": "1.48",
    "opentelemetry": "0.31",
    "opentelemetry_sdk": "0.31",
    "opentelemetry-otlp": "0.31",
    "opentelemetry-http": "0.31",
}

packages = [
    "knhk-hot", "knhk-otel", "knhk-connectors", "knhk-lockchain",
    "knhk-unrdf", "knhk-etl", "knhk-warm", "knhk-aot",
    "knhk-validation", "knhk-config", "knhk-sidecar", "knhk-cli",
    "knhk-integration-tests"
]

mismatches = []
metadata_issues = []

for pkg in packages:
    pkg_file = Path(pkg) / "Cargo.toml"
    if not pkg_file.exists():
        continue
    
    with open(pkg_file) as f:
        content = f.read()
        lines = content.split('\n')
        
        # Check metadata
        name = None
        version = None
        edition = None
        
        for line in lines:
            if line.strip().startswith('name ='):
                name = line.split('=')[1].strip().strip('"')
            elif line.strip().startswith('version ='):
                version = line.split('=')[1].strip().strip('"')
            elif line.strip().startswith('edition ='):
                edition = line.split('=')[1].strip().strip('"')
        
        if edition and edition != "2021":
            metadata_issues.append((pkg, f"edition = {edition} (expected 2021)"))
        
        # Check dependency versions
        for dep_name, expected_ver in WORKSPACE_VERSIONS.items():
            # Look for dependency declarations (handle both quoted and unquoted)
            patterns = [
                rf'{re.escape(dep_name)}\s*=\s*["\']([^"\']+)["\']',
                rf'{re.escape(dep_name)}\s*=\s*{{[^}}]*version\s*=\s*["\']([^"\']+)["\']',
            ]
            
            for pattern in patterns:
                matches = re.findall(pattern, content)
                for match in matches:
                    actual_ver = match
                    if actual_ver != expected_ver:
                        mismatches.append((pkg, dep_name, actual_ver, expected_ver))
                        break

print("=== VERSION MISMATCHES ===")
if mismatches:
    for pkg, dep, actual, expected in sorted(set(mismatches)):
        print(f"{pkg}: {dep} = {actual} (expected {expected})")
else:
    print("No version mismatches found!")

print("\n=== METADATA ISSUES ===")
if metadata_issues:
    for pkg, issue in metadata_issues:
        print(f"{pkg}: {issue}")
else:
    print("All packages have correct metadata!")

