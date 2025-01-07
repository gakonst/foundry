use crate::{
    document::{read_context, DocumentContent},
    parser::ParseSource,
    writer::BufWriter,
    CommentTag, Comments, CommentsRef, Document, Markdown, PreprocessorOutput,
    CONTRACT_INHERITANCE_ID, DEPLOYMENTS_ID, GIT_SOURCE_ID, INHERITDOC_ID,
};
use forge_fmt::solang_ext::SafeUnwrap;
use itertools::Itertools;
use solang_parser::pt::{Base, FunctionDefinition};
use std::path::{Path, PathBuf};

/// The result of [`AsDoc::as_doc`].
pub type AsDocResult = Result<String, std::fmt::Error>;

/// A trait for formatting a parse unit as documentation.
pub trait AsDoc {
    /// Formats a parse tree item into a doc string.
    fn as_doc(&self) -> AsDocResult;
}

impl AsDoc for String {
    fn as_doc(&self) -> AsDocResult {
        Ok(self.to_owned())
    }
}

impl AsDoc for Comments {
    fn as_doc(&self) -> AsDocResult {
        CommentsRef::from(self).as_doc()
    }
}

impl AsDoc for CommentsRef<'_> {
    // TODO: support other tags
    fn as_doc(&self) -> AsDocResult {
        let mut writer = BufWriter::default();

        // Write author tag(s)
        let authors = self.include_tag(CommentTag::Author);
        if !authors.is_empty() {
            writer.write_bold(&format!("Author{}:", if authors.len() == 1 { "" } else { "s" }))?;
            writer.writeln_raw(authors.iter().map(|a| &a.value).join(", "))?;
            writer.writeln()?;
        }

        // Write notice tags
        let notices = self.include_tag(CommentTag::Notice);
        for n in notices.iter() {
            writer.writeln_raw(&n.value)?;
            writer.writeln()?;
        }

        // Write dev tags
        let devs = self.include_tag(CommentTag::Dev);
        for d in devs.iter() {
            writer.write_italic(&d.value)?;
            writer.writeln()?;
        }

        // Write custom tags
        let customs = self.get_custom_tags();
        if !customs.is_empty() {
            writer.write_bold(&format!("Note{}:", if customs.len() == 1 { "" } else { "s" }))?;
            for c in customs.iter() {
                writer.writeln_raw(format!(
                    "{}{}: {}",
                    if customs.len() == 1 { "" } else { "- " },
                    &c.tag,
                    &c.value
                ))?;
                writer.writeln()?;
            }
        }

        Ok(writer.finish())
    }
}

impl AsDoc for Base {
    fn as_doc(&self) -> AsDocResult {
        Ok(self.name.identifiers.iter().map(|ident| ident.name.to_owned()).join("."))
    }
}

