from pathlib import Path
import subprocess


MODEL_DIR = Path(__file__).resolve().parent
REPOSITORY_ROOT = MODEL_DIR.parents[2]
TLA_TOOLS = REPOSITORY_ROOT / "tools" / "tla2tools.jar"

if not TLA_TOOLS.is_file():
    raise SystemExit(f"TLA+ tools jar not found: {TLA_TOOLS}")

raise SystemExit(
    subprocess.run(
        [
            "java",
            "-XX:+UseParallelGC",
            "-cp",
            str(TLA_TOOLS),
            "tlc2.TLC",
            "-cleanup",
            "-config",
            "BoundedPipeline.cfg",
            "BoundedPipeline.tla",
        ],
        cwd=MODEL_DIR,
        check=False,
    ).returncode
)
