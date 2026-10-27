import sys
import tempfile
from pathlib import Path
import unittest

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / ".github" / "scripts"))

import prepare_release


class UpdateDependencyVersionsTests(unittest.TestCase):
    def bump(self, text: str) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "README.md"
            path.write_text(text)
            prepare_release.update_dependency_versions([path], "0.5.0")
            return path.read_text()

    def test_bumps_install_snippets_and_workspace_pins(self) -> None:
        text = (
            'merkle-core = "0.4.1"\n'
            'merkleforge-hash = "0.4.1"\n'
            'merkle-variants = "0.4.1"\n'
            'merkle-core = { version = "0.4.1", default-features = false, features = ["alloc"] }\n'
            'merkleforge-hash = { path = "merkle-hash", version = "0.4.1" }\n'
        )
        self.assertEqual(self.bump(text), text.replace("0.4.1", "0.5.0"))

    def test_leaves_historical_mentions_and_other_crates_alone(self) -> None:
        text = (
            "`BinaryMerkleTree::from_leaves` is new since 0.4.1.\n"
            "### 0.4.1 (2026-09-05)\n"
            'sha2 = { version = "0.10.9", default-features = false }\n'
        )
        self.assertEqual(self.bump(text), text)


if __name__ == "__main__":
    unittest.main()
