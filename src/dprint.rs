use std::path::{Path, PathBuf};
use zed::settings::{CommandSettings, LspSettings};
use zed_extension_api::{
  self as zed, LanguageServerId, Result, Worktree,
  serde_json::{self, Value},
};

const WORKTREE_BIN: &str = "node_modules/.bin/dprint";
const PACKAGE_NAME: &str = "dprint";
const DEFAULT_CONFIG_CANDIDATES: &[&str] = &["dprint.json", ".dprint.json"];

struct DprintExtension;

impl DprintExtension {
  fn worktree_rooted(&self, worktree: &Worktree, relative: &str) -> String {
    Path::new(worktree.root_path().as_str())
      .join(relative)
      .to_string_lossy()
      .to_string()
  }

  fn read_json_file(&self, worktree: &Worktree, path: &str) -> Option<Value> {
    let contents = worktree.read_text_file(path).ok()?;
    serde_json::from_str(contents.as_str()).ok()
  }

  fn worktree_dprint_exists(&self, worktree: &Worktree) -> bool {
    let package_json = self.read_json_file(worktree, "package.json");
    let deno_json = self.read_json_file(worktree, "deno.json");

    let in_package_json = package_json.is_some_and(|f| {
      f.get("dependencies")
        .and_then(|deps| deps.get(PACKAGE_NAME))
        .is_some()
        || f
          .get("devDependencies")
          .and_then(|deps| deps.get(PACKAGE_NAME))
          .is_some()
    });

    let in_deno_json = deno_json.is_some_and(|f| {
      f.get("imports")
        .and_then(|imports| imports.get(PACKAGE_NAME))
        .is_some()
    });

    in_package_json || in_deno_json
  }

  /// Returns a resolved configuration path if present.
  fn config_path(&self, worktree: &Worktree, settings: &Value) -> Option<PathBuf> {
    if let Some(config_path) = settings.get("config_path").and_then(|v| v.as_str()) {
      if worktree.read_text_file(config_path).is_ok() {
        return Some(Path::new(worktree.root_path().as_str()).join(config_path));
      } else {
        log::warn!(
          "dprint config_path setting '{}' not found in worktree {}",
          config_path,
          worktree.root_path()
        );
        return None;
      }
    }

    for candidate in DEFAULT_CONFIG_CANDIDATES {
      if worktree.read_text_file(candidate).is_ok() {
        return Some(Path::new(worktree.root_path().as_str()).join(candidate));
      }
    }

    None
  }

  fn require_config_file(&self, settings: &Value) -> bool {
    settings
      .get("require_config_file")
      .and_then(|value| value.as_bool())
      .unwrap_or(false)
  }

  fn command_with_custom_binary(
    &self,
    binary: &CommandSettings,
    default_path: String,
    default_args: Vec<String>,
  ) -> zed::Command {
    zed::Command {
      command: binary.path.clone().unwrap_or(default_path),
      args: binary.arguments.clone().unwrap_or(default_args),
      env: Default::default(),
    }
  }
}

impl zed::Extension for DprintExtension {
  fn new() -> Self {
    Self
  }

  fn language_server_command(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
  ) -> Result<zed::Command> {
    let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

    let mut args = vec!["lsp".to_string()];
    log::info!("dprint: preparing language server command");

    // Custom binary supplied by user settings
    if let Some(binary) = settings.binary {
      let default_path = self.worktree_rooted(worktree, WORKTREE_BIN);
      return Ok(self.command_with_custom_binary(&binary, default_path, args));
    }

    // Prefer workspace installation driven through node
    if self.worktree_dprint_exists(worktree) {
      log::info!("dprint: found workspace installation");
      let server_path = self.worktree_rooted(worktree, WORKTREE_BIN);

      let mut node_args = vec![server_path];
      node_args.append(&mut args);

      return Ok(zed::Command {
        command: zed::node_binary_path()?,
        args: node_args,
        env: Default::default(),
      });
    }

    // Fallback: try to execute the local binary path directly (relative to worktree)
    let server_path = self.worktree_rooted(worktree, WORKTREE_BIN);
    log::info!(
      "dprint: workspace install not detected; falling back to {}",
      server_path
    );

    Ok(zed::Command {
      command: server_path,
      args,
      env: Default::default(),
    })
  }
}

zed::register_extension!(DprintExtension);
