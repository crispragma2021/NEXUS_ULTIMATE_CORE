#!/usr/bin/env python3
import os
import sys
import json
import platform
import subprocess
from pathlib import Path

def get_cpu_jobs():
    """Detects CPU cores and limits threads for Intel Core i7-12700F stability."""
    try:
        cores = os.cpu_count() or 4
        # Limit to 14 jobs (limit set by Architect for compilation) or standard cores
        return min(14, max(1, cores))
    except Exception:
        return 4

def compile_workspace(jobs):
    """Compiles all Rust workspace members in release mode."""
    print(f"🧬 [PORTABLE SETUP] Compiling workspace with -j {jobs} jobs...")
    try:
        # Set optimization flags
        env = os.environ.copy()
        env["RUSTFLAGS"] = "-C target-cpu=native -C opt-level=3 -C codegen-units=1"

        cmd = ["cargo", "build", "--release", "--jobs", str(jobs)]
        result = subprocess.run(cmd, env=env, capture_output=True, text=True)
        if result.returncode == 0:
            print("✅ [PORTABLE SETUP] Workspace compilation completed successfully.")
        else:
            print(f"⚠️ [PORTABLE SETUP] Workspace compilation failed:\n{result.stderr}")
    except Exception as e:
        print(f"❌ [PORTABLE SETUP] Error running cargo build: {e}")

