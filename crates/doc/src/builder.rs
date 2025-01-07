use crate::{
    document::DocumentContent, helpers::merge_toml_table, AsDoc, BufWriter, Document, ParseItem,
    ParseSource, Parser, Preprocessor,
};
use alloy_primitives::map::HashMap;
use forge_fmt::{FormatterConfig, Visitable};
use foundry_compilers::{compilers::solc::SOLC_EXTENSIONS, utils::source_files_iter};
use foundry_config::{filter::expand_globs, DocConfig};
use itertools::Itertools;
use mdbook::MDBook;
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};
use toml::value;

/// Build Solidity documentation for a project from natspec comments.
/// The builder parses the source files using [Parser],
/// then formats and writes the elements as the output.
#[derive(Debug)]
pub struct DocBuilder {
    /// The project root
    pub root: PathBuf,
    /// Path to Solidity source files.
    pub sources: PathBuf,
    /// Paths to external libraries.
    pub libraries: Vec<PathBuf>,
    /// Flag whether to build mdbook.
    pub should_build: bool,
    /// Documentation configuration.
    pub config: DocConfig,
    /// The array of preprocessors to apply.
    pub preprocessors: Vec<Box<dyn Preprocessor>>,
    /// The formatter config.
    pub fmt: FormatterConfig,
    /// Whether to include libraries to the output.
    pub include_libraries: bool,
}

// TODO: consider using `tfio`
impl DocBuilder {
    pub(crate) const SRC: &'static str = "src";
    const SOL_EXT: &'static str = "sol";
    const README: &'static str = "README.md";
    const SUMMARY: &'static str = "SUMMARY.md";

    /// Create new instance of builder.
    pub fn new(
        root: PathBuf,
        sources: PathBuf,
        libraries: Vec<PathBuf>,
        include_libraries: bool,
    ) -> Self {
        Self {
            root,
            sources,
            libraries,
            include_libraries,
            should_build: false,
            config: DocConfig::default(),
            preprocessors: Default::default(),
            fmt: Default::default(),
        }
    }

    /// Set `should_build` flag on the builder
    pub fn with_should_build(mut self, should_build: bool) -> Self {
        self.should_build = should_build;
        self
    }

    /// Set config on the builder.
    pub fn with_config(mut self, config: DocConfig) -> Self {
        self.config = config;
        self
    }

    /// Set formatter config on the builder.
    pub fn with_fmt(mut self, fmt: FormatterConfig) -> Self {
        self.fmt = fmt;
        self
    }

