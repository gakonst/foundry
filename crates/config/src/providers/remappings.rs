use crate::{foundry_toml_dirs, remappings_from_env_var, remappings_from_newline, Config};
use figment::{
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use foundry_compilers::artifacts::remappings::{RelativeRemapping, Remapping};
use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

/// Wrapper types over a `Vec<Remapping>` that only appends unique remappings.
#[derive(Clone, Debug, Default)]
pub struct Remappings {
    /// Remappings.
    remappings: Vec<Remapping>,
    /// Source, test and script configured project dirs.
    /// Remappings of these dirs from libs are ignored.
    project_paths: Vec<Remapping>,
}

impl Remappings {
    /// Create a new `Remappings` wrapper with an empty vector.
    pub fn new() -> Self {
        Self { remappings: Vec::new(), project_paths: Vec::new() }
    }

    /// Create a new `Remappings` wrapper with a vector of remappings.
    pub fn new_with_remappings(remappings: Vec<Remapping>) -> Self {
        Self { remappings, project_paths: Vec::new() }
    }

    /// Extract project paths that cannot be remapped by dependencies.
    pub fn with_figment(mut self, figment: &Figment) -> Self {
        let mut add_project_remapping = |path: &str| {
            if let Ok(path) = figment.find_value(path) {
                if let Some(path) = path.into_string() {
                    let remapping = Remapping {
                        context: None,
                        name: format!("{path}/"),
                        path: format!("{path}/"),
                    };
                    self.project_paths.push(remapping);
                }
            }
        };
        add_project_remapping("src");
        add_project_remapping("test");
        add_project_remapping("script");
        self
    }

    /// Filters the remappings vector by name and context.
    fn filter_key(r: &Remapping) -> String {
        match &r.context {
            Some(str) => str.clone() + &r.name.clone(),
            None => r.name.clone(),
        }
    }

    /// Consumes the wrapper and returns the inner remappings vector.
    pub fn into_inner(self) -> Vec<Remapping> {
        let mut seen = HashSet::new();
        let remappings =
            self.remappings.iter().filter(|r| seen.insert(Self::filter_key(r))).cloned().collect();
        remappings
    }

    /// Push an element to the remappings vector, but only if it's not already present.
    fn push(&mut self, remapping: Remapping) {
        // Special handling for .sol file remappings, only allow one remapping per source file.
        if remapping.name.ends_with(".sol") && !remapping.path.ends_with(".sol") {
            return;
        }

        if self.remappings.iter().any(|existing| {
            if remapping.name.ends_with(".sol") {
                // For .sol files, only prevent duplicate source names in the same context
                return existing.name == remapping.name &&
                    existing.context == remapping.context &&
                    existing.path == remapping.path
            }

            // What we're doing here is filtering for ambiguous paths. For example, if we have
            // @prb/math/=node_modules/@prb/math/src/ as existing, and
            // @prb/=node_modules/@prb/ as the one being checked,
            // we want to keep the already existing one, which is the first one. This way we avoid
            // having to deal with ambiguous paths which is unwanted when autodetecting remappings.
            // Remappings are added from root of the project down to libraries, so
            // we also want to exclude any conflicting remappings added from libraries. For example,
            // if we have `@utils/=src/` added in project remappings and `@utils/libraries/=src/`
            // added in a dependency, we don't want to add the new one as it conflicts with project
            // existing remapping.
            let mut existing_name_path = existing.name.clone();
            if !existing_name_path.ends_with('/') {
                existing_name_path.push('/')
            }
            let is_conflicting = remapping.name.starts_with(&existing_name_path) ||
                existing.name.starts_with(&remapping.name);
            is_conflicting && existing.context == remapping.context
        }) {
            return;
        };

        // Ignore remappings of root project src, test or script dir.
        // See <https://github.com/foundry-rs/foundry/issues/3440>.
        if self
            .project_paths
            .iter()
            .any(|project_path| remapping.name.eq_ignore_ascii_case(&project_path.name))
        {
            return;
        };

        self.remappings.push(remapping);
    }

    /// Extend the remappings vector, leaving out the remappings that are already present.
    pub fn extend(&mut self, remappings: Vec<Remapping>) {
        for remapping in remappings {
            self.push(remapping);
        }
    }
}

/// A figment provider that checks if the remappings were previously set and if they're unset looks
/// up the fs via
///   - `DAPP_REMAPPINGS` || `FOUNDRY_REMAPPINGS` env var
///   - `<root>/remappings.txt` file
///   - `Remapping::find_many`.
pub struct RemappingsProvider<'a> {
    /// Whether to auto detect remappings from the `lib_paths`
    pub auto_detect_remappings: bool,
    /// The lib/dependency directories to scan for remappings
    pub lib_paths: Cow<'a, Vec<PathBuf>>,
    /// the root path used to turn an absolute `Remapping`, as we're getting it from
    /// `Remapping::find_many` into a relative one.
    pub root: &'a Path,
    /// This contains either:
    ///   - previously set remappings
    ///   - a `MissingField` error, which means previous provider didn't set the "remappings" field
    ///   - other error, like formatting
    pub remappings: Result<Vec<Remapping>, Error>,
}

