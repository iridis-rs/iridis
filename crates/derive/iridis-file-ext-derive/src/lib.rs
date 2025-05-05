extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, ImplItem, ItemImpl, ReturnType, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro_derive(FileExtPlugin)]
pub fn derive_file_ext_plugin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        #[cfg(feature = "cdylib")]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub static IRIDIS_FILE_EXT_PLUGIN: DynamicallyLinkedFileExtPluginInstance =
            || <#name>::new();

        static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<iridis_file_ext::prelude::thirdparty::tokio::runtime::Runtime> =
            std::sync::LazyLock::new(|| {
                iridis_file_ext::prelude::thirdparty::tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
            });

        fn default_runtime<T: Send + 'static>(
            task: impl Future<Output = T> + Send + 'static,
        ) -> iridis_file_ext::prelude::thirdparty::tokio::task::JoinHandle<T> {
            match iridis_file_ext::prelude::thirdparty::tokio::runtime::Handle::try_current() {
                Ok(handle) => handle.spawn(task),
                Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task),
            }
        }
    };

    TokenStream::from(expanded)
}

struct MacroArgs {
    runtime: String,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut runtime = String::new();

        let vars = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;

        for var in vars {
            if let syn::Meta::NameValue(name_value) = var {
                let name = name_value.path.get_ident().unwrap().to_string();

                if name == "runtime" {
                    if let syn::Expr::Lit(lit) = &name_value.value {
                        if let syn::Lit::Str(lit_str) = &lit.lit {
                            runtime = lit_str.value();
                        }
                    }
                }
            }
        }

        Ok(MacroArgs { runtime })
    }
}

#[proc_macro_attribute]
pub fn file_ext_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(item as ItemImpl);

    let args = parse_macro_input!(attr as MacroArgs);
    let runtime_tokens = args.runtime.parse::<proc_macro2::TokenStream>().unwrap();

    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item {
            let was_async = method.sig.asyncness.is_some();
            method.sig.asyncness = None;

            let old_block = method.block.clone();

            if was_async {
                let old_return_type = match &method.sig.output {
                    ReturnType::Default => quote! { () },
                    ReturnType::Type(_, ty) => {
                        if method.sig.ident == "new" {
                            quote! { iridis_file_ext::prelude::thirdparty::eyre::Result<Box<dyn FileExtPlugin>> }
                        } else {
                            quote! { #ty }
                        }
                    }
                };

                method.sig.output = syn::parse_quote! {
                    -> iridis_file_ext::prelude::thirdparty::tokio::task::JoinHandle<#old_return_type>
                };

                if method.sig.ident == "new" {
                    method.block = syn::parse_quote! {
                        {
                            #runtime_tokens(async move {
                                #old_block.map(|node| Box::new(node) as Box<dyn FileExtPlugin>)
                            })
                        }
                    };
                } else if method.sig.ident == "load" {
                    method.block = syn::parse_quote! {
                        {
                            #runtime_tokens(async move {
                                #old_block
                            })
                        }
                    };
                }
            }
        }
    }

    quote! {
        #impl_block
    }
    .into()
}