def update_configs():
    """Updates paths in gateway and client config files based on current system paths and OS."""
    current_os = platform.system().lower()
    is_windows = current_os == "windows"
    bin_ext = ".exe" if is_windows else ""

    # Locate base directories
    workspace_root = Path(__file__).resolve().parents[1]
    bin_dir = workspace_root / "bin"

    print(f"🌐 [PORTABLE SETUP] Operating System detected: {platform.system()} ({current_os})")
    print(f"📁 [PORTABLE SETUP] Workspace root resolved to: {workspace_root}")

    # Ensure symlink or copy for rust_filesystem to mcp_filesystem_native exists
    fs_native = bin_dir / f"mcp_filesystem_native{bin_ext}"
    rust_fs = bin_dir / f"rust_filesystem{bin_ext}"
    if fs_native.exists():
        if not rust_fs.exists():
            print(f"🔗 [PORTABLE SETUP] Generating reference for rust_filesystem...")
            try:
                if is_windows:
                    # Windows copy
                    import shutil
                    shutil.copy2(fs_native, rust_fs)
                else:
                    # Unix symlink
                    rust_fs.symlink_to(fs_native.name)
                print("✅ [PORTABLE SETUP] Reference generated successfully.")
            except Exception as e:
                print(f"⚠️ [PORTABLE SETUP] Could not create reference: {e}")

    # 1. Update mcp_gateway_config.json
    gateway_config_path = workspace_root / "mcp_gateway_config.json"
    if gateway_config_path.exists():
        print(f"📝 [PORTABLE SETUP] Optimizing gateway configuration at: {gateway_config_path}")
        try:
            with open(gateway_config_path, "r", encoding="utf-8") as f:
                config = json.load(f)

            mcp_servers = config.get("mcpServers", {})
            for name, server in mcp_servers.items():
                cmd_path = Path(server.get("command", ""))
                # If command is absolute or relative inside workspace bin, update it
                if "bin" in cmd_path.parts or cmd_path.name == "nexus_mcp_runner":
                    server["command"] = str(bin_dir / f"nexus_mcp_runner{bin_ext}")

                args = server.get("args", [])
                for i, arg in enumerate(args):
                    arg_path = Path(arg)
                    if "bin" in arg_path.parts:
                        # Keep the filename and binary extension but update path
                        args[i] = str(bin_dir / f"{arg_path.stem}{bin_ext}")

            with open(gateway_config_path, "w", encoding="utf-8") as f:
                json.dump(config, f, indent=2)
            print("✅ [PORTABLE SETUP] Gateway configuration optimized.")
        except Exception as e:
            print(f"❌ [PORTABLE SETUP] Error updating gateway config: {e}")
    else:
        print(f"⚠️ [PORTABLE SETUP] mcp_gateway_config.json not found at {gateway_config_path}")

    # 2. Update agent mcp_config.json
    home_dir = Path.home()
    agent_config_dir = home_dir / ".gemini" / "antigravity"
    agent_config_path = agent_config_dir / "mcp_config.json"

    # Make sure target dir exists
    agent_config_dir.mkdir(parents=True, exist_ok=True)

    print(f"📝 [PORTABLE SETUP] Optimizing agent client configuration at: {agent_config_path}")
    try:
        # Load existing keys if file has content
        existing_env = {}
        if agent_config_path.exists() and agent_config_path.stat().st_size > 0:
            try:
                with open(agent_config_path, "r", encoding="utf-8") as f:
                    old_cfg = json.load(f)
                    # Preserve env vars from previous config
                    for srv_name, srv_data in old_cfg.get("mcpServers", {}).items():
                        if "env" in srv_data:
                            existing_env[srv_name] = srv_data["env"]
            except Exception:
                pass

        # Read keys from .env if possible to enrich the environment
        env_vars = {}
        env_file_path = workspace_root / ".env"
        if env_file_path.exists():
            with open(env_file_path, "r", encoding="utf-8") as f:
                for line in f:
                    if "=" in line and not line.strip().startswith("#"):
                        key, val = line.strip().split("=", 1)
                        env_vars[key.strip()] = val.strip()

        # Build fresh config
        fresh_config = {
            "mcpServers": {
                "rust-filesystem": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_filesystem{bin_ext}")],
                    "env": existing_env.get("rust-filesystem", {
                        "NEXUS_AUTONOMY_LEVEL": env_vars.get("NEXUS_AUTONOMY_LEVEL", "MAX"),
                        "FORCE_NON_INTERACTIVE": "true"
                    })
                },
                "rust-browser": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_browser{bin_ext}")],
                    "env": existing_env.get("rust-browser", {
                        "NEXUS_AUTONOMY_LEVEL": env_vars.get("NEXUS_AUTONOMY_LEVEL", "MAX")
                    })
                },
                "rust-search": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_google_search{bin_ext}")],
                    "env": existing_env.get("rust-search", {
                        "TAVILY_API_KEY": env_vars.get("TAVILY_API_KEY", ""),
                        "EXA_API_KEY": env_vars.get("EXA_API_KEY", "")
                    })
                },
                "sequential-thinking": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_sequential_thinking{bin_ext}")]
                },
                "nexus-context-7": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_context7{bin_ext}")]
                },
                "rust-github": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_github{bin_ext}")],
                    "env": existing_env.get("rust-github", {
                        "NEXUS_AUTONOMY_LEVEL": env_vars.get("NEXUS_AUTONOMY_LEVEL", "MAX")
                    })
                },
                "rust-sqlite": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_sqlite{bin_ext}")]
                },
                "rust-firecrawl": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"rust_firecrawl{bin_ext}")]
                },
                "nexus-link": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"nexus_link{bin_ext}")],
                    "env": existing_env.get("nexus-link", {
                        "TELEGRAM_TOKEN": env_vars.get("TELEGRAM_TOKEN", "")
                    })
                },
                "nexus-nerve": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [
                        str(bin_dir / f"nexus_nerve{bin_ext}"),
                        "--mode",
                        "ram"
                    ]
                },
                "nexus-brain-ra": {
                    "command": str(bin_dir / f"nexus_mcp_runner{bin_ext}"),
                    "args": [str(bin_dir / f"nexus_brain_ra{bin_ext}")]
                }
            }
        }

        with open(agent_config_path, "w", encoding="utf-8") as f:
            json.dump(fresh_config, f, indent=2)
        print("✅ [PORTABLE SETUP] Agent config updated successfully.")
    except Exception as e:
        print(f"❌ [PORTABLE SETUP] Error updating agent config: {e}")

if __name__ == "__main__":
    print("🤖 [PORTABLE SETUP] Starting setup for NEXUS multi-platform portability...")
    jobs = get_cpu_jobs()
    # If build is explicitly requested, compile first
    if "--build" in sys.argv:
        compile_workspace(jobs)
    update_configs()
    print("✅ [PORTABLE SETUP] Done. All native tools are synchronized.")
