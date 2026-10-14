#!/usr/bin/env python3
"""Build the MerkleForge website (Vocs, in docs/) plus Criterion reports into one Pages artifact."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tomllib
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DOCS_DIR = ROOT / "docs"
PUBLIC_DIR = DOCS_DIR / "public"
VOCS_OUTPUT = DOCS_DIR / "dist" / "public"


def load_json(path: Path) -> dict[str, Any]:
    """Read a JSON object from disk."""
    return json.loads(path.read_text())


def workspace_version(manifest_path: Path) -> str:
    """Read the shared package version from the workspace manifest."""
    manifest = tomllib.loads(manifest_path.read_text())
    return str(manifest["workspace"]["package"]["version"])


def parse_size(value: Any) -> float:
    """Return a sortable numeric benchmark input size."""
    try:
        return float(str(value).replace(",", ""))
    except (TypeError, ValueError):
        return 0


def benchmark_identity(benchmark: dict[str, Any]) -> tuple[str, str, str]:
    """Normalize Criterion benchmark labels across grouped benchmark styles."""
    group_id = str(benchmark.get("group_id", "benchmark"))
    function_id = str(benchmark.get("function_id", "default"))
    group_parts = group_id.split("/")
    function_parts = function_id.split("/")

    if len(group_parts) > 1:
        tree_variant = group_parts[0]
        operation = group_parts[-1]
        algorithm = function_parts[0]
        if len(function_parts) > 1:
            operation = f"{operation}_{function_parts[-1]}"
    elif len(function_parts) > 1:
        tree_variant = group_id
        operation = function_parts[0]
        algorithm = function_parts[1]
    else:
        tree_variant = group_id
        operation = function_id
        algorithm = "Default"

    return tree_variant, operation, algorithm


def display_operation(operation: str) -> str:
    """Return a readable operation label for the benchmark dashboard."""
    labels = {
        "batch_update_batch_insert": "Batch insert",
        "batch_update_sequential_insert": "Sequential insert",
        "construction": "Construction",
        "insert": "Insert",
        "non_membership_generation": "Non-membership proof",
        "non_membership_verification": "Non-membership verify",
        "proof_generation": "Proof generation",
        "proof_verification": "Proof verification",
    }
    return labels.get(operation, operation.replace("_", " ").title())


def collect_benchmarks(criterion_dir: Path) -> list[dict[str, Any]]:
    """Collect completed Criterion estimates for the website dashboard."""
    benchmarks: list[dict[str, Any]] = []

    for benchmark_path in sorted(criterion_dir.glob("**/new/benchmark.json")):
        estimates_path = benchmark_path.with_name("estimates.json")
        if not estimates_path.exists():
            continue

        benchmark = load_json(benchmark_path)
        estimates = load_json(estimates_path)
        tree_variant, operation, algorithm = benchmark_identity(benchmark)
        mean = estimates["mean"]
        confidence = mean["confidence_interval"]
        throughput = benchmark.get("throughput") or {}
        elements = throughput.get("Elements")
        mean_ns = float(mean["point_estimate"])

        benchmarks.append(
            {
                "id": str(benchmark["full_id"]),
                "tree_variant": tree_variant.replace("_", " ").title(),
                "operation": display_operation(operation),
                "operation_key": operation,
                "algorithm": algorithm,
                "size": str(benchmark.get("value_str", "n/a")),
                "size_value": parse_size(benchmark.get("value_str")),
                "mean_ns": mean_ns,
                "lower_ns": float(confidence["lower_bound"]),
                "upper_ns": float(confidence["upper_bound"]),
                "elements_per_second": (
                    float(elements) * 1_000_000_000 / mean_ns
                    if elements is not None and mean_ns > 0
                    else None
                ),
                "report": (
                    "reports/criterion/"
                    + str(benchmark["directory_name"])
                    + "/report/index.html"
                ),
            }
        )

    return benchmarks


def write_json_asset(path: Path, payload: dict[str, Any]) -> None:
    """Write a JSON file the site fetches at runtime."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, separators=(",", ":")))


def run_vocs_build(base_path: str) -> None:
    """Compile the Vocs site for the requested base path."""
    env = os.environ.copy()
    env["VOCS_BASE_PATH"] = base_path
    subprocess.run(["npm", "run", "build"], cwd=DOCS_DIR, check=True, env=env)


def copy_criterion_reports(criterion_dir: Path, output_dir: Path) -> None:
    """Copy generated Criterion HTML reports into the built site."""
    if not criterion_dir.exists():
        return
    reports_dir = output_dir / "reports" / "criterion"
    reports_dir.parent.mkdir(parents=True, exist_ok=True)
    if reports_dir.exists():
        shutil.rmtree(reports_dir)
    shutil.copytree(criterion_dir, reports_dir)


def build_site(
    output_dir: Path,
    criterion_dir: Path,
    repository_url: str,
    commit_sha: str,
    crate_version: str,
    minimum_samples: int,
    base_path: str,
) -> int:
    """Build the site and return the number of benchmark samples."""
    benchmarks = collect_benchmarks(criterion_dir)
    if len(benchmarks) < minimum_samples:
        raise SystemExit(
            f"Expected at least {minimum_samples} completed benchmark samples, "
            f"found {len(benchmarks)}"
        )

    commit = commit_sha[:7] if commit_sha else "local"
    generated_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    write_json_asset(
        PUBLIC_DIR / "benchmark-data.json",
        {"benchmarks": benchmarks, "generatedAt": generated_at, "commit": commit},
    )
    write_json_asset(
        PUBLIC_DIR / "site-data.json",
        {"crateVersion": crate_version, "repositoryUrl": repository_url, "commit": commit},
    )

    run_vocs_build(base_path)

    if output_dir.exists():
        shutil.rmtree(output_dir)
    shutil.copytree(VOCS_OUTPUT, output_dir)
    copy_criterion_reports(criterion_dir, output_dir)
    (output_dir / ".nojekyll").touch()
    return len(benchmarks)


def main() -> None:
    """Parse command-line arguments and build the site."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--criterion-dir", type=Path, default=ROOT / "target" / "criterion")
    parser.add_argument("--output", type=Path, default=ROOT / "_site")
    parser.add_argument("--minimum-samples", type=int, default=0)
    parser.add_argument(
        "--repository-url",
        default=(
            os.environ.get("GITHUB_SERVER_URL", "https://github.com")
            + "/"
            + os.environ.get("GITHUB_REPOSITORY", "1888-Labs/MerkleForge")
        ),
    )
    parser.add_argument("--commit", default=os.environ.get("GITHUB_SHA", ""))
    parser.add_argument(
        "--base-path",
        default=os.environ.get("MERKLEFORGE_SITE_BASE", "/MerkleForge"),
    )
    parser.add_argument(
        "--crate-version",
        default=workspace_version(ROOT / "Cargo.toml"),
    )
    args = parser.parse_args()

    sample_count = build_site(
        args.output,
        args.criterion_dir,
        args.repository_url,
        args.commit,
        args.crate_version,
        args.minimum_samples,
        args.base_path,
    )
    print(f"Generated {args.output} with {sample_count} benchmark samples")


if __name__ == "__main__":
    main()
