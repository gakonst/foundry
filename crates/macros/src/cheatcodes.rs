use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Result};

pub fn derive_cheatcode(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let name_s = name.to_string();
    match &input.data {
        Data::Struct(s) if name_s.ends_with("Call") => derive_call(name, s, &input.attrs),
        Data::Struct(_) if name_s.ends_with("Return") => Ok(TokenStream::new()),
        Data::Struct(s) => derive_struct(name, s, &input.attrs),
        Data::Enum(e) if name_s.ends_with("Calls") => derive_calls_enum(name, e),
        Data::Enum(e) if name_s.ends_with("Errors") => derive_errors_events_enum(name, e, false),
        Data::Enum(e) if name_s.ends_with("Events") => derive_errors_events_enum(name, e, true),
        Data::Enum(e) => derive_enum(name, e, &input.attrs),
        Data::Union(_) => Err(Error::new(name.span(), "unions are not supported")),
    }
}

/// Implements `CheatcodeDef` for a function call struct.
fn derive_call(name: &Ident, data: &DataStruct, attrs: &[Attribute]) -> Result<TokenStream> {
    let mut group = None::<Ident>;
    let mut status = None::<TokenStream>;
    let mut safety = None::<Ident>;
    let mut requires = None::<TokenStream>;
    for attr in attrs.iter().filter(|a| a.path().is_ident("cheatcode")) {
        attr.meta.require_list()?.parse_nested_meta(|meta| {
            let path = meta.path.get_ident().ok_or_else(|| meta.error("expected ident"))?;
            let path_s = path.to_string();
            match path_s.as_str() {
                "group" if group.is_none() => group = Some(meta.value()?.parse()?),
                "status" if status.is_none() => status = Some(meta.value()?.parse()?),
                "safety" if safety.is_none() => safety = Some(meta.value()?.parse()?),
                "requires" if requires.is_none() => requires = Some(meta.value()?.parse()?),
                _ => return Err(meta.error("unexpected attribute")),
            };
            Ok(())
        })?;
    }
    let group = group.ok_or_else(|| {
        syn::Error::new(name.span(), "missing #[cheatcode(group = ...)] attribute")
    })?;
    let status = status.unwrap_or_else(|| quote!(Stable));
    let safety = if let Some(safety) = safety {
        quote!(Safety::#safety)
    } else {
        quote! {
            match Group::#group.safety() {
                Some(s) => s,
                None => panic_unknown_safety(),
            }
        }
    };
    let requires = requires.unwrap_or_else(|| quote!(Requires::None));

    check_named_fields(data, name);

    let id = name.to_string();
    let id = id.strip_suffix("Call").expect("function struct ends in Call");

    let doc = get_docstring(attrs);
    let (signature, selector, declaration, description) = func_docstring(&doc);

    let mut params = declaration;
    if let Some(ret) = params.find(" returns ") {
        params = &params[..ret];
    }
    if params.contains(" memory ") {
        emit_warning!(
            name.span(),
            "parameter data locations must be `calldata` instead of `memory`"
        );
    }

    let (visibility, mutability) = parse_function_attrs(declaration, name.span())?;
    let visibility = Ident::new(visibility, Span::call_site());
    let mutability = Ident::new(mutability, Span::call_site());

    if description.is_empty() {
        emit_warning!(name.span(), "missing documentation for a cheatcode")
    }
    let description = description.replace("\n ", "\n");

    Ok(quote! {
        impl CheatcodeDef for #name {
            const CHEATCODE: &'static Cheatcode<'static> = &Cheatcode {
                func: Function {
                    id: #id,
                    description: #description,
                    declaration: #declaration,
                    visibility: Visibility::#visibility,
                    mutability: Mutability::#mutability,
                    signature: #signature,
                    selector: #selector,
                    selector_bytes: <Self as ::alloy_sol_types::SolCall>::SELECTOR,
                },
                group: Group::#group,
                status: Status::#status,
                safety: #safety,
                requires: #requires,
            };
        }
    })
}