impl RemappingsProvider<'_> {
    /// Find and parse remappings for the projects
    ///
    /// **Order**
    ///
    /// Remappings are built in this order (last item takes precedence)
    /// - Autogenerated remappings
    /// - toml remappings
    /// - `remappings.txt`
    /// - Environment variables
    /// - CLI parameters
    fn get_remappings(&self, remappings: Vec<Remapping>) -> Result<Vec<Remapping>, Error> {
        trace!("get all remappings from {:?}", self.root);
        /// prioritizes remappings that are closer: shorter `path`
        ///   - ("a", "1/2") over ("a", "1/2/3")
        ///
        /// grouped by remapping context
        fn insert_closest(
            mappings: &mut BTreeMap<Option<String>, BTreeMap<String, PathBuf>>,
            context: Option<String>,
            key: String,
            path: PathBuf,
        ) {
            let context_mappings = mappings.entry(context).or_default();
            match context_mappings.entry(key) {
                Entry::Occupied(mut e) => {
                    if e.get().components().count() > path.components().count() {
                        e.insert(path);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(path);
                }
            }
        }

        // Let's first just extend the remappings with the ones that were passed in,
        // without any filtering.
        let mut user_remappings = Vec::new();

        // check env vars
        if let Some(env_remappings) = remappings_from_env_var("DAPP_REMAPPINGS")
            .or_else(|| remappings_from_env_var("FOUNDRY_REMAPPINGS"))
        {
            user_remappings
                .extend(env_remappings.map_err::<Error, _>(|err| err.to_string().into())?);
        }

        // check remappings.txt file
        let remappings_file = self.root.join("remappings.txt");
        if remappings_file.is_file() {
            let content = fs::read_to_string(remappings_file).map_err(|err| err.to_string())?;
            let remappings_from_file: Result<Vec<_>, _> =
                remappings_from_newline(&content).collect();
            user_remappings
                .extend(remappings_from_file.map_err::<Error, _>(|err| err.to_string().into())?);
        }

        user_remappings.extend(remappings);
        // Let's now use the wrapper to conditionally extend the remappings with the autodetected
        // ones. We want to avoid duplicates, and the wrapper will handle this for us.
        let mut all_remappings = Remappings::new_with_remappings(user_remappings);

        // scan all library dirs and autodetect remappings
        // TODO: if a lib specifies contexts for remappings manually, we need to figure out how to
        // resolve that
        if self.auto_detect_remappings {
            let mut lib_remappings = BTreeMap::new();
            // use auto detection for all libs
            for r in self
                .lib_paths
                .iter()
                .map(|lib| self.root.join(lib))
                .inspect(|lib| trace!(?lib, "find all remappings"))
                .flat_map(|lib| Remapping::find_many(&lib))
            {
                // this is an additional safety check for weird auto-detected remappings
                if ["lib/", "src/", "contracts/"].contains(&r.name.as_str()) {
                    trace!(target: "forge", "- skipping the remapping");
                    continue
                }
                insert_closest(&mut lib_remappings, r.context, r.name, r.path.into());
            }

            // Find all remappings from libs that use a foundry.toml
            // Explicit remappings should override auto-detected remappings.
            for r in self.lib_foundry_toml_remappings() {
                lib_remappings.entry(r.context).or_default().insert(r.name, r.path.into());
            }

            all_remappings.extend(
                lib_remappings
                    .into_iter()
                    .flat_map(|(context, remappings)| {
                        remappings.into_iter().map(move |(name, path)| Remapping {
                            context: context.clone(),
                            name,
                            path: path.to_string_lossy().into(),
                        })
                    })
                    .collect(),
            );
        }

        Ok(all_remappings.into_inner())
    }

    /// Returns all remappings declared in foundry.toml files of libraries
    fn lib_foundry_toml_remappings(&self) -> impl Iterator<Item = Remapping> + '_ {
        self.lib_paths
            .iter()
            .map(|p| if p.is_absolute() { self.root.join("lib") } else { self.root.join(p) })
            .flat_map(foundry_toml_dirs)
            .inspect(|lib| {
                trace!("find all remappings of nested foundry.toml lib: {:?}", lib);
            })
            .flat_map(|lib: PathBuf| {
                // load config, of the nested lib if it exists
                let Ok(config) = Config::load_with_root(&lib) else { return vec![] };
                let config = config.sanitized();

                // if the configured _src_ directory is set to something that
                // [Remapping::find_many()] doesn't classify as a src directory (src, contracts,
                // lib), then we need to manually add a remapping here
                let mut src_remapping = None;
                if ![Path::new("src"), Path::new("contracts"), Path::new("lib")]
                    .contains(&config.src.as_path())
                {
                    if let Some(name) = lib.file_name().and_then(|s| s.to_str()) {
                        let mut r = Remapping {
                            context: None,
                            name: format!("{name}/"),
                            path: format!("{}", lib.join(&config.src).display()),
                        };
                        if !r.path.ends_with('/') {
                            r.path.push('/')
                        }
                        src_remapping = Some(r);
                    }
                }

                // Eventually, we could set context for remappings at this location,
                // taking into account the OS platform. We'll need to be able to handle nested
                // contexts depending on dependencies for this to work.
                // For now, we just leave the default context (none).
                let mut remappings =
                    config.remappings.into_iter().map(Remapping::from).collect::<Vec<Remapping>>();

                if let Some(r) = src_remapping {
                    remappings.push(r);
                }
                remappings
            })
    }
}

