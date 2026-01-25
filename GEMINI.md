# Gemini Context

This file stores context for the Gemini CLI to remember for this project.

## Project Overview
- **Project Name**: frbktg
- **Backend (Rust)**: `backend_rust`
- **Backend (Go)**: `backend_go`
- **Frontend**: `frontend`

## Rust Backend (`backend_rust`)
- **Repositories Location**: `backend_rust/src/infrastructure/repositories/`
- **Testing Tools**:
    - Test Runner: `cargo nextest`
    - Coverage: `llvm-cov`

## Workflow for Adding Tests
1.  Add **one** test case at a time.
2.  After adding a test, run `cargo nextest run` to ensure all tests pass.
3.  Once verified, re-run the coverage report with `cargo llvm-cov nextest --lcov --output-path lcov.info` to find the next area to test.
4.  Always re-read files before editing.

## Commands
- **Run tests**: `cargo nextest run` (in `backend_rust` directory)
- **Run tests with coverage**: `cargo llvm-cov nextest --lcov --output-path lcov.info` (in `backend_rust` directory)

## Notes
- The repositories in `backend_rust` appear to be well-tested. All 155 tests passed, and a coverage report was successfully generated.
- The user has fixed the `Default` trait implementation for `ListQuery`.