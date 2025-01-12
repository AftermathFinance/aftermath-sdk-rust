use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, DeriveInput, GenericParam, Generics, Path, TypeParamBound};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(move_))]
struct MoveAttributes {
    #[deluxe(rename = crate)]
    thecrate: Option<Path>,
    address: Option<String>,
    module: Option<Ident>,
    #[deluxe(default = false)]
    nameless: bool,
}

pub fn impl_move_struct(item: TokenStream) -> deluxe::Result<TokenStream> {
    // parse
    let mut ast: DeriveInput = syn::parse2(item)?;

    ensure_nonempty_struct(&ast)?;

    let MoveAttributes {
        thecrate,
        address,
        module,
        nameless,
    } = deluxe::extract_attributes(&mut ast)?;

    let thecrate: Path = thecrate
        .map(|c| parse_quote!(#c))
        .unwrap_or_else(|| parse_quote!(::af_move_type));
    let module = module.map(|i| i.to_string());

    let type_tag_params = type_tag_parameters(
        &ast,
        thecrate.clone(),
        address.clone(),
        module.clone(),
        nameless,
    );

    let type_tag_impl = impl_type_tag(type_tag_params.clone(), thecrate.clone());
    let move_struct_impl = impl_move_struct_(&ast, thecrate.clone(), type_tag_params);
    let static_address_impl = address
        .map(|a| impl_static_address(&ast, thecrate.clone(), a))
        .unwrap_or_else(|| quote!());
    let static_module_impl = module
        .map(|m| impl_static_module(&ast, thecrate.clone(), m))
        .unwrap_or_else(|| quote!());
    let static_name_impl = nameless
        .then(|| quote!())
        .unwrap_or_else(|| impl_static_name(&ast, thecrate.clone()));
    // There's always an implementation: either there are no type params or all type params can be
    // constrained to implement `StaticTypeTag`
    let static_type_params_impl = impl_static_type_params(&ast, thecrate);

    Ok(quote! {
        #type_tag_impl
        #move_struct_impl
        #static_address_impl
        #static_module_impl
        #static_name_impl
        #static_type_params_impl
    })
}

fn ensure_nonempty_struct(ast: &DeriveInput) -> deluxe::Result<()> {
    if let syn::Data::Struct(data) = &ast.data {
        if data.fields.is_empty() {
            return Err(syn::Error::new(
                ast.ident.span(),
                "Structs in Move have at least a (hidden) `dummy_field: bool` field.",
            ));
        }
    } else {
        return Err(syn::Error::new(
            ast.span(),
            "MoveStruct only defined for structs",
        ));
    };
    Ok(())
}

/// Implementation of the `_TypeTag` for the struct.
fn impl_type_tag(type_tag_params: TypeTagParameters, thecrate: Path) -> TokenStream {
    let TypeTagParameters {
        ident,
        attr_idents,
        attr_types,
        generics,
        struct_tag_var_attrs,
        struct_tag_const_attrs,
        struct_tag_const_vals,
        struct_tag_consts_checks,
        mut type_param_idents,
    } = type_tag_params;

    let attr_declarations: Vec<_> = attr_idents
        .iter()
        .zip(&attr_types)
        .map(|(ident, type_)| quote!(pub #ident: #type_))
        .collect();
    let struct_tag_const_declarations: Vec<_> = struct_tag_const_attrs
        .iter()
        .zip(&struct_tag_const_vals)
        .map(|(ident, val)| quote!(#ident: #val))
        .collect();

    type_param_idents.reverse();
    let unpack_type_params = quote!(
        // Unwrap here since we already checked the vector length
        #(
            let #type_param_idents = type_params
                .pop()
                .unwrap()
                .try_into()
                .map_err(#thecrate::TypeParamsError::from)?;)*
    );

    // define impl variables
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let type_tag_type = quote!(#thecrate::external::TypeTag);
    let struct_tag_type = quote!(#thecrate::external::StructTag);
    let result_type = quote!(::std::result::Result);
    let derive_ord = if has_type_params(&generics) {
        quote! {
            #[#thecrate::external::derive_where::derive_where(
                crate = #thecrate::external::derive_where
            )]
            #[derive_where(PartialOrd, Ord)]
        }
    } else {
        quote!(#[derive(PartialOrd, Ord)])
    };
    let serde_with_crate = quote!(#thecrate::external::serde_with).to_string();

    quote! {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            Hash,
            #thecrate::external::DeserializeFromStr,
            #thecrate::external::SerializeDisplay,
        )]
        #[serde_with(crate = #serde_with_crate)]
        #derive_ord
        pub struct #ident #generics {
            #(#attr_declarations),*
        }

        impl #impl_generics ::std::convert::From<#ident #type_generics> for #type_tag_type
        #where_clause
        {
            fn from(value: #ident #type_generics) -> Self {
                Self::Struct(::std::boxed::Box::new(value.into()))
            }
        }

        impl #impl_generics ::std::convert::From<#ident #type_generics> for #struct_tag_type
        #where_clause
        {
            fn from(value: #ident #type_generics) -> Self {
                let #ident {
                    #(#attr_idents),*
                } = value;
                Self {
                    #(#struct_tag_var_attrs,)*
                    #(#struct_tag_const_declarations),*
                }
            }
        }

        impl #impl_generics TryFrom<#type_tag_type> for #ident #type_generics
        #where_clause
        {
            type Error = #thecrate::TypeTagError;

            fn try_from(value: #type_tag_type) -> #result_type<Self, Self::Error> {
                match value {
                    #type_tag_type::Struct(stag) => #result_type::Ok((*stag).try_into()?),
                    other => #result_type::Err(#thecrate::TypeTagError::Variant {
                        expected: "Struct(_)".to_owned(),
                        got: other,
                    }),
                }
            }
        }

        impl #impl_generics TryFrom<#struct_tag_type> for #ident #type_generics
        #where_clause
        {
            type Error = #thecrate::StructTagError;

            fn try_from(value: #struct_tag_type) -> #result_type<Self, Self::Error> {
                use #thecrate::StructTagError::*;
                let #struct_tag_type {
                    address,
                    module,
                    name,
                    mut type_params,
                } = value;
                #struct_tag_consts_checks
                #unpack_type_params
                #result_type::Ok(Self {
                    #(#attr_idents),*
                })
            }
        }

        impl #impl_generics ::std::fmt::Display for #ident #type_generics
        #where_clause
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let stag: #struct_tag_type = self.clone().into();
                write!(f, "{}", stag)
            }
        }

        impl #impl_generics ::std::str::FromStr for #ident #type_generics
        #where_clause
        {
            type Err = #thecrate::ParseStructTagError;

            fn from_str(s: &str) -> #result_type<Self, Self::Err> {
                let stag: #struct_tag_type = s.parse()?;
                #result_type::Ok(stag.try_into()?)
            }
        }
    }
}

