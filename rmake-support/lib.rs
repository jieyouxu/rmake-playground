use std::collections::HashMap;

#[derive(Debug)]
pub struct RmakeSupportContext {
    environment: Environment,
}

/// The environment consists of named environment variables.
#[derive(Debug)]
pub struct Environment {
    env_map: HashMap<String, EnvironmentVariable>,
}

/// An environment variable can be *static* or *dynamic* depending on the way it resolves its
/// constituent path segments which may involve other environment variables or shell expansions.
#[derive(Debug)]
pub enum EnvironmentVariable {
    /// If an environment variable is *statically* defined, it can be immediately resolved based on
    /// the current environment.
    Static(String),
    /// Resolution of a dynamically defined environment variable needs to be deferred until
    /// the time of use.
    Dynamic(Vec<PathFragment>),
}

#[derive(Debug, Clone)]
pub enum PathFragment {
    Str(String),
    ExpandEnvVar(Vec<PathFragment>),
    ExpandShell(Vec<PathFragment>),
}

impl RmakeSupportContext {
    pub fn init() -> Self {
        let mut environment = Environment {
            env_map: HashMap::default(),
        };

        let ld_lib_path_envvar = "LD_LIB_PATH".to_string();

        environment.env_map.insert(
            "LD_LIB_PATH_ENVVAR".to_string(),
            EnvironmentVariable::Static("LD_LIB_PATH".to_string()),
        );
        environment.env_map.insert(
            ld_lib_path_envvar.clone(),
            EnvironmentVariable::Static("LD_LIB_PATH_PLACEHOLDER".to_string()),
        );
        environment.env_map.insert(
            "TMPDIR".to_string(),
            EnvironmentVariable::Static("TMPDIR_PLACEHOLDER".to_string()),
        );
        environment.env_map.insert(
            "HOST_RPATH_DIR".to_string(),
            EnvironmentVariable::Static("HOST_RPATH_DIR_PLACEHOLDER".to_string()),
        );
        environment.env_map.insert(
            "HOST_RPATH_ENV".to_string(),
            EnvironmentVariable::Dynamic(vec![
                PathFragment::ExpandEnvVar(vec![PathFragment::Str("TMPDIR".to_string())]),
                PathFragment::Str(":".to_string()),
                PathFragment::ExpandEnvVar(vec![PathFragment::Str("HOST_RPATH_DIR".to_string())]),
                PathFragment::Str(":".to_string()),
                PathFragment::ExpandEnvVar(vec![PathFragment::ExpandEnvVar(vec![
                    PathFragment::Str("LD_LIB_PATH_ENVVAR".to_string()),
                ])]),
            ]),
        );

        Self { environment }
    }

    /// Resolve an environment variable (possibly dynamically based on the current environment).
    /// Note that because environment variables can be *dynamically* resolved, calling this method
    /// can mutate the current environment.
    pub fn resolve_env_var(&mut self, env_var: &str) -> std::io::Result<String> {
        let Some(var) = self.environment.env_map.get(env_var) else { return Ok(String::new()) };

        match var {
            EnvironmentVariable::Static(s) => Ok(s.to_string()),
            EnvironmentVariable::Dynamic(fragments) => {
                self.resolve_env_var_fragments(fragments.to_vec())
            }
        }
    }

    fn resolve_env_var_fragments(
        &mut self,
        fragments: Vec<PathFragment>,
    ) -> std::io::Result<String> {
        let mut res = String::new();
        for fragment in fragments {
            match fragment {
                PathFragment::Str(s) => res.push_str(&s),
                PathFragment::ExpandEnvVar(inner_fragments) => {
                    let env_var = self.resolve_env_var_fragments(inner_fragments)?;
                    res.push_str(&self.resolve_env_var(&env_var)?);
                }
                PathFragment::ExpandShell(inner_fragments) => {
                    let inner_str = self.resolve_env_var_fragments(inner_fragments)?;
                    let std::process::Output {
                        status,
                        stdout,
                        stderr,
                    } = std::process::Command::new(inner_str).output()?;
                    if !status.success() || !stderr.is_empty() {
                        panic!("child process error");
                    }
                    res.push_str(&String::from_utf8_lossy(&stdout));
                }
            };
        }
        Ok(res)
    }
}
