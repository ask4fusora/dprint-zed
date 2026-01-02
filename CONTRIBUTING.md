# Contributing

This repository contains a Zed extension written in Rust.

**Important context (read first):**
- **dprint itself is primarily a formatter CLI.** It is not universally shipped as a Language Server.
- This extension integrates with Zed via Zed’s **Language Server integration**. Therefore, it only works if the `dprint` executable you point Zed at supports an **LSP-compatible mode** (for example, a wrapper/proxy executable or a `dprint` build that actually implements such a mode).
- Prior documentation in this repo referenced `lsp-proxy` as if it were guaranteed. That is **not** a safe assumption and depends on which executable you install.

## Development setup

### Prerequisites

- Rust toolchain installed (stable is fine).
- `wasm32-wasip2` target installed (Zed loads Rust extensions as WASM):

```
rustup target add wasm32-wasip2
```

### Build / check

From the repo root:

```
cargo check
```

(Optionally run `cargo fmt` / `cargo clippy` if you use them locally.)

### Load the extension in Zed (developer install)

1. Open Zed
2. Open the command palette
3. Run: `zed: install dev extensions`
4. Select the directory of this repo

After changes, open the Extensions view (`zed: extensions`), locate this extension under Installed, and click **Rebuild**.

## Testing the extension behavior

### 1) Confirm what your `dprint` executable supports

Before debugging Zed, verify the executable you expect Zed to run.

In a terminal, run:

- `dprint --help`
- `dprint --version`

If you expect LSP support, also try:

- `dprint lsp --help`

If `lsp` does not exist (or if they don’t start a language server), then Zed cannot use that binary as a language server and the extension will not function as designed.

### 2) Configure Zed to use a specific `dprint` binary (recommended)

Zed language server configuration is done in `settings.json` under the `lsp` key, using the language server id from `extension.toml` (for this extension: `dprint`).

Example:

```
{
  "lsp": {
    "dprint": {
      "binary": {
        "path": "/absolute/path/to/dprint-or-wrapper",
        "arguments": ["lsp"]
      }
    }
  }
}
```

Notes:
- The correct `arguments` depend entirely on the executable you are using.
- If you are using a wrapper that provides an LSP server, its flags/arguments may differ.
- This repository’s Rust code also attempts to find `node_modules/.bin/dprint` in the current worktree, but explicit configuration is the fastest way to eliminate ambiguity.

### 3) Extension settings passed to the server

This extension passes workspace configuration under the `dprint` key. You can set extension-specific settings like:

- `config_path` (string, relative to worktree root)
- `require_config_file` (boolean)

Example:

```
{
  "lsp": {
    "dprint": {
      "settings": {
        "config_path": "dprint.json",
        "require_config_file": true
      }
    }
  }
}
```

## Logs / troubleshooting

Zed log locations vary by OS and installation method. Use Zed’s documentation to locate logs for your platform and then search for `dprint` entries.

When filing issues, include:
- Zed version
- Your `zed_extension_api` version (from `Cargo.toml`)
- The exact `dprint` executable you configured (path) and `--version` output
- What `dprint lsp --help` / `dprint lsp-proxy --help` returns (if applicable)
- Relevant log excerpts

## Contribution workflow

1. Make changes in a branch
2. Ensure `cargo check` passes
3. Keep changes focused and documented
4. Open a PR with:
   - What you changed
   - How you tested it
   - Any known limitations

## Security note

Do not commit secrets (API keys, tokens) to this repository. If you add tooling that requires credentials, document secure configuration via environment variables instead of hardcoding.
