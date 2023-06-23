use super::{PathFragment, RmakeSupportContext};

use std::collections::{HashMap, HashSet};
use std::process::{Command, Output};

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum EmitKind {
    Metadata,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CrateType {
    Lib,
    Bin,
}

impl CrateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lib => "lib",
            Self::Bin => "bin",
        }
    }
}

#[derive(Debug)]
pub struct RustcBuilder<'scx> {
    pub(crate) scx: &'scx mut RmakeSupportContext,
    pub(crate) current_dir: Vec<PathFragment>,
    pub(crate) path: Vec<PathFragment>,
    pub(crate) emits: HashSet<EmitKind>,
    pub(crate) crate_type: Option<CrateType>,
    pub(crate) externs: HashMap<String, Vec<PathFragment>>,
}

impl<'scx> RustcBuilder<'scx> {
    pub(crate) fn new(scx: &'scx mut RmakeSupportContext) -> Self {
        Self {
            scx,
            current_dir: vec![],
            path: vec![],
            emits: HashSet::default(),
            crate_type: None,
            externs: HashMap::default(),
        }
    }

    pub fn current_dir(mut self, path: &[PathFragment]) -> Self {
        self.current_dir = path.to_vec();
        self
    }

    pub fn path(mut self, path: &[PathFragment]) -> Self {
        self.path = path.to_vec();
        self
    }

    pub fn emit(mut self, emit_kind: EmitKind) -> Self {
        self.emits.insert(emit_kind);
        self
    }

    pub fn crate_type(mut self, crate_type: CrateType) -> Self {
        self.crate_type = Some(crate_type);
        self
    }

    pub fn r#extern(mut self, name: String, path: &[PathFragment]) -> Self {
        self.externs.insert(name, path.to_vec());
        self
    }

    pub fn compile(self) -> std::io::Result<Output> {
        assert!(!self.current_dir.is_empty());
        assert!(!self.path.is_empty());

        let mut cmd = Command::new("rustc");
        cmd.arg("+nightly");
        cmd.current_dir(self.scx.resolve_env_var_fragments(&self.current_dir)?);

        for emit in self.emits {
            match emit {
                EmitKind::Metadata => cmd.arg("--emit").arg("metadata"),
            };
        }

        if let Some(crate_type) = self.crate_type {
            cmd.arg("--crate-type").arg(crate_type.as_str());
        }

        for (name, path) in self.externs.iter() {
            let path = self.scx.resolve_env_var_fragments(&path)?;
            cmd.arg("--extern").arg(format!("{}={}", name, path));
        }

        cmd.arg(format!(
            "{}",
            self.scx.resolve_env_var_fragments(&self.path)?
        ));
        cmd.output()
    }
}