    /// Set preprocessors on the builder.
    pub fn with_preprocessor<P: Preprocessor + 'static>(mut self, preprocessor: P) -> Self {
        self.preprocessors.push(Box::new(preprocessor) as Box<dyn Preprocessor>);
        self
    }

    /// Get the output directory
    pub fn out_dir(&self) -> PathBuf {
        self.root.join(&self.config.out)
    }

    /// Parse the sources and build the documentation.
    pub fn build(self) -> eyre::Result<()> {
        // Expand ignore globs
        let ignored = expand_globs(&self.root, self.config.ignore.iter())?;

        // Collect and parse source files
        let sources = source_files_iter(&self.sources, SOLC_EXTENSIONS)
            .filter(|file| !ignored.contains(file))
            .collect::<Vec<_>>();

        if sources.is_empty() {
            sh_println!("No sources detected at {}", self.sources.display())?;
            return Ok(())
        }

        let library_sources = self
            .libraries
            .iter()
            .flat_map(|lib| source_files_iter(lib, SOLC_EXTENSIONS))
            .collect::<Vec<_>>();

        let combined_sources = sources
            .iter()
            .map(|path| (path, false))
            .chain(library_sources.iter().map(|path| (path, true)))
            .collect::<Vec<_>>();

        let documents = combined_sources
            .par_iter()
            .enumerate()
            .map(|(i, (path, from_library))| {
                let path = *path;
                let from_library = *from_library;

                // Read and parse source file
                let source = fs::read_to_string(path)?;

                let (mut source_unit, comments) = match solang_parser::parse(&source, i) {
                    Ok(res) => res,
                    Err(err) => {
                        if from_library {
                            // Ignore failures for library files
                            return Ok(Vec::new());
                        } else {
                            return Err(eyre::eyre!(
                                "Failed to parse Solidity code for {}\nDebug info: {:?}",
                                path.display(),
                                err
                            ));
                        }
                    }
                };

                // Visit the parse tree
                let mut doc = Parser::new(comments, source).with_fmt(self.fmt.clone());
                source_unit
                    .visit(&mut doc)
                    .map_err(|err| eyre::eyre!("Failed to parse source: {err}"))?;

                // Split the parsed items on top-level constants and rest.
                let (items, consts): (Vec<ParseItem>, Vec<ParseItem>) = doc
                    .items()
                    .into_iter()
                    .partition(|item| !matches!(item.source, ParseSource::Variable(_)));

                // Attempt to group overloaded top-level functions
                let mut remaining = Vec::with_capacity(items.len());
                let mut funcs: HashMap<String, Vec<ParseItem>> = HashMap::default();
                for item in items {
                    if matches!(item.source, ParseSource::Function(_)) {
                        funcs.entry(item.source.ident()).or_default().push(item);
                    } else {
                        // Put the item back
                        remaining.push(item);
                    }
                }
                let (items, overloaded): (
                    HashMap<String, Vec<ParseItem>>,
                    HashMap<String, Vec<ParseItem>>,
                ) = funcs.into_iter().partition(|(_, v)| v.len() == 1);
                remaining.extend(items.into_iter().flat_map(|(_, v)| v));

                // Each regular item will be written into its own file.
                let mut files = remaining
                    .into_iter()
                    .map(|item| {
                        let relative_path = path.strip_prefix(&self.root)?.join(item.filename());
                        let target_path = self.config.out.join(Self::SRC).join(relative_path);
                        let ident = item.source.ident();
                        Ok(Document::new(
                            path.clone(),
                            target_path,
                            from_library,
                            self.config.out.clone(),
                        )
                        .with_content(DocumentContent::Single(item), ident))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?;

                // If top-level constants exist, they will be written to the same file.
                if !consts.is_empty() {
                    let filestem = path.file_stem().and_then(|stem| stem.to_str());

                    let filename = {
                        let mut name = "constants".to_owned();
                        if let Some(stem) = filestem {
                            name.push_str(&format!(".{stem}"));
                        }
                        name.push_str(".md");
                        name
                    };
                    let relative_path = path.strip_prefix(&self.root)?.join(filename);
                    let target_path = self.config.out.join(Self::SRC).join(relative_path);

                    let identity = match filestem {
                        Some(stem) if stem.to_lowercase().contains("constants") => stem.to_owned(),
                        Some(stem) => format!("{stem} constants"),
                        None => "constants".to_owned(),
                    };

                    files.push(
                        Document::new(
                            path.clone(),
                            target_path,
                            from_library,
                            self.config.out.clone(),
                        )
                        .with_content(DocumentContent::Constants(consts), identity),
                    )
                }

                // If overloaded functions exist, they will be written to the same file
                if !overloaded.is_empty() {
                    for (ident, funcs) in overloaded {
                        let filename = funcs.first().expect("no overloaded functions").filename();
                        let relative_path = path.strip_prefix(&self.root)?.join(filename);
                        let target_path = self.config.out.join(Self::SRC).join(relative_path);
                        files.push(
                            Document::new(
                                path.clone(),
                                target_path,
                                from_library,
                                self.config.out.clone(),
                            )
                            .with_content(DocumentContent::OverloadedFunctions(funcs), ident),
                        );
                    }
                }

                Ok(files)
            })
            .collect::<eyre::Result<Vec<_>>>()?;

        // Flatten results and apply preprocessors to files
        let documents = self
            .preprocessors
            .iter()
            .try_fold(documents.into_iter().flatten().collect_vec(), |docs, p| {
                p.preprocess(docs)
            })?;

        // Sort the results
        let documents = documents.into_iter().sorted_by(|doc1, doc2| {
            doc1.item_path.display().to_string().cmp(&doc2.item_path.display().to_string())
        });

        // Write mdbook related files
        self.write_mdbook(
            documents.filter(|d| !d.from_library || self.include_libraries).collect_vec(),
        )?;

        // Build the book if requested
        if self.should_build {
            MDBook::load(self.out_dir())
                .and_then(|book| book.build())
                .map_err(|err| eyre::eyre!("failed to build book: {err:?}"))?;
        }

        Ok(())
    }

    fn write_mdbook(&self, documents: Vec<Document>) -> eyre::Result<()> {
        let out_dir = self.out_dir();
        let out_dir_src = out_dir.join(Self::SRC);
        fs::create_dir_all(&out_dir_src)?;

        // Write readme content if any
        let homepage_content = {
            // Default to the homepage README if it's available.
            // If not, use the src README as a fallback.
            let homepage_or_src_readme = self
                .config
                .homepage
                .as_ref()
                .map(|homepage| self.root.join(homepage))
                .unwrap_or_else(|| self.sources.join(Self::README));
            // Grab the root readme.
            let root_readme = self.root.join(Self::README);

            // Check to see if there is a 'homepage' option specified in config.
            // If not, fall back to src and root readme files, in that order.
            if homepage_or_src_readme.exists() {
                fs::read_to_string(homepage_or_src_readme)?
            } else if root_readme.exists() {
                fs::read_to_string(root_readme)?
            } else {
                String::new()
            }
        };

        let readme_path = out_dir_src.join(Self::README);
        fs::write(readme_path, homepage_content)?;

        // Write summary and section readmes
        let mut summary = BufWriter::default();
        summary.write_title("Summary")?;
        summary.write_link_list_item("Home", Self::README, 0)?;
        self.write_summary_section(&mut summary, &documents.iter().collect::<Vec<_>>(), None, 0)?;
        fs::write(out_dir_src.join(Self::SUMMARY), summary.finish())?;

        // Write solidity syntax highlighting
        fs::write(out_dir.join("solidity.min.js"), include_str!("../static/solidity.min.js"))?;

        // Write css files
        fs::write(out_dir.join("book.css"), include_str!("../static/book.css"))?;

        // Write book config
        fs::write(self.out_dir().join("book.toml"), self.book_config()?)?;

        // Write .gitignore
        let gitignore = "book/";
        fs::write(self.out_dir().join(".gitignore"), gitignore)?;

        // Write doc files
        for document in documents {
            fs::create_dir_all(
                document
                    .target_path
                    .parent()
                    .ok_or_else(|| eyre::format_err!("empty target path; noop"))?,
            )?;
            fs::write(&document.target_path, document.as_doc()?)?;
        }

        Ok(())
    }

    fn book_config(&self) -> eyre::Result<String> {
        // Read the default book first
        let mut book: value::Table = toml::from_str(include_str!("../static/book.toml"))?;
        book["book"]
            .as_table_mut()
            .unwrap()
            .insert(String::from("title"), self.config.title.clone().into());
        if let Some(ref repo) = self.config.repository {
            book["output"].as_table_mut().unwrap()["html"]
                .as_table_mut()
                .unwrap()
                .insert(String::from("git-repository-url"), repo.clone().into());
        }

        // Attempt to find the user provided book path
        let book_path = {
            if self.config.book.is_file() {
                Some(self.config.book.clone())
            } else {
                let book_path = self.config.book.join("book.toml");
                if book_path.is_file() {
                    Some(book_path)
                } else {
                    None
                }
            }
        };

        // Merge two book configs
        if let Some(book_path) = book_path {
            merge_toml_table(&mut book, toml::from_str(&fs::read_to_string(book_path)?)?);
        }

        Ok(toml::to_string_pretty(&book)?)
    }

    fn write_summary_section(
        &self,
        summary: &mut BufWriter,
        files: &[&Document],
        base_path: Option<&Path>,
        depth: usize,
    ) -> eyre::Result<()> {
        if files.is_empty() {
            return Ok(())
        }

        if let Some(path) = base_path {
            let title = path.iter().last().unwrap().to_string_lossy();
            if depth == 1 {
                summary.write_title(&title)?;
            } else {
                let summary_path = path.join(Self::README);
                summary.write_link_list_item(
                    &format!("❱ {title}"),
                    &summary_path.display().to_string(),
                    depth - 1,
                )?;
            }
        }

        // Group entries by path depth
        let mut grouped = HashMap::new();
        for file in files {
            let path = file.item_path.strip_prefix(&self.root)?;
            let key = path.iter().take(depth + 1).collect::<PathBuf>();
            grouped.entry(key).or_insert_with(Vec::new).push(*file);
        }
        // Sort entries by path depth
        let grouped = grouped.into_iter().sorted_by(|(lhs, _), (rhs, _)| {
            let lhs_at_end = lhs.extension().map(|ext| ext == Self::SOL_EXT).unwrap_or_default();
            let rhs_at_end = rhs.extension().map(|ext| ext == Self::SOL_EXT).unwrap_or_default();
            if lhs_at_end == rhs_at_end {
                lhs.cmp(rhs)
            } else if lhs_at_end {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        let mut readme = BufWriter::new("\n\n# Contents\n");
        for (path, files) in grouped {
            if path.extension().map(|ext| ext == Self::SOL_EXT).unwrap_or_default() {
                for file in files {
                    let ident = &file.identity;

                    let summary_path = file
                        .target_path
                        .strip_prefix(self.out_dir().strip_prefix(&self.root)?.join(Self::SRC))?;
                    summary.write_link_list_item(
                        ident,
                        &summary_path.display().to_string(),
                        depth,
                    )?;

                    let readme_path = base_path
                        .map(|path| summary_path.strip_prefix(path))
                        .transpose()?
                        .unwrap_or(summary_path);
                    readme.write_link_list_item(ident, &readme_path.display().to_string(), 0)?;
                }
            } else {
                // Generate a relative path to "index.html" based on the current directory depth and
                let name = path.iter().last().unwrap().to_string_lossy();
                let depth = path.components().count();
                let mut prefix = PathBuf::new();
                for _ in 0..depth {
                    prefix.push("..");
                }
                let readme_path = prefix.join(&path).join("index.html");
                let readme_path_str = readme_path.to_string_lossy();
                readme.write_link_list_item(&name, &readme_path_str, 0)?;
                self.write_summary_section(summary, &files, Some(&path), depth + 1)?;
            }
        }
        if !readme.is_empty() {
            if let Some(path) = base_path {
                let path = self.out_dir().join(Self::SRC).join(path);
                fs::create_dir_all(&path)?;
                fs::write(path.join(Self::README), readme.finish())?;
            }
        }
        Ok(())
    }
}