/// Main `impl` block for the struct and `MoveStruct` impl for it
fn impl_move_struct_(
    ast: &DeriveInput,
    thecrate: Path,
    type_tag_params: TypeTagParameters,
) -> TokenStream {
    let TypeTagParameters {
        ident: type_tag_ident,
        generics: type_tag_generics,
        attr_idents,
        attr_types,
        ..
    } = type_tag_params;

    // Remove the bounds from the type tag generics and construct the type tag type
    let type_tag_type = {
        let (_, type_generics, _) = type_tag_generics.split_for_impl();
        quote!(#type_tag_ident #type_generics)
    };

    // for use in function signatures
    let type_tag_fn_args: Vec<_> = attr_idents
        .iter()
        .zip(&attr_types)
        .map(|(name, ty)| quote!(#name: #ty))
        .collect();

    // let generics = add_type_bound(ast.generics.clone(), parse_quote!(#move_type_trait));
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics #thecrate::MoveType for #ident #type_generics #where_clause {
            type TypeTag = #type_tag_type;
        }

        impl #impl_generics #thecrate::MoveStruct for #ident #type_generics #where_clause {
            type StructTag = #type_tag_type;
        }

        impl #impl_generics #ident #type_generics #where_clause {
            pub fn move_instance(self, #(#type_tag_fn_args),*) -> #thecrate::MoveInstance<Self> {
                #thecrate::MoveInstance {
                    type_: Self::type_(#(#attr_idents),*),
                    value: self,
                }
            }

            pub fn type_(#(#type_tag_fn_args),*) -> #type_tag_type {
                #type_tag_ident {
                    #(#attr_idents),*
                }
            }
        }
    }
}

