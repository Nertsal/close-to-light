use std::collections::VecDeque;

use darling::export::syn;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub struct AtlasOpts {
    vis: syn::Visibility,
    struct_name: syn::Ident,
    fields: Punctuated<AtlasFieldOpts, syn::Token![,]>,
}

enum AtlasFieldOpts {
    Texture(syn::Ident),
    Folder(syn::Ident, Punctuated<AtlasFieldOpts, syn::Token![,]>),
}

impl syn::parse::Parse for AtlasOpts {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<syn::Visibility>()?;

        let struct_name = input.parse::<syn::Ident>()?;

        let fields;
        syn::braced!(fields in input);
        let fields = fields.parse_terminated(AtlasFieldOpts::parse, syn::Token![,])?;

        Ok(Self {
            vis,
            struct_name,
            fields,
        })
    }
}

impl syn::parse::Parse for AtlasFieldOpts {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        match input.parse::<Option<syn::Token![:]>>()? {
            Some(_colon) => {
                let fields;
                syn::braced!(fields in input);
                let fields = fields.parse_terminated(AtlasFieldOpts::parse, syn::Token![,])?;
                Ok(Self::Folder(ident, fields))
            }
            None => Ok(Self::Texture(ident)),
        }
    }
}

impl AtlasOpts {
    pub fn generate(self) -> TokenStream {
        let Self {
            vis,
            struct_name,
            fields,
        } = self;

        let mut generated = TokenStream::new();
        let atlas = quote! { ctl_render_core::TextureAtlas };
        let subtexture = quote! { ctl_render_core::SubTexture };

        {
            fn process_field(field: &AtlasFieldOpts) -> Vec<VecDeque<&syn::Ident>> {
                match field {
                    AtlasFieldOpts::Texture(ident) => {
                        vec![vec![ident].into()]
                    }
                    AtlasFieldOpts::Folder(ident, fields) => fields
                        .iter()
                        .flat_map(process_field)
                        .map(|mut inner| {
                            inner.push_front(ident);
                            inner
                        })
                        .collect(),
                }
            }

            let all_textures: Vec<_> = fields.iter().flat_map(process_field).collect();
            let field_getters = all_textures.iter().enumerate().map(|(i, ident_path)| {
                let mut ident = String::new();
                let mut ident_path = ident_path.iter();
                if let Some(id) = ident_path.next() {
                    ident += &id.to_string();
                }
                for id in ident_path {
                    ident.push('_');
                    ident += &id.to_string();
                }
                let ident = syn::Ident::new(&ident, proc_macro2::Span::mixed_site());
                quote! {
                    #vis fn #ident(&self) -> #subtexture {
                        self.0.get(#i)
                    }
                }
            });

            let load_textures = all_textures.iter().map(|ident_path| {
                let path_mut = ident_path.iter().map(|ident| {
                    quote! { let path = path.join(stringify!(#ident)); }
                });
                quote! {{
                    #(#path_mut)*;
                    let path = path.with_extension("png");
                    let options = geng::asset::TextureOptions{
                        filter: ugli::Filter::Nearest,
                        ..default()
                    };
                    <ugli::Texture as geng::asset::Load>::load(&manager, &path, &options)
                }}
            });
            let load_textures = quote! { [#(#load_textures),*] };

            generated.extend(quote! {
                #vis struct #struct_name(#atlas);

                impl #struct_name {
                    #vis fn atlas(&self) -> &#atlas {
                        &self.0
                    }

                    #vis fn texture(&self) -> &ugli::Texture {
                        self.0.texture()
                    }

                    #(#field_getters)*
                }

                impl geng::asset::Load for #struct_name {
                    type Options = ();

                    fn load(
                        manager: &geng::asset::Manager,
                        path: &std::path::Path,
                        options: &Self::Options,
                    ) -> geng::asset::Future<Self> {
                        let path = path.to_owned();
                        let manager = manager.clone();
                        async move {
                            let textures = #load_textures;
                            let textures_loaded = future::join_all(textures).await;
                            let mut textures = Vec::new();
                            for texture in textures_loaded {
                                let texture = texture?;
                                textures.push(texture);
                            }
                            let textures: Vec<_> = textures.iter().collect();
                            let atlas = #atlas::new(manager.ugli(), &textures, ugli::Filter::Nearest);
                            Ok(Self(atlas))
                        }.boxed_local()
                    }

                    const DEFAULT_EXT: Option<&'static str> = None;
                }
            });
        }

        generated
    }
}
