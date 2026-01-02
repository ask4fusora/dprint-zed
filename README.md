# dprint extension for Zed

This extension integrates **dprint** with **Zed** by wiring Zed’s Language Server integration to a `dprint` executable.

Important: **dprint is primarily a formatter CLI.** This extension expects a `dprint` executable that can run in a **language-server-compatible mode**. Not every `dprint` installation provides that.

If you installed a `dprint` CLI that only supports `dprint fmt` / `dprint check` (the common case), then **there may be no LSP to run**, and this extension will not function as an LSP-backed formatter.

## What this extension does

- Registers a `dprint` language server entry for a set of web languages in `extension.toml`.
- Starts a `dprint` executable for Zed to talk to using Zed’s LSP integration.
- Sends Zed LSP workspace configuration under the `dprint` key:
  - `requireConfiguration` (boolean)
  - `configurationPath` (string | null)

## What this extension does *not* do

- It does **not** implement a formatter itself.
- It does **not** wrap `dprint fmt` via stdin as a fallback.
- It does **not** magically add LSP support to the standard `dprint` CLI.

## Supported languages (Zed side)

Zed will attempt to use this LSP for the languages listed in `extension.toml`:

- JavaScript, TypeScript, JSX, TSX
- JSON, JSONC
- HTML, CSS
- GraphQL
- Markdown
- Astro, Svelte, Vue

Whether formatting actually works depends on:
1) the `dprint` executable you are running, and
2) your `dprint` configuration/plugins.

## Requirements

- Zed version: depends on the Zed extension API version used by this extension. (This repo currently targets modern Zed builds compatible with `zed_extension_api`.)
- A `dprint` executable available either:
  - via `node_modules/.bin/dprint` in your project, or
  - via an explicit Zed LSP binary override.

## Installing

1. In Zed: open the command palette and run `zed: extensions`.
2. Install the extension (or use “Install Dev Extension” for local development).

## Configuration

### 1) Ensure you have a compatible `dprint` executable

This is the critical part.

Verify what your `dprint` supports by running (in your project, if using a local install):

- `dprint --help`
- `dprint lsp --help` (if you expect an LSP mode)
- `dprint lsp-proxy --help` (if you expect an LSP proxy mode)

If your `dprint` does not expose an LSP-compatible mode, Zed cannot run it as a language server.

### 2) Configure the dprint config file

This extension looks for a config file in the worktree:

- `dprint.json`
- `.dprint.json`

You can also specify a custom relative path via Zed settings.

Example `settings.json`:

```json
{
  "lsp": {
    "dprint": {
      "settings": {
        "config_path": "config/dprint.json",
        "require_config_file": true
      }
    }
  }
}
```

- `config_path` (string): path relative to the worktree root.
- `require_config_file` (boolean): if `true`, sets `requireConfiguration` for the server.

### 3) Override the binary (recommended when troubleshooting)

You can point Zed at a specific executable and arguments:

```json
{
  "lsp": {
    "dprint": {
      "binary": {
        "path": "/absolute/path/to/dprint",
        "arguments": ["lsp"]
      }
    }
  }
}
```

Notes:
- The correct `arguments` depend entirely on the executable you are using.
- If you’re using a wrapper/proxy executable, its argument may differ.

## Troubleshooting

### “Nothing happens” / no formatting
Most commonly: you are using a `dprint` CLI that **does not implement an LSP server**.

Confirm by running `dprint lsp --help` (or equivalent). If it fails, Zed cannot start it as an LSP.

### Logs
Zed logs locations vary by OS and installation. Consult Zed’s documentation for where to find logs on your platform, then search for entries related to `dprint`.

## Development (local)

- Install Rust.
- Install the `wasm32-wasip2` target:
  - `rustup target add wasm32-wasip2`
- In Zed: `zed: install dev extensions` and select the repo folder.
- Use the Extensions UI to rebuild after changes.

## License

MIT