impl AsDoc for Document {
    fn as_doc(&self) -> AsDocResult {
        let mut writer = BufWriter::default();

        match &self.content {
            DocumentContent::OverloadedFunctions(items) => {
                writer
                    .write_title(&format!("function {}", items.first().unwrap().source.ident()))?;
                if let Some(git_source) = read_context!(self, GIT_SOURCE_ID, GitSource) {
                    writer.write_link("Git Source", &git_source)?;
                    writer.writeln()?;
                }

                for item in items.iter() {
                    let func = item.as_function().unwrap();
                    let mut heading = item.source.ident();
                    if !func.params.is_empty() {
                        heading.push_str(&format!(
                            "({})",
                            func.params
                                .iter()
                                .map(|p| p.1.as_ref().map(|p| p.ty.to_string()).unwrap_or_default())
                                .join(", ")
                        ));
                    }
                    writer.write_heading(&heading)?;
                    writer.write_section(&item.comments, &item.code)?;
                }
            }
            DocumentContent::Constants(items) => {
                writer.write_title("Constants")?;
                if let Some(git_source) = read_context!(self, GIT_SOURCE_ID, GitSource) {
                    writer.write_link("Git Source", &git_source)?;
                    writer.writeln()?;
                }

                for item in items.iter() {
                    let var = item.as_variable().unwrap();
                    writer.write_heading(&var.name.safe_unwrap().name)?;
                    writer.write_section(&item.comments, &item.code)?;
                }
            }
            DocumentContent::Single(item) => {
                writer.write_title(&item.source.ident())?;
                if let Some(git_source) = read_context!(self, GIT_SOURCE_ID, GitSource) {
                    writer.write_link("Git Source", &git_source)?;
                    writer.writeln()?;
                }

                if let Some(deployments) = read_context!(self, DEPLOYMENTS_ID, Deployments) {
                    writer.write_deployments_table(deployments)?;
                }

                match &item.source {
                    ParseSource::Contract(contract) => {
                        if !contract.base.is_empty() {
                            writer.write_bold("Inherits:")?;

                            // we need this to find the _relative_ paths
                            let src_target_dir = self.target_src_dir();

                            let mut bases = vec![];
                            let linked =
                                read_context!(self, CONTRACT_INHERITANCE_ID, ContractInheritance);
                            for base in contract.base.iter() {
                                let base_doc = base.as_doc()?;
                                let base_ident = &base.name.identifiers.last().unwrap().name;

                                let link = linked
                                    .as_ref()
                                    .and_then(|link| {
                                        link.get(base_ident).map(|path| {
                                            let path = Path::new("/").join(
                                                path.strip_prefix(&src_target_dir)
                                                    .ok()
                                                    .unwrap_or(path),
                                            );
                                            Markdown::Link(
                                                &base_doc,
                                                &format!(
                                                    "../../../{}",
                                                    path.display()
                                                        .to_string()
                                                        .trim_start_matches('/')
                                                ),
                                            )
                                            .as_doc()
                                        })
                                    })
                                    .transpose()?
                                    .unwrap_or(base_doc);

                                bases.push(link);
                            }

                            writer.writeln_raw(bases.join(", "))?;
                            writer.writeln()?;
                        }

                        writer.writeln_doc(&item.comments)?;

                        if let Some(state_vars) = item.variables() {
                            writer.write_subtitle("State Variables")?;
                            state_vars.into_iter().try_for_each(|(item, comments, code)| {
                                let comments = comments.merge_inheritdoc(
                                    &item.name.safe_unwrap().name,
                                    read_context!(self, INHERITDOC_ID, Inheritdoc),
                                );

                                writer.write_heading(&item.name.safe_unwrap().name)?;
                                writer.write_section(&comments, code)?;
                                writer.writeln()
                            })?;
                        }

                        if let Some(funcs) = item.functions() {
                            writer.write_subtitle("Functions")?;

                            for (func, comments, code) in funcs.iter() {
                                self.write_function(&mut writer, func, comments, code)?;
                            }
                        }

                        if let Some(events) = item.events() {
                            writer.write_subtitle("Events")?;
                            events.into_iter().try_for_each(|(item, comments, code)| {
                                writer.write_heading(&item.name.safe_unwrap().name)?;
                                writer.write_section(comments, code)?;
                                writer.try_write_events_table(&item.fields, comments)
                            })?;
                        }

                        if let Some(errors) = item.errors() {
                            writer.write_subtitle("Errors")?;
                            errors.into_iter().try_for_each(|(item, comments, code)| {
                                writer.write_heading(&item.name.safe_unwrap().name)?;
                                writer.write_section(comments, code)?;
                                writer.try_write_errors_table(&item.fields, comments)
                            })?;
                        }

                        if let Some(structs) = item.structs() {
                            writer.write_subtitle("Structs")?;
                            structs.into_iter().try_for_each(|(item, comments, code)| {
                                writer.write_heading(&item.name.safe_unwrap().name)?;
                                writer.write_section(comments, code)?;
                                writer.try_write_properties_table(&item.fields, comments)
                            })?;
                        }

                        if let Some(enums) = item.enums() {
                            writer.write_subtitle("Enums")?;
                            enums.into_iter().try_for_each(|(item, comments, code)| {
                                writer.write_heading(&item.name.safe_unwrap().name)?;
                                writer.write_section(comments, code)
                            })?;
                        }
                    }

                    ParseSource::Function(func) => {
                        // TODO: cleanup
                        // Write function docs
                        writer.writeln_doc(
                            &item.comments.exclude_tags(&[CommentTag::Param, CommentTag::Return]),
                        )?;

                        // Write function header
                        writer.write_code(&item.code)?;

                        // Write function parameter comments in a table
                        let params =
                            func.params.iter().filter_map(|p| p.1.as_ref()).collect::<Vec<_>>();
                        writer.try_write_param_table(CommentTag::Param, &params, &item.comments)?;

                        // Write function return parameter comments in a table
                        let returns =
                            func.returns.iter().filter_map(|p| p.1.as_ref()).collect::<Vec<_>>();
                        writer.try_write_param_table(
                            CommentTag::Return,
                            &returns,
                            &item.comments,
                        )?;

                        writer.writeln()?;
                    }

                    ParseSource::Struct(ty) => {
                        writer.write_section(&item.comments, &item.code)?;
                        writer.try_write_properties_table(&ty.fields, &item.comments)?;
                    }
                    ParseSource::Event(ev) => {
                        writer.write_section(&item.comments, &item.code)?;
                        writer.try_write_events_table(&ev.fields, &item.comments)?;
                    }
                    ParseSource::Error(err) => {
                        writer.write_section(&item.comments, &item.code)?;
                        writer.try_write_errors_table(&err.fields, &item.comments)?;
                    }
                    ParseSource::Variable(_) | ParseSource::Enum(_) | ParseSource::Type(_) => {
                        writer.write_section(&item.comments, &item.code)?;
                    }
                }
            }
            DocumentContent::Empty => (),
        };

        Ok(writer.finish())
    }
}

impl Document {
    /// Where all the source files are written to
    fn target_src_dir(&self) -> PathBuf {
        self.out_target_dir.join("src")
    }

    /// Writes a function to the buffer.
    fn write_function(
        &self,
        writer: &mut BufWriter,
        func: &FunctionDefinition,
        comments: &Comments,
        code: &str,
    ) -> Result<(), std::fmt::Error> {
        let func_name = func.name.as_ref().map_or(func.ty.to_string(), |n| n.name.to_owned());
        let comments =
            comments.merge_inheritdoc(&func_name, read_context!(self, INHERITDOC_ID, Inheritdoc));

        // Write function name
        writer.write_heading(&func_name)?;

        writer.writeln()?;

        // Write function docs
        writer.writeln_doc(&comments.exclude_tags(&[CommentTag::Param, CommentTag::Return]))?;

        // Write function header
        writer.write_code(code)?;

        // Write function parameter comments in a table
        let params = func.params.iter().filter_map(|p| p.1.as_ref()).collect::<Vec<_>>();
        writer.try_write_param_table(CommentTag::Param, &params, &comments)?;

        // Write function return parameter comments in a table
        let returns = func.returns.iter().filter_map(|p| p.1.as_ref()).collect::<Vec<_>>();
        writer.try_write_param_table(CommentTag::Return, &returns, &comments)?;

        writer.writeln()?;
        Ok(())
    }
}
