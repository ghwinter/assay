import subprocess
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RepositoryBoundaryTests(unittest.TestCase):
    def test_external_research_inputs_remain_local(self):
        result = subprocess.run(
            [
                "git",
                "check-ignore",
                "--no-index",
                "pdf/example-paper.pdf",
                "docs/CLI_REFERENCE_NOTE.md",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        )

        self.assertEqual(
            set(result.stdout.splitlines()),
            {"pdf/example-paper.pdf", "docs/CLI_REFERENCE_NOTE.md"},
        )

    def test_shared_tool_configs_are_portable(self):
        config_paths = [
            ROOT / "tools" / "config.toml",
            ROOT / "examples" / "rust-service" / "tools" / "config.toml",
        ]

        for config_path in config_paths:
            with self.subTest(config=config_path.relative_to(ROOT)):
                with config_path.open("rb") as config_file:
                    tools = tomllib.load(config_file)["tools"]
                self.assertEqual(tools["python"], "python")
                self.assertEqual(tools["verus"], "verus")
                self.assertEqual(tools["cdb"], "cdb")


if __name__ == "__main__":
    unittest.main()
