# DAO_lim Windows Toolchain Fix

This note captures the exact local blocker seen on the current Windows machine
while trying to build `dao` and `daoctl` in order to collect real benchmark
numbers.

## Current machine snapshot

- OS: Windows
- CPU: `AMD Ryzen 7 5700U with Radeon Graphics`
- RAM: `16 GB`
- Rust toolchain state from `rustup show`:
  - installed toolchains:
    - `stable-x86_64-pc-windows-gnu`
    - `stable-x86_64-pc-windows-msvc`
  - active default toolchain:
    - `stable-x86_64-pc-windows-gnu`
  - installed targets:
    - `x86_64-pc-windows-gnu`
- Rust version:
  - `rustc 1.93.0 (254b59607 2026-01-19)`

## Exact failure modes

### 1. GNU toolchain fails at link stage

Command:

```powershell
cargo build --release -p dao -p daoctl
```

Observed error:

```text
lld: error: unable to find library -lgcc_eh
lld: error: unable to find library -lgcc
```

Interpretation:

- the project itself is not the blocker
- the active `windows-gnu` toolchain does not have a working MinGW runtime
  available to the linker on this machine

### 2. MSVC toolchain is installed but not usable yet

Command:

```powershell
cargo +stable-x86_64-pc-windows-msvc build --release -p dao -p daoctl
```

Observed error:

```text
linker `link.exe` not found
```

Interpretation:

- the `windows-msvc` Rust toolchain is present
- but Visual Studio Build Tools are missing, or not loaded into the shell

## Best fix path

The most reliable fix for this machine is to use the `MSVC` toolchain and
install the Microsoft C++ build tools.

### Recommended steps

1. Install Visual Studio Build Tools

Install:

- `Build Tools for Visual Studio 2022`
- workload: `Desktop development with C++`

Make sure these components are present:

- MSVC v143 or newer C++ build tools
- Windows 10 or Windows 11 SDK
- C++ CMake tools for Windows

2. Open a developer shell

Use one of these:

- `x64 Native Tools Command Prompt for VS 2022`
- PowerShell after running `VsDevCmd.bat`

3. Switch Rust to MSVC

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
```

4. Verify the linker exists

```powershell
where.exe link
```

This should return a real path inside Visual Studio Build Tools.

5. Build DAO binaries

```powershell
cargo build --release -p dao -p daoctl
```

6. Run the benchmark harness

```powershell
python scripts/e2e_benchmark.py
```

## Alternative fix path

If you want to keep the `GNU` toolchain instead, you need a working MinGW-w64
runtime that provides `libgcc` and `libgcc_eh`.

Typical recovery path:

1. Reinstall or repair the MinGW-w64 toolchain that Rust GNU depends on
2. Ensure the MinGW `bin` and corresponding runtime libraries are on `PATH`
3. Re-run:

```powershell
cargo build --release -p dao -p daoctl
```

This route is usually more fragile on Windows than `MSVC`, so it is not the
recommended path for this repository.

## Fastest route to real benchmark numbers

If the goal is to capture `DAO_lim` numbers as quickly as possible:

1. switch this machine to `MSVC`
2. build `dao` and `daoctl`
3. run `python scripts/e2e_benchmark.py`
4. copy the output into `docs/BENCHMARKS.md` using the benchmark report template

## Success checklist

Once the environment is fixed, all of the following should work:

```powershell
rustup show
where.exe link
cargo build --release -p dao -p daoctl
python scripts/e2e_benchmark.py --help
python scripts/e2e_benchmark.py
```

## Why this matters

Until the toolchain is fixed, the repository already has:

- a benchmark harness
- a benchmark config
- a benchmark report template

But it does not yet have the most valuable next artifact:

- a real benchmark report with measured numbers from a named machine