/// Generates the `CHEATCODES` constant and implements `CheatcodeImpl` dispatch for an enum.
fn derive_calls_enum(name: &Ident, input: &syn::DataEnum) -> Result<TokenStream> {
    if input.variants.iter().any(|v| v.fields.len() != 1) {
        return Err(syn::Error::new(name.span(), "expected all variants to have a single field"))
    }

    // keep original order for matching
    let variant_names = input.variants.iter().map(|v| &v.ident);

    let mut variants = input.variants.iter().collect::<Vec<_>>();
    variants.sort_by(|a, b| a.ident.cmp(&b.ident));
    let variant_tys = variants.iter().map(|v| {
        assert_eq!(v.fields.len(), 1);
        &v.fields.iter().next().unwrap().ty
    });
    Ok(quote! {
        /// All the cheatcodes in [this contract](self).
        pub const CHEATCODES: &'static [&'static Cheatcode<'static>] = &[#(<#variant_tys as CheatcodeDef>::CHEATCODE,)*];

        /// Internal macro to implement the `Cheatcode` trait for the Vm calls enum.
        #[doc(hidden)]
        #[macro_export]
        macro_rules! vm_calls {
            ($mac:ident) => {
                $mac!(#(#variant_names),*)
            };
        }
    })
}

fn derive_errors_events_enum(
    name: &Ident,
    input: &syn::DataEnum,
    events: bool,
) -> Result<TokenStream> {
    if input.variants.iter().any(|v| v.fields.len() != 1) {
        return Err(syn::Error::new(name.span(), "expected all variants to have a single field"))
    }

    let (ident, ty_assoc_name, ty, doc) = if events {
        ("VM_EVENTS", "EVENT", "Event", "events")
    } else {
        ("VM_ERRORS", "ERROR", "Error", "custom errors")
    };
    let ident = Ident::new(ident, Span::call_site());
    let ty_assoc_name = Ident::new(ty_assoc_name, Span::call_site());
    let ty = Ident::new(ty, Span::call_site());
    let doc = format!("All the {doc} in [this contract](self).");

    let mut variants = input.variants.iter().collect::<Vec<_>>();
    variants.sort_by(|a, b| a.ident.cmp(&b.ident));
    let variant_tys = variants.iter().map(|v| {
        assert_eq!(v.fields.len(), 1);
        &v.fields.iter().next().unwrap().ty
    });
    Ok(quote! {
        #[doc = #doc]
        pub const #ident: &'static [&'static #ty<'static>] = &[#(#variant_tys::#ty_assoc_name,)*];
    })
}

fn derive_struct(
    name: &Ident,
    input: &syn::DataStruct,
    attrs: &[Attribute],
) -> Result<TokenStream> {
    let name_s = name.to_string();

    let doc = get_docstring(attrs);
    let doc = doc.trim();
    let kind = match () {
        () if doc.contains("Custom error ") => StructKind::Error,
        () if doc.contains("Event ") => StructKind::Event,
        _ => StructKind::Struct,
    };

    let (doc, def) = doc.split_once("```solidity\n").expect("bad docstring");
    let mut doc = doc.trim_end();
    let def_end = def.rfind("```").expect("bad docstring");
    let def = def[..def_end].trim();

    match kind {
        StructKind::Error => doc = &doc[..doc.find("Custom error ").expect("bad doc")],
        StructKind::Event => doc = &doc[..doc.find("Event ").expect("bad doc")],
        StructKind::Struct => {}
    }
    let doc = doc.trim();

    if doc.is_empty() {
        let n = match kind {
            StructKind::Error => "n",
            StructKind::Event => "n",
            StructKind::Struct => "",
        };
        emit_warning!(name.span(), "missing documentation for a{n} {}", kind.as_str());
    }

    if kind == StructKind::Struct {
        check_named_fields(input, name);
    }

    let def = match kind {
        StructKind::Struct => {
            let fields = input.fields.iter().map(|f| {
                let name = f.ident.as_ref().expect("field has no name").to_string();

                let to_find = format!("{name};");
                let ty_end = def.find(&to_find).expect("field not found in def");
                let ty = &def[..ty_end];
                let ty_start = ty.rfind(';').or_else(|| ty.find('{')).expect("bad struct def") + 1;
                let ty = ty[ty_start..].trim();
                if ty.is_empty() {
                    panic!("bad struct def: {def:?}")
                }

                let doc = get_docstring(&f.attrs);
                let doc = doc.trim();
                quote! {
                    StructField {
                        name: #name,
                        ty: #ty,
                        description: #doc,
                    }
                }
            });
            quote! {
                /// The struct definition.
                pub const STRUCT: &'static Struct<'static> = &Struct {
                    name: #name_s,
                    description: #doc,
                    fields: Cow::Borrowed(&[#(#fields),*]),
                };
            }
        }
        StructKind::Error => {
            quote! {
                /// The custom error definition.
                pub const ERROR: &'static Error<'static> = &Error {
                    name: #name_s,
                    description: #doc,
                    declaration: #def,
                };
            }
        }
        StructKind::Event => {
            quote! {
                /// The event definition.
                pub const EVENT: &'static Event<'static> = &Event {
                    name: #name_s,
                    description: #doc,
                    declaration: #def,
                };
            }
        }
    };
    Ok(quote! {
        impl #name {
            #def
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StructKind {
    Struct,
    Error,
    Event,
}

impl StructKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Error => "error",
            Self::Event => "event",
        }
    }
}

fn derive_enum(name: &Ident, input: &syn::DataEnum, attrs: &[Attribute]) -> Result<TokenStream> {
    let name_s = name.to_string();
    let doc = get_docstring(attrs);
    let doc_end = doc.find("```solidity").expect("bad docstring");
    let doc = doc[..doc_end].trim();
    if doc.is_empty() {
        emit_warning!(name.span(), "missing documentation for an enum");
    }
    let variants = input.variants.iter().filter(|v| v.discriminant.is_none()).map(|v| {
        let name = v.ident.to_string();
        let doc = get_docstring(&v.attrs);
        let doc = doc.trim();
        if doc.is_empty() {
            emit_warning!(v.ident.span(), "missing documentation for a variant");
        }
        quote! {
            EnumVariant {
                name: #name,
                description: #doc,
            }
        }
    });
    Ok(quote! {
        impl #name {
            /// The enum definition.
            pub const ENUM: &'static Enum<'static> = &Enum {
                name: #name_s,
                description: #doc,
                variants: Cow::Borrowed(&[#(#variants),*]),
            };
        }
    })
}

fn check_named_fields(data: &DataStruct, ident: &Ident) {
    for field in data.fields.iter() {
        if field.ident.is_none() {
            emit_warning!(ident, "all params must be named");
        }
    }
}

/// Flattens all the `#[doc = "..."]` attributes into a single string.
fn get_docstring(attrs: &[Attribute]) -> String {
    let mut doc = String::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue
        }
        let syn::Meta::NameValue(syn::MetaNameValue {
            value: syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }),
            ..
        }) = &attr.meta
        else {
            continue
        };

        let value = s.value();
        if !value.is_empty() {
            if !doc.is_empty() {
                doc.push('\n');
            }
            doc.push_str(&value);
        }
    }
    doc
}

