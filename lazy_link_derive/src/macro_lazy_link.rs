use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_quote, parse_str,
    punctuated::Punctuated,
    spanned::Spanned,
    Abi, Error, FnArg, ForeignItem, ForeignItemFn, ItemFn, ItemForeignMod, Lit, LitStr,
    MetaNameValue, Pat, Path, Result, Signature, Token,
};

#[derive(Debug)]
enum CacheMode {
    Static,
    AtomicStatic,
    None,
}

impl TryFrom<&LitStr> for CacheMode {
    type Error = syn::Error;

    fn try_from(value: &LitStr) -> Result<Self> {
        let value = value.value();
        Ok(match value.as_str() {
            "static" => Self::Static,
            "static-atomic" => Self::AtomicStatic,
            "none" => Self::None,
            _ => {
                return Err(Error::new(
                    value.span(),
                    r#"expected one of the following values: static, static-atomic, none"#,
                ))
            }
        })
    }
}

#[derive(Debug)]
struct MacroArgs {
    resolver: Path,
    cache: CacheMode,
    module: Option<LitStr>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars: Punctuated<MetaNameValue, syn::token::Comma> =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut resolver = None;
        let mut cache = None;
        let mut module = None;

        for kv in &vars {
            if kv.path.is_ident("resolver") {
                let Lit::Str(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected a string"));
                };

                resolver = Some(value.parse()?);
            } else if kv.path.is_ident("cache") {
                let Lit::Str(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected a string"));
                };

                cache = Some(value.try_into()?);
            } else if kv.path.is_ident("module") {
                let Lit::Str(value) = &kv.lit else {
                    return Err(Error::new(kv.lit.span(), "expected a string"));
                };

                module = Some(value.clone());
            } else {
                return Err(Error::new(kv.path.span(), "unknown attribute"));
            }
        }

        Ok(Self {
            resolver: resolver.ok_or(Error::new(
                vars.span(),
                "missing resolver = \"...\" attribute",
            ))?,
            cache: cache.unwrap_or(CacheMode::Static),
            module,
        })
    }
}

pub fn lazy_link(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = parse2::<MacroArgs>(attr)?;
    let input = parse2::<ItemForeignMod>(input)?;

    let mut result = Vec::with_capacity(input.items.len());
    for item in input.items {
        match item {
            ForeignItem::Fn(function) => {
                result.push(process_external_function(&args, &input.abi, function)?);
            }
            item => {
                return Err(Error::new(
                    item.span(),
                    "only functions can be lazily imported",
                ))
            }
        }
    }

    Ok(quote! {
        #(#result)*
    })
}

fn process_external_function(
    args: &MacroArgs,
    abi: &Abi,
    function: ForeignItemFn,
) -> Result<TokenStream> {
    let fn_name_str = function.sig.ident.to_string();
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;

    let module = if let Some(module) = &args.module {
        quote! { Some(#module) }
    } else {
        quote! { None }
    };

    let fn_args = function
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(index, _)| parse_str(&format!("arg{}", index)))
        .collect::<Result<Vec<Pat>>>()?;

    let cache = parse_str::<Pat>(&format!(
        "lazy_link::{}",
        match &args.cache {
            CacheMode::Static => "StaticCache",
            CacheMode::AtomicStatic => "StaticAtomicCache",
            CacheMode::None => "NoCache",
        }
    ))?;

    let resolver = &args.resolver;
    let block = parse_quote! {{
        use core::sync::atomic::Ordering;
        use lazy_link::Cache;

        type TargetFn = #abi fn(#fn_inputs) #fn_output;
        static CACHE: #cache = #cache ::new();

        let fn_ptr = CACHE.resolve(|| (#resolver)(#module, #fn_name_str));
        let target_fn: TargetFn = core::mem::transmute(fn_ptr.as_ptr());
        (target_fn)(#(#fn_args),*)
    }};

    Ok(ItemFn {
        sig: Signature {
            unsafety: Some(Default::default()),
            inputs: function
                .sig
                .inputs
                .into_iter()
                .enumerate()
                .map(|(index, arg)| {
                    if let FnArg::Typed(mut arg) = arg {
                        arg.pat = parse_str(&format!("arg{}", index))?;
                        Ok(FnArg::Typed(arg))
                    } else {
                        Err(Error::new(arg.span(), "expected only typed arguments"))
                    }
                })
                .collect::<Result<_>>()?,
            ..function.sig
        },
        attrs: function.attrs,
        vis: function.vis,
        block,
    }
    .into_token_stream())
}
