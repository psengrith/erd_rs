use clap::Parser;
use erd_rs::formaters::{ClassDiagramFormater, MMDFormater, Vis};
use regex::Regex;
use std::{collections::HashMap, fs, ops::Index, path::PathBuf, process::Command};
use syn::{
    Expr, ImplItem, Item, ItemImpl, ItemStruct, Lit, Meta, Pat, ReturnType, Type, Visibility,
};
use tracing::{debug, error, warn};
use tracing_subscriber::{
    layer::{Layered, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

const RELATION_META_REGEX: &str =
    r"^(?:`#\[relation(?:\s+)?=(?:\s+)?)(.+)(?:\s+)?:(?:\s+)?(.+)\]`$";

/// A command to build Entity Relation (ER) diagram from Rust code.
#[derive(Parser)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Struct identifier name suffix or compiler flag to be included
    /// during parsing.
    #[arg(short, long, default_value_t = String::from("Model"))]
    suffix: String,

    /// Working directory.
    #[arg(short, long)]
    dir: Option<String>,

    /// Output file name.
    /// File extension should be either `.mmd` or `.md`.
    #[arg(short, long, default_value_t = String::from("ER.mmd"))]
    output: String,

    /// Diagram title.
    #[arg(short, long, default_value_t = String::from("ER Diagram"))]
    title: String,
}

fn main() -> anyhow::Result<()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let fmt_layer = tracing_subscriber::fmt::layer::<Layered<EnvFilter, Registry>>()
        .pretty()
        .with_target(false);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let args = Args::parse();
    let dir = {
        let mut pwd = args.dir.to_owned().unwrap_or("./".to_string());
        if !pwd.ends_with("/") {
            pwd += "/";
        }
        PathBuf::from(pwd)
    };
    let mut output = dir.clone();
    output.push(args.output.to_owned());

    if !dir.as_path().try_exists()? {
        panic!("Directory does not exist! ({:?})", dir);
    }

    debug!("suffix: {}", args.suffix);
    debug!("dir: {}", dir.as_path().display());
    debug!("output: {}", output.as_path().display());

    let mut final_md = vec![
        "---".to_string(),
        format!("title: {}", args.title),
        "---".to_string(),
        "classDiagram".to_string(),
    ];
    let fmt = MMDFormater();
    let mut clazz_map = HashMap::<String, (Vis, Vec<String>)>::new();
    let mut rel_map = HashMap::<String, String>::new();
    let mut methods_map = HashMap::<String, Vec<String>>::new();

    let o = Command::new("cargo")
        .arg("expand")
        .arg("--release")
        .current_dir(dir.to_owned())
        .output()?;

    if !o.status.success() {
        let msg = String::from_utf8_lossy(&o.stderr).to_string();
        error!("{msg}");
        return Err(anyhow::anyhow!(msg));
    }

    let content = String::from_utf8_lossy(&o.stdout);
    let ast = syn::parse_file(&content)?;

    parse_items(
        ast.items,
        &args,
        &fmt,
        &mut clazz_map,
        &mut rel_map,
        &mut methods_map,
    )?;

    final_md.extend(rel_map.into_values());
    for (clazz, (_, mut m)) in clazz_map.into_iter() {
        if let Some(methods) = methods_map.remove(&clazz) {
            m.extend(methods);
        }

        final_md.extend(m);
    }

    let mut final_md = final_md.join("\n");
    if output.extension().is_some() && output.extension().unwrap().eq("md") {
        final_md = format!("```mermaid\n{final_md}\n```")
    }
    fs::write(output, final_md)?;

    Ok(())
}