/// Returns `(signature, hex_selector, declaration, description)` from a given `sol!`-generated
/// docstring for a function.
///
/// # Examples
///
/// The following docstring (string literals are joined with newlines):
/// ```text
/// "Function with signature `foo(uint256)` and selector `0x1234abcd`."
/// "```solidity"
/// "function foo(uint256 x) external view returns (bool y);"
/// "```"
/// "Description of the function."
/// ```
///
/// Will return:
/// ```text
/// (
///     "foo(uint256)",
///     "0x1234abcd",
///     "function foo(uint256 x) external view returns (bool y);",
///     "Description of the function."
/// )
/// ```
fn func_docstring(doc: &str) -> (&str, &str, &str, &str) {
    let expected_start = "Function with signature `";
    let start = doc.find(expected_start).expect("no auto docstring");
    let (descr_before, auto) = doc.split_at(start);

    let mut lines = auto.lines();
    let mut next = || lines.next().expect("unexpected end of docstring");

    let sig_line = next();
    let example_start = next();
    assert_eq!(example_start, "```solidity");
    let declaration = next();
    let example_end = next();
    assert_eq!(example_end, "```");

    let n = expected_start.len();
    let mut sig_end = n;
    sig_end += sig_line[n..].find('`').unwrap();
    let sig = &sig_line[n..sig_end];
    assert!(!sig.starts_with('`') && !sig.ends_with('`'));

    let selector_end = sig_line.rfind('`').unwrap();
    let selector = sig_line[sig_end..selector_end].strip_prefix("` and selector `").unwrap();
    assert!(!selector.starts_with('`') && !selector.ends_with('`'));
    assert!(selector.starts_with("0x"));

    let description = match doc.find("```\n") {
        Some(i) => &doc[i + 4..],
        None => descr_before,
    };

    (sig, selector, declaration, description.trim())
}

/// Returns `(visibility, mutability)` from a given Solidity function declaration.
fn parse_function_attrs(f: &str, span: Span) -> Result<(&str, &str)> {
    let Some(ext_start) = f.find("external") else {
        return Err(Error::new(span, "functions must have `external` visibility"))
    };
    let visibility = "External";

    let f = &f[ext_start..];
    let mutability = if f.contains("view") {
        "View"
    } else if f.contains("pure") {
        "Pure"
    } else {
        "None"
    };
    Ok((visibility, mutability))
}
