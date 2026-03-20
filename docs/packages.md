# Native Package Catalog (v0.6)

Trellis v0.6 ships a deliberately small first-class native catalog in the official `vineyard-core` registry.

## Featured native packages

### 1) `overstrings-cli`

**Role:** Flagship text utility package.

**Why it exists:** proves Trellis can distribute a practical command-line tool with repeatable behavior.

**Key commands:**
- `overstrings normalize "Hello World"`
- `overstrings title "hello world"`
- `overstrings stats "hello world"`

### 2) `vineyard-core`

**Role:** Ecosystem substrate package.

**Why it exists:** anchors environment/path/platform identity for operators using Trellis.

**Key commands:**
- `vineyard-core status`
- `vineyard-core paths`
- `vineyard-core doctor-hint`

### 3) `tiekat-pulse`

**Role:** Diagnostic and introspection package.

**Why it exists:** demonstrates shipping lightweight health snapshot tooling through Trellis.

**Key commands:**
- `tiekat-pulse snapshot`
- `tiekat-pulse process --name trellis`
- `tiekat-pulse version`

## Why this matters

This catalog proves Trellis is no longer just package infrastructure. It now ships distinct, installable native tools while remaining local-first and trust-conscious.