fn parse_items(
    items: Vec<Item>,
    args: &Args,
    fmt: &impl ClassDiagramFormater,
    clazz_map: &mut HashMap<String, (Vis, Vec<String>)>,
    rel_map: &mut HashMap<String, String>,
    methods_map: &mut HashMap<String, Vec<String>>,
) -> anyhow::Result<()> {
    let flag = args.suffix.to_string().to_lowercase();
    for item in items {
        match item {
            syn::Item::Struct(mut istruct) => {
                istruct.attrs.retain_mut(|a| a.path().is_ident(&flag));
                let clazz = istruct.ident.to_string();
                if !istruct.attrs.is_empty()
                    || clazz.ends_with(&args.suffix)
                    || args.suffix.is_empty()
                {
                    let mut m = clazz_map.get_mut(&clazz);
                    if m.is_none() {
                        clazz_map.insert(clazz.to_owned(), (Vis::default(), vec![]));
                        m = clazz_map.get_mut(&clazz);
                    }
                    let r = parse_fields(istruct, fmt, rel_map)?;
                    let m = m.unwrap();
                    m.0 = r.0;
                    m.1.extend(r.1);
                }
            }
            syn::Item::Impl(iimpl) => match iimpl.self_ty.as_ref() {
                Type::Path(ty) => {
                    let clazz = ty.path.segments.last().unwrap().ident.to_string();
                    let mut m = methods_map.get_mut(&clazz);
                    if m.is_none() {
                        methods_map.insert(clazz.to_owned(), vec![]);
                        m = methods_map.get_mut(&clazz);
                    }
                    let clazz_vis = clazz_map.get(&clazz).map(|v| v.0).unwrap_or_default();
                    m.unwrap().extend(parse_impl(iimpl, clazz, clazz_vis, fmt)?);
                }
                _ => {}
            },
            syn::Item::Mod(imod) => {
                if let Some((_, items)) = imod.content {
                    parse_items(items, args, fmt, clazz_map, rel_map, methods_map)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Parse struct fields syntax, and convert it to classDiagram markdown
/// describing attributes in the provided `item`.
fn parse_fields(
    mut item: ItemStruct,
    fmt: &impl ClassDiagramFormater,
    rel_map: &mut HashMap<String, String>,
) -> anyhow::Result<(Vis, Vec<String>)> {
    let clazz = item.ident.to_string();
    let clazz_vis = item.vis.into();
    let mut markdown = vec![fmt.format_class(clazz.to_owned())];
    let relation_regex = Regex::new(RELATION_META_REGEX)?;

    for i in 0..item.fields.len() {
        let field = item.fields.iter_mut().nth(i).unwrap();
        let prop = field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or(i.to_string());
        let (_, vis) = parse_vis(&field.vis, fmt, Vis::Private);

        match &field.ty {
            Type::Path(ty) => {
                let ty = ty.path.segments.last().unwrap().ident.to_string();
                markdown.push(fmt.format_field(vis, prop, ty.to_owned()));

                field.attrs.retain(|a| a.path().is_ident("doc"));
                if !field.attrs.is_empty() {
                    let doc = field.attrs.index(0);
                    let doc = match &doc.meta {
                        Meta::NameValue(doc) => match &doc.value {
                            Expr::Lit(v) => match &v.lit {
                                Lit::Str(v) => Some(v.value()),
                                _ => None,
                            },
                            _ => None,
                        },
                        _ => None,
                    };

                    if let Some(doc) = doc {
                        if let Some((_, [r_type, r_label])) =
                            relation_regex.captures(doc.as_str()).map(|c| c.extract())
                        {
                            if clazz < ty {
                                let key = format!("{clazz}-{ty}");
                                let rel = rel_map.get(&key);
                                if rel.is_none() {
                                    rel_map.insert(
                                        key,
                                        fmt.format_link(
                                            clazz.to_owned(),
                                            r_type.trim().to_string(),
                                            ty,
                                            "".to_string(),
                                            r_label.trim().to_string(),
                                        ),
                                    );
                                }
                            } else {
                                let key = format!("{ty}-{clazz}");
                                let rel = rel_map.get(&key);
                                if rel.is_none() {
                                    rel_map.insert(
                                        key,
                                        fmt.format_link(
                                            ty,
                                            "".to_string(),
                                            clazz.to_owned(),
                                            r_type.trim().to_string(),
                                            r_label.trim().to_string(),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                warn!(
                    "Struct `{}` contains unsupported field type! (field: `{}`)",
                    clazz, prop
                )
            }
        };
    }

    markdown.push(fmt.format_class_end());
    Ok((clazz_vis, markdown))
}

/// Parse struct impl syntax, and convert it to classDiagram markdown
/// describing implemented methods.
fn parse_impl(
    item: ItemImpl,
    clazz: String,
    clazz_vis: Vis,
    fmt: &impl ClassDiagramFormater,
) -> anyhow::Result<Vec<String>> {
    let mut markdown: Vec<String> = vec![];

    let default_vis = match &item.trait_ {
        Some(_) => clazz_vis,
        _ => Vis::Private,
    };

    for impl_item in item.items {
        match impl_item {
            ImplItem::Fn(impl_item) => {
                let (_, vis) = parse_vis(&impl_item.vis, fmt, default_vis);
                let method = impl_item.sig.ident.to_string();
                let inputs = impl_item
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|i| match i {
                        syn::FnArg::Typed(p_ty) => {
                            let path = match p_ty.pat.as_ref() {
                                Pat::Ident(p) => Some(p.ident.to_string()),
                                _ => None,
                            };
                            let ty = match p_ty.ty.as_ref() {
                                Type::Path(p) => {
                                    Some(p.path.segments.last().unwrap().ident.to_string())
                                }
                                _ => None,
                            };

                            if !path.is_some() && !ty.is_some() {
                                warn!("Failed to parse inputs of fn {method}");
                                None
                            } else {
                                Some((path.unwrap(), ty.unwrap()))
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<(String, String)>>();
                let output = match impl_item.sig.output {
                    ReturnType::Default => "".to_string(),
                    ReturnType::Type(_, ty) => parse_return_type(ty.as_ref(), &clazz)?,
                };
                markdown.push(fmt.format_fn(clazz.to_owned(), vis, method, inputs, output));
            }
            _ => {}
        }
    }

    Ok(markdown)
}

fn parse_vis(vis: &Visibility, fmt: &impl ClassDiagramFormater, inherit_vis: Vis) -> (Vis, String) {
    match vis {
        Visibility::Restricted(res) => {
            if res.path.is_ident("self") {
                (Vis::Private, fmt.format_vis(Vis::Private))
            } else {
                (Vis::Internal, fmt.format_vis(Vis::Internal))
            }
        }
        Visibility::Inherited => (inherit_vis, fmt.format_vis(inherit_vis)),
        _ => (Vis::Public, fmt.format_vis(Vis::Public)),
    }
}

fn parse_return_type(ty: &Type, self_clazz: &str) -> anyhow::Result<String> {
    if let Type::Path(ty) = ty {
        Ok(ty.path.segments.last().unwrap().ident.to_string())
    } else if let Type::Reference(ty) = ty {
        Ok(format!("&{}", parse_return_type(&ty.elem, self_clazz)?))
    } else if let Type::Ptr(ty) = ty {
        if ty.const_token.is_some() {
            Ok(format!(
                "*const {}",
                parse_return_type(&ty.elem, self_clazz)?
            ))
        } else if ty.mutability.is_some() {
            Ok(format!(
                "*const {}",
                parse_return_type(&ty.elem, self_clazz)?
            ))
        } else {
            Ok(format!("*{}", parse_return_type(&ty.elem, self_clazz)?))
        }
    } else if let Type::Paren(ty) = ty {
        Ok(format!("({})", parse_return_type(&ty.elem, self_clazz)?))
    } else if let Type::Array(ty) = ty {
        if let Expr::Lit(l) = &ty.len {
            if let Lit::Int(n) = &l.lit {
                return Ok(format!(
                    "[{}; {}]",
                    parse_return_type(&ty.elem, self_clazz)?,
                    n.to_string()
                ));
            }
        }
        Err(anyhow::anyhow!("Expected ReturnType!"))
    } else {
        Err(anyhow::anyhow!("Expected ReturnType!"))
    }
    .map(|r_ty| r_ty.as_str().replace("Self", &self_clazz))
}

/*
#[cfg(test)]
mod test {

    #[test]
    fn test() {
        let code = r#"
struct TestModel {
    name: String,
    count: u32,
    #[doc = "`#[relation = 1..n : belong_to]`"]
    rel_id: RelModel,
}

impl Default for TestModel {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            count: 0,
            rel_id: RelModel::new()
        }
    }
}

struct RelModel {}

impl RelModel {
    fn new() -> Self {
        Self {}
    }
}"#;
    }
}
*/
