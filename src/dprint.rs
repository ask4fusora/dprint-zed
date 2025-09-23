use std::path::{Path, PathBuf};
use zed::settings::LspSettings;
use zed_extension_api::{
  self as zed, LanguageServerId, Result, Worktree,
  serde_json::{self, Value},
};

const WORKTREE_SERVER_PATH: &str = "node_modules/.bin/dprint";

const PACKAGE_NAME: &str = "dprint";

const DPRINT_CONFIG_PATHS: &[&str] = &["dprint.json"];

struct DprintExtension;

impl DprintExtension {
  fn worktree_dprint_exists(&self, worktree: &zed::Worktree) -> bool {
    // This is a workaround, as reading the file from wasm doesn't work.
    // Instead we try to read the `package.json`, see if `dprint` is installed
    let package_json = worktree
      .read_text_file("package.json")
      .unwrap_or(String::from(r#"{}"#));

    let deno_json = worktree
      .read_text_file("deno.json")
      .unwrap_or(String::from(r#"{}"#));

    let package_json: Option<serde_json::Value> = serde_json::from_str(package_json.as_str()).ok();
    let deno_json: Option<serde_json::Value> = serde_json::from_str(deno_json.as_str()).ok();

    let in_package_json = package_json.is_some_and(|f| {
      !f["dependencies"][PACKAGE_NAME].is_null() || !f["devDependencies"][PACKAGE_NAME].is_null()
    });

    let in_deno_json = deno_json.is_some_and(|f| !f["imports"][PACKAGE_NAME].is_null());

    in_package_json || in_deno_json
  }

  // Returns the path if a config file exists
  pub fn config_path(&self, worktree: &zed::Worktree, settings: &Value) -> Option<String> {
    let config_path_setting = settings.get("config_path").and_then(|value| value.as_str());

    if let Some(config_path) = config_path_setting {
      if worktree.read_text_file(config_path).is_ok() {
        return Some(config_path.to_string());
      } else {
        return None;
      }
    }

    for config_path in DPRINT_CONFIG_PATHS {
      if worktree.read_text_file(config_path).is_ok() {
        return Some(config_path.to_string());
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
}

impl zed::Extension for DprintExtension {
  fn new() -> Self {
    Self
  }

  fn language_server_command(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
  ) -> Result<zed::Command> {
    let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

    let mut args = vec!["lsp".to_string()];

    log::info!("Executing server command");

    // check and run dprint with custom binary
    if let Some(binary) = settings.binary {
      return Ok(zed::Command {
        command: binary
          .path
          .map_or(WORKTREE_SERVER_PATH.to_string(), |path| path),
        args: binary.arguments.map_or(args, |args| args),
        env: Default::default(),
      });
    }

    // try to run from worktree dprint package
    if self.worktree_dprint_exists(worktree) {
      log::info!("dpring in workspace");

      let server_path = Path::new(worktree.root_path().as_str())
        .join(WORKTREE_SERVER_PATH)
        .to_string_lossy()
        .to_string();

      let mut node_args = vec![server_path];
      node_args.append(&mut args);

      return Ok(zed::Command {
        command: zed::node_binary_path()?,
        args: node_args,
        env: Default::default(),
      });
    }

    let server_path = PathBuf::from("./node_modules/.bin/dprint");
    log::info!("dpring not in workspace");

    Ok(zed::Command {
      command: server_path.to_string_lossy().to_string(),
      args,
      env: Default::default(),
    })
  }

  fn language_server_workspace_configuration(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
  ) -> Result<Option<Value>> {
    let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

    let Some(settings) = lsp_settings.settings else {
      return Ok(Some(serde_json::json!({
        "dprint": {},
      })));
    };

    let config_path = self
      .config_path(worktree, &settings)
      .map(|p| Path::new(&worktree.root_path()).join(p));

    Ok(Some(serde_json::json!({
      "dprint": {
        "requireConfiguration": self.require_config_file(&settings),
        "configurationPath": config_path,
      },
    })))
  }
}

zed::register_extension!(DprintExtension);