impl Provider for RemappingsProvider<'_> {
    fn metadata(&self) -> Metadata {
        Metadata::named("Remapping Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let remappings = match &self.remappings {
            Ok(remappings) => self.get_remappings(remappings.clone()),
            Err(err) => {
                if let figment::error::Kind::MissingField(_) = err.kind {
                    self.get_remappings(vec![])
                } else {
                    return Err(err.clone())
                }
            }
        }?;

        // turn the absolute remapping into a relative one by stripping the `root`
        let remappings = remappings
            .into_iter()
            .map(|r| RelativeRemapping::new(r, self.root).to_string())
            .collect::<Vec<_>>();

        Ok(Map::from([(
            Config::selected_profile(),
            Dict::from([("remappings".to_string(), figment::value::Value::from(remappings))]),
        )]))
    }

    fn profile(&self) -> Option<Profile> {
        Some(Config::selected_profile())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sol_file_remappings() {
        let mut remappings = Remappings::new();

        // First valid remapping
        remappings.push(Remapping {
            context: None,
            name: "MyContract.sol".to_string(),
            path: "implementations/Contract1.sol".to_string(),
        });

        // Same source to different target (should be rejected)
        remappings.push(Remapping {
            context: None,
            name: "MyContract.sol".to_string(),
            path: "implementations/Contract2.sol".to_string(),
        });

        // Different source to same target (should be allowed)
        remappings.push(Remapping {
            context: None,
            name: "OtherContract.sol".to_string(),
            path: "implementations/Contract1.sol".to_string(),
        });

        // Exact duplicate (should be silently ignored)
        remappings.push(Remapping {
            context: None,
            name: "MyContract.sol".to_string(),
            path: "implementations/Contract1.sol".to_string(),
        });

        // Invalid .sol remapping (target not .sol)
        remappings.push(Remapping {
            context: None,
            name: "Invalid.sol".to_string(),
            path: "implementations/Contract1.txt".to_string(),
        });

        let result = remappings.into_inner();
        assert_eq!(result.len(), 2, "Should only have 2 valid remappings");

        // Verify the correct remappings exist
        assert!(
            result
                .iter()
                .any(|r| r.name == "MyContract.sol" && r.path == "implementations/Contract1.sol"),
            "Should keep first mapping of MyContract.sol"
        );
        assert!(
            !result
                .iter()
                .any(|r| r.name == "MyContract.sol" && r.path == "implementations/Contract2.sol"),
            "Should keep first mapping of MyContract.sol"
        );
        assert!(result.iter().any(|r| r.name == "OtherContract.sol" && r.path == "implementations/Contract1.sol"),
            "Should allow different source to same target");

        // Verify the rejected remapping doesn't exist
        assert!(
            !result
                .iter()
                .any(|r| r.name == "MyContract.sol" && r.path == "implementations/Contract2.sol"),
            "Should reject same source to different target"
        );
    }

    #[test]
    fn test_mixed_remappings() {
        let mut remappings = Remappings::new();

        remappings.push(Remapping {
            context: None,
            name: "@openzeppelin-contracts/".to_string(),
            path: "lib/openzeppelin-contracts/".to_string(),
        });
        remappings.push(Remapping {
            context: None,
            name: "@openzeppelin/contracts/".to_string(),
            path: "lib/openzeppelin/contracts/".to_string(),
        });

        remappings.push(Remapping {
            context: None,
            name: "MyContract.sol".to_string(),
            path: "os/Contract.sol".to_string(),
        });

        let result = remappings.into_inner();
        assert_eq!(result.len(), 3, "Should have 3 remappings");
        assert_eq!(result.first().unwrap().name, "@openzeppelin-contracts/");
        assert_eq!(result.first().unwrap().path, "lib/openzeppelin-contracts/");
        assert_eq!(result.get(1).unwrap().name, "@openzeppelin/contracts/");
        assert_eq!(result.get(1).unwrap().path, "lib/openzeppelin/contracts/");
        assert_eq!(result.get(2).unwrap().name, "MyContract.sol");
        assert_eq!(result.get(2).unwrap().path, "os/Contract.sol");
    }

    #[test]
    fn test_remappings_with_context() {
        let mut remappings = Remappings::new();

        // Same name but different contexts
        remappings.push(Remapping {
            context: Some("test/".to_string()),
            name: "MyContract.sol".to_string(),
            path: "test/Contract.sol".to_string(),
        });
        remappings.push(Remapping {
            context: Some("prod/".to_string()),
            name: "MyContract.sol".to_string(),
            path: "prod/Contract.sol".to_string(),
        });

        let result = remappings.into_inner();
        assert_eq!(result.len(), 2, "Should allow same name with different contexts");
        assert!(result
            .iter()
            .any(|r| r.context == Some("test/".to_string()) && r.path == "test/Contract.sol"));
        assert!(result
            .iter()
            .any(|r| r.context == Some("prod/".to_string()) && r.path == "prod/Contract.sol"));
    }
}