#[derive(Clone)]
struct TypeTagParameters {
    ident: Ident,
    attr_idents: Vec<TokenStream>,
    attr_types: Vec<TokenStream>,
    generics: Generics,
    struct_tag_var_attrs: Vec<TokenStream>,
    struct_tag_const_attrs: Vec<TokenStream>,
    struct_tag_const_vals: Vec<TokenStream>,
    struct_tag_consts_checks: TokenStream,
    type_param_idents: Vec<Ident>,
}

fn type_tag_parameters(
    ast: &DeriveInput,
    thecrate: Path,
    address: Option<String>,
    module: Option<String>,
    nameless: bool,
) -> TypeTagParameters {
    let result_type = quote!(::std::result::Result);
    let mut params = TypeTagParameters {
        ident: type_tag_ident(ast),
        attr_idents: vec![],
        attr_types: vec![],
        generics: ast.generics.clone(),
        struct_tag_var_attrs: vec![],
        struct_tag_const_attrs: vec![],
        struct_tag_const_vals: vec![],
        struct_tag_consts_checks: quote!(),
        type_param_idents: vec![],
    };

    let address_check = if let Some(address) = address {
        params.struct_tag_const_attrs.push(quote!(address));
        let value = quote!(#address.parse().unwrap());
        let check = quote!(
            let expected = #value;
            if address != expected {
                return #result_type::Err(Address { expected, got: address });
            }
        );
        params.struct_tag_const_vals.push(value);
        check
    } else {
        params.attr_idents.push(quote!(address));
        params.attr_types.push(quote!(#thecrate::external::Address));
        params.struct_tag_var_attrs.push(quote!(address));
        quote!()
    };

    let module_check = if let Some(module) = module {
        params.struct_tag_const_attrs.push(quote!(module));
        let value = quote!(#module.parse().unwrap());
        let check = quote!(
            let expected = #value;
            if module != expected {
                return #result_type::Err(Module { expected, got: module });
            }
        );
        params.struct_tag_const_vals.push(value);
        check
    } else {
        params.attr_idents.push(quote!(module));
        params
            .attr_types
            .push(quote!(#thecrate::external::Identifier));
        params.struct_tag_var_attrs.push(quote!(module));
        quote!()
    };

    let name_check = if nameless {
        params.attr_idents.push(quote!(name));
        params
            .attr_types
            .push(quote!(#thecrate::external::Identifier));
        params.struct_tag_var_attrs.push(quote!(name));
        quote!()
    } else {
        let name = ast.ident.to_string();
        params.struct_tag_const_attrs.push(quote!(name));
        let value = quote!(#name.parse().unwrap());
        let check = quote!(
            let expected = #value;
            if name != expected {
                return #result_type::Err(Name { expected, got: name });
            }
        );
        params.struct_tag_const_vals.push(value);
        check
    };

    let n_types_expected = if has_type_params(&ast.generics) {
        let move_type_trait = quote!(#thecrate::MoveType);
        let TypeNames {
            snake: type_names_snake,
            pascal,
        } = extract_type_names(&ast.generics);
        params
            .attr_idents
            .extend(type_names_snake.iter().map(|n| quote!(#n)));
        params.attr_types.extend(
            pascal
                .iter()
                .map(|n| quote!(<#n as #move_type_trait>::TypeTag)),
        );
        params
            .struct_tag_var_attrs
            .push(quote!(type_params: vec![#(#type_names_snake.into()),*]));
        params.type_param_idents = type_names_snake;
        let n_types = pascal.len();
        quote!(#n_types)
    } else {
        params.struct_tag_const_attrs.push(quote!(type_params));
        params.struct_tag_const_vals.push(quote!(vec![]));
        quote!(0_usize)
    };

    params.struct_tag_consts_checks = quote!(
        #address_check
        #module_check
        #name_check
        let expected = #n_types_expected;
        let n_types = type_params.len();
        if n_types != expected {
            return #result_type::Err(TypeParams(#thecrate::TypeParamsError::Number {
                expected, got: n_types
            }));
        }
    );

    params
}

fn type_tag_ident(ast: &DeriveInput) -> Ident {
    let ident = &ast.ident;
    Ident::new(&format!("{ident}TypeTag"), ident.span())
}

// https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs#L36-L44
fn add_type_bound(mut generics: Generics, bound: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(bound.clone());
        }
    }
    generics
}

#[derive(Default)]
struct TypeNames {
    pascal: Vec<Ident>,
    snake: Vec<Ident>,
}

fn extract_type_names(generics: &Generics) -> TypeNames {
    let mut type_names: TypeNames = Default::default();
    for param in &generics.params {
        if let GenericParam::Type(ref type_param) = *param {
            let ident = type_param.ident.clone();
            type_names.snake.push(Ident::new(
                &type_param.ident.to_string().as_str().to_case(Case::Snake),
                ident.span(),
            ));
            type_names.pascal.push(ident); // Assume type names are already Pascal
        }
    }
    type_names
}

fn has_type_params(generics: &Generics) -> bool {
    generics
        .params
        .iter()
        .any(|g| matches!(g, GenericParam::Type(_)))
}

fn impl_static_address(ast: &DeriveInput, thecrate: Path, address: String) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    quote! {
        impl #impl_generics #thecrate::StaticAddress for #name #type_generics #where_clause {
            fn address() -> #thecrate::external::Address {
                #address.parse().unwrap()
            }
        }
    }
}

fn impl_static_module(ast: &DeriveInput, thecrate: Path, module: String) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    quote! {
        impl #impl_generics #thecrate::StaticModule for #name #type_generics #where_clause {
            fn module() -> #thecrate::external::Identifier {
                #module.parse().unwrap()
            }
        }
    }
}

fn impl_static_name(ast: &DeriveInput, thecrate: Path) -> TokenStream {
    let name = &ast.ident;
    let name_str = name.to_string();
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    quote! {
        impl #impl_generics #thecrate::StaticName for #name #type_generics #where_clause {
            fn name() -> #thecrate::external::Identifier {
                #name_str.parse().unwrap()
            }
        }
    }
}

fn impl_static_type_params(ast: &DeriveInput, thecrate: Path) -> TokenStream {
    let name = &ast.ident;
    let trait_type = quote! { #thecrate::StaticTypeParams };
    if has_type_params(&ast.generics) {
        let static_type_tag = quote!(#thecrate::StaticTypeTag);
        let generics = add_type_bound(ast.generics.clone(), parse_quote!(#static_type_tag));
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let TypeNames {
            pascal: type_names, ..
        } = extract_type_names(&generics);
        quote! {
            impl #impl_generics #trait_type for #name #type_generics #where_clause {
                fn type_params() -> Vec<#thecrate::external::TypeTag> {
                    vec![#(<#type_names as #static_type_tag>::type_tag()),*]
                }
            }
        }
    } else {
        let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
        quote! {
            impl #impl_generics #trait_type for #name #type_generics #where_clause {
                fn type_params() -> Vec<#thecrate::external::TypeTag> {
                    vec![]
                }
            }
        }
    }
}
