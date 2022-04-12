use proc_macro2::{Span, TokenStream};

use quote::{quote, ToTokens};

use std::collections::HashMap;

use syn::*;
use syn::parse::Parser;
use syn::spanned::Spanned;

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
/// The type of callback to register with the [`Moisture`](Moisture) structure.
pub enum CallbackType {
    File,
    
    Item,
    ItemConst,
    ItemEnum,
    ItemExternCrate,
    ItemFn,
    ItemForeignMod,
    ItemImpl,
    ItemMacro,
    ItemMacro2,
    ItemMod,
    ItemStatic,
    ItemStruct,
    ItemTrait,
    ItemTraitAlias,
    ItemType,
    ItemUnion,
    ItemUse,
    
    Variant,

    ForeignItem,
    ForeignItemFn,
    ForeignItemStatic,
    ForeignItemType,
    ForeignItemMacro,

    ImplItem,
    ImplItemConst,
    ImplItemMethod,
    ImplItemType,
    ImplItemMacro,

    TraitItem,
    TraitItemConst,
    TraitItemMethod,
    TraitItemType,
    TraitItemMacro,

    Block,

    Stmts,

    Stmt,

    Local,

    Pat,
    PatBox,
    PatIdent,
    PatLit,
    PatMacro,
    PatOr,
    PatPath,
    PatRange,
    PatReference,
    PatRest,
    PatSlice,
    PatStruct,
    PatTuple,
    PatTupleStruct,
    PatType,
    PatWild,

    Expr,
    ExprArray,
    ExprAssign,
    ExprAssignOp,
    ExprAsync,
    ExprAwait,
    ExprBinary,
    ExprBlock,
    ExprBox,
    ExprBreak,
    ExprCall,
    ExprCast,
    ExprClosure,
    ExprContinue,
    ExprField,
    ExprForLoop,
    ExprGroup,
    ExprIf,
    ExprIndex,
    ExprLet,
    ExprLit,
    ExprLoop,
    ExprMacro,
    ExprMatch,
    ExprMethodCall,
    ExprParen,
    ExprPath,
    ExprRange,
    ExprReference,
    ExprRepeat,
    ExprReturn,
    ExprStruct,
    ExprTry,
    ExprTryBlock,
    ExprTuple,
    ExprType,
    ExprUnary,
    ExprUnsafe,
    ExprWhile,
    ExprYield,

    Arm,

    Lit,
    LitStr,
    LitByteStr,
    LitByte,
    LitChar,
    LitInt,
    LitFloat,
    LitBool,

    FieldValue,

    Verbatim,
}

#[derive(Clone, Debug)]
pub struct Context {
    stack: Vec<(CallbackType, TokenStream)>,
}
impl Context {
    pub fn new() -> Self {
        Self { stack: Vec::<(CallbackType, TokenStream)>::new() }
    }
    pub fn push(&mut self, ty: CallbackType, tokens: TokenStream) {
        self.stack.push((ty, tokens));
    }
    pub fn pop(&mut self) -> Option<(CallbackType, TokenStream)> {
        self.stack.pop()
    }
    pub fn peek(&self, distance: usize) -> Option<(CallbackType, TokenStream)> {
        if self.stack.len() == 0 || distance >= self.stack.len() { None }
        else { let entry = &self.stack[self.stack.len()-distance-1]; Some(entry.clone()) }
    }
}

/// The callback function to register with the [`Moisture`](Moisture) structure.
pub type Callback = fn(&Moisture, &Context, TokenStream) -> Result<TokenStream>;

#[macro_export]
macro_rules! run_moisture {
    ($moisture:ident, $callback_ty:path, $tokens:ident) => {
        match $moisture.callback(&Context::new(), $callback_ty, $tokens) {
            Ok(new_tokens) => new_tokens,
            Err(e) => e.to_compile_error(),
        }
    };
}

fn get_pat_type(tokens: TokenStream) -> Result<PatType> {
    let stub = quote! { let #tokens ; };
    let stmt = parse2::<Stmt>(stub)?;
    let stmt_span = stmt.span().clone();
    let local_data;

    if let Stmt::Local(local) = stmt {
        local_data = local;
    }
    else { return Err(Error::new(stmt_span, "expected Local statement in PatType interpretation")); }

    if let Pat::Type(pat) = local_data.pat { Ok(pat) }
    else { Err(Error::new(stmt_span, "expected PatType object in Local statement pattern")) }
}

#[derive(Clone)]
pub struct Moisture {
    callbacks: HashMap<CallbackType, Callback>,
}
impl Moisture {
    pub fn new() -> Self {
        let mut result = Self { callbacks: HashMap::<CallbackType, Callback>::new() };
        result.load_defaults();
        result
    }
    fn load_defaults(&mut self) {
        self.register_callback(CallbackType::File, Moisture::file);
        
        self.register_callback(CallbackType::Item, Moisture::item);
        self.register_callback(CallbackType::ItemConst, Moisture::item_const);
        self.register_callback(CallbackType::ItemEnum, Moisture::item_enum);
        self.register_callback(CallbackType::ItemExternCrate, Moisture::item_extern_crate);
        self.register_callback(CallbackType::ItemFn, Moisture::item_fn);
        self.register_callback(CallbackType::ItemForeignMod, Moisture::item_foreign_mod);
        self.register_callback(CallbackType::ItemImpl, Moisture::item_impl);
        self.register_callback(CallbackType::ItemMacro, Moisture::item_macro);
        self.register_callback(CallbackType::ItemMacro2, Moisture::item_macro2);
        self.register_callback(CallbackType::ItemMod, Moisture::item_mod);
        self.register_callback(CallbackType::ItemStatic, Moisture::item_static);
        self.register_callback(CallbackType::ItemStruct, Moisture::item_struct);
        self.register_callback(CallbackType::ItemTrait, Moisture::item_trait);
        self.register_callback(CallbackType::ItemTraitAlias, Moisture::item_trait_alias);
        self.register_callback(CallbackType::ItemType, Moisture::item_type);
        self.register_callback(CallbackType::ItemUnion, Moisture::item_union);
        self.register_callback(CallbackType::ItemUse, Moisture::item_use);
        
        self.register_callback(CallbackType::Variant, Moisture::variant);

        self.register_callback(CallbackType::ForeignItem, Moisture::foreign_item);
        self.register_callback(CallbackType::ForeignItemFn, Moisture::foreign_item_fn);
        self.register_callback(CallbackType::ForeignItemStatic, Moisture::foreign_item_static);
        self.register_callback(CallbackType::ForeignItemType, Moisture::foreign_item_type);
        self.register_callback(CallbackType::ForeignItemMacro, Moisture::foreign_item_macro);

        self.register_callback(CallbackType::ImplItem, Moisture::impl_item);
        self.register_callback(CallbackType::ImplItemConst, Moisture::impl_item_const);
        self.register_callback(CallbackType::ImplItemMethod, Moisture::impl_item_method);
        self.register_callback(CallbackType::ImplItemType, Moisture::impl_item_type);
        self.register_callback(CallbackType::ImplItemMacro, Moisture::impl_item_macro);

        self.register_callback(CallbackType::TraitItem, Moisture::trait_item);
        self.register_callback(CallbackType::TraitItemConst, Moisture::trait_item_const);
        self.register_callback(CallbackType::TraitItemMethod, Moisture::trait_item_method);
        self.register_callback(CallbackType::TraitItemType, Moisture::trait_item_type);
        self.register_callback(CallbackType::TraitItemMacro, Moisture::trait_item_macro);

        self.register_callback(CallbackType::Block, Moisture::block);

        self.register_callback(CallbackType::Stmts, Moisture::stmts);

        self.register_callback(CallbackType::Stmt, Moisture::stmt);

        self.register_callback(CallbackType::Local, Moisture::local);

        self.register_callback(CallbackType::Pat, Moisture::pat);
        self.register_callback(CallbackType::PatBox, Moisture::pat_box);
        self.register_callback(CallbackType::PatIdent, Moisture::pat_ident);
        self.register_callback(CallbackType::PatLit, Moisture::pat_lit);
        self.register_callback(CallbackType::PatMacro, Moisture::pat_macro);
        self.register_callback(CallbackType::PatOr, Moisture::pat_or);
        self.register_callback(CallbackType::PatPath, Moisture::pat_path);
        self.register_callback(CallbackType::PatRange, Moisture::pat_range);
        self.register_callback(CallbackType::PatReference, Moisture::pat_reference);
        self.register_callback(CallbackType::PatRest, Moisture::pat_rest);
        self.register_callback(CallbackType::PatSlice, Moisture::pat_slice);
        self.register_callback(CallbackType::PatStruct, Moisture::pat_struct);
        self.register_callback(CallbackType::PatTuple, Moisture::pat_tuple);
        self.register_callback(CallbackType::PatTupleStruct, Moisture::pat_tuple_struct);
        self.register_callback(CallbackType::PatType, Moisture::pat_type);
        self.register_callback(CallbackType::PatWild, Moisture::pat_wild);

        self.register_callback(CallbackType::Expr, Moisture::expr);
        self.register_callback(CallbackType::ExprArray, Moisture::expr_array);
        self.register_callback(CallbackType::ExprAssign, Moisture::expr_assign);
        self.register_callback(CallbackType::ExprAssignOp, Moisture::expr_assign_op);
        self.register_callback(CallbackType::ExprAsync, Moisture::expr_async);
        self.register_callback(CallbackType::ExprAwait, Moisture::expr_await);
        self.register_callback(CallbackType::ExprBinary, Moisture::expr_binary);
        self.register_callback(CallbackType::ExprBlock, Moisture::expr_block);
        self.register_callback(CallbackType::ExprBox, Moisture::expr_box);
        self.register_callback(CallbackType::ExprBreak, Moisture::expr_break);
        self.register_callback(CallbackType::ExprCall, Moisture::expr_call);
        self.register_callback(CallbackType::ExprCast, Moisture::expr_cast);
        self.register_callback(CallbackType::ExprClosure, Moisture::expr_closure);
        self.register_callback(CallbackType::ExprContinue, Moisture::expr_continue);
        self.register_callback(CallbackType::ExprField, Moisture::expr_field);
        self.register_callback(CallbackType::ExprForLoop, Moisture::expr_for_loop);
        self.register_callback(CallbackType::ExprGroup, Moisture::expr_group);
        self.register_callback(CallbackType::ExprIf, Moisture::expr_if);
        self.register_callback(CallbackType::ExprIndex, Moisture::expr_index);
        self.register_callback(CallbackType::ExprLet, Moisture::expr_let);
        self.register_callback(CallbackType::ExprLit, Moisture::expr_lit);
        self.register_callback(CallbackType::ExprLoop, Moisture::expr_loop);
        self.register_callback(CallbackType::ExprMacro, Moisture::expr_macro);
        self.register_callback(CallbackType::ExprMatch, Moisture::expr_match);
        self.register_callback(CallbackType::ExprMethodCall, Moisture::expr_method_call);
        self.register_callback(CallbackType::ExprParen, Moisture::expr_paren);
        self.register_callback(CallbackType::ExprPath, Moisture::expr_path);
        self.register_callback(CallbackType::ExprRange, Moisture::expr_range);
        self.register_callback(CallbackType::ExprReference, Moisture::expr_reference);
        self.register_callback(CallbackType::ExprRepeat, Moisture::expr_repeat);
        self.register_callback(CallbackType::ExprReturn, Moisture::expr_return);
        self.register_callback(CallbackType::ExprStruct, Moisture::expr_struct);
        self.register_callback(CallbackType::ExprTry, Moisture::expr_try);
        self.register_callback(CallbackType::ExprTryBlock, Moisture::expr_try_block);
        self.register_callback(CallbackType::ExprTuple, Moisture::expr_tuple);
        self.register_callback(CallbackType::ExprType, Moisture::expr_type);
        self.register_callback(CallbackType::ExprUnary, Moisture::expr_unary);
        self.register_callback(CallbackType::ExprUnsafe, Moisture::expr_unsafe);
        self.register_callback(CallbackType::ExprWhile, Moisture::expr_while);
        self.register_callback(CallbackType::ExprYield, Moisture::expr_yield);

        self.register_callback(CallbackType::Arm, Moisture::arm);

        self.register_callback(CallbackType::Lit, Moisture::lit);
        self.register_callback(CallbackType::LitStr, Moisture::lit_str);
        self.register_callback(CallbackType::LitByteStr, Moisture::lit_byte_str);
        self.register_callback(CallbackType::LitByte, Moisture::lit_byte);
        self.register_callback(CallbackType::LitChar, Moisture::lit_char);
        self.register_callback(CallbackType::LitInt, Moisture::lit_int);
        self.register_callback(CallbackType::LitFloat, Moisture::lit_float);
        self.register_callback(CallbackType::LitBool, Moisture::lit_bool);

        self.register_callback(CallbackType::FieldValue, Moisture::field_value);

        self.register_callback(CallbackType::Verbatim, Moisture::verbatim);
    }
    pub fn register_callback(&mut self, ty: CallbackType, callback: Callback) {
        self.callbacks.insert(ty, callback);
    }
    pub fn callback(&self, context: &Context, ty: CallbackType, tokens: TokenStream) -> Result<TokenStream> {
        if let Some(callback) = self.callbacks.get(&ty) {
            let mut new_context = context.clone();
            new_context.push(ty, tokens.clone());
            
            let result = callback(self, &new_context, tokens)?;

            Ok(result)
        }
        else { Err(Error::new(Span::call_site(), format!("couldn't find function for callback type {:?}", ty))) }
    }
    pub fn file(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<File>(tokens)?;
        let mut tokens = TokenStream::new();
        
        let mut new_attrs = Vec::<TokenStream>::new();
            
        for attr in &parsed.attrs {
            new_attrs.push(attr.to_token_stream());
        }

        tokens.extend(new_attrs.into_iter());

        let mut new_items = Vec::<TokenStream>::new();

        for item in &parsed.items {
            let item_stream = self.callback(context, CallbackType::Item, item.to_token_stream())?;
            new_items.push(item_stream);
        }

        tokens.extend(new_items.into_iter());
        
        Ok(tokens)
    }

    pub fn item(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Item>(tokens)?;
        
        let result = match parsed {
            Item::Const(ref const_) => self.callback(context, CallbackType::ItemConst, const_.to_token_stream()),
            Item::Enum(ref enum_) => self.callback(context, CallbackType::ItemEnum, enum_.to_token_stream()),
            Item::ExternCrate(ref crate_) => self.callback(context, CallbackType::ItemExternCrate, crate_.to_token_stream()),
            Item::Fn(ref fn_) => self.callback(context, CallbackType::ItemFn, fn_.to_token_stream()),
            Item::ForeignMod(ref mod_) => self.callback(context, CallbackType::ItemForeignMod, mod_.to_token_stream()),
            Item::Impl(ref impl_) => self.callback(context, CallbackType::ItemImpl, impl_.to_token_stream()),
            Item::Macro(ref macro_) => self.callback(context, CallbackType::ItemMacro, macro_.to_token_stream()),
            Item::Macro2(ref macro_) => self.callback(context, CallbackType::ItemMacro2, macro_.to_token_stream()),
            Item::Mod(ref mod_) => self.callback(context, CallbackType::ItemMod, mod_.to_token_stream()),
            Item::Static(ref static_) => self.callback(context, CallbackType::ItemStatic, static_.to_token_stream()),
            Item::Struct(ref struct_) => self.callback(context, CallbackType::ItemStruct, struct_.to_token_stream()),
            Item::Trait(ref trait_) => self.callback(context, CallbackType::ItemTrait, trait_.to_token_stream()),
            Item::TraitAlias(ref alias) => self.callback(context, CallbackType::ItemTraitAlias, alias.to_token_stream()),
            Item::Type(ref type_) => self.callback(context, CallbackType::ItemType, type_.to_token_stream()),
            Item::Union(ref union) => self.callback(context, CallbackType::ItemUnion, union.to_token_stream()),
            Item::Use(ref use_) => self.callback(context, CallbackType::ItemUse, use_.to_token_stream()),
            
            Item::Verbatim(ref vert_tokens) => self.callback(context, CallbackType::Verbatim, vert_tokens.clone()),
            _ => Ok(parsed.to_token_stream()),
        }?;

        Ok(result)
    }
    pub fn item_const(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemConst {
            attrs,
            vis,
            const_token,
            ident,
            colon_token,
            ty,
            eq_token,
            expr,
            semi_token } = parse2::<ItemConst>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #vis #const_token #ident #colon_token #ty #eq_token
        });
        
        let filtered_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        tokens.push(filtered_expr);
        
        tokens.push(quote! { #semi_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_enum(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemEnum {
            attrs,
            vis,
            enum_token,
            ident,
            generics,
            brace_token: _,
            variants } = parse2::<ItemEnum>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #vis #enum_token #ident #generics
        });

        let mut filtered_variants = Vec::<TokenStream>::new();

        for variant in &variants {
            let new_variant = self.callback(context, CallbackType::Variant, variant.to_token_stream())?;
            filtered_variants.push(new_variant);
        }

        tokens.push(quote! {
            {
                #(#filtered_variants),*
            }
        });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn variant(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let Variant {
            attrs,
            ident,
            fields,
            discriminant } = parse2::<Variant>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #ident #fields
        });

        if let Some((eq_token, expr)) = discriminant {
            let filtered_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

            tokens.push(quote! { #eq_token #filtered_expr });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_extern_crate(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        // nothing to expand for now, essentially a no-op to parse the type and turn
        // it back into tokens
        let parsed = parse2::<ItemExternCrate>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_fn(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemFn {
            attrs,
            vis,
            sig,
            block } = parse2::<ItemFn>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #vis #sig
        });
        
        let filtered_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;
        tokens.push(filtered_block);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_foreign_mod(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemForeignMod {
            attrs,
            abi,
            brace_token: _,
            items } = parse2::<ItemForeignMod>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #abi
        });

        let mut filtered_items = Vec::<TokenStream>::new();

        for item in &items {
            let filtered = self.callback(context, CallbackType::ForeignItem, item.to_token_stream())?;
            filtered_items.push(filtered);
        }

        tokens.push(quote! { #(#filtered_items)* });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_impl(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            brace_token: _,
            items } = parse2::<ItemImpl>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(default_token) = defaultness {
            tokens.push(quote! { #default_token });
        }

        if let Some(unsafe_token) = unsafety {
            tokens.push(quote! { #unsafe_token });
        }

        tokens.push(quote! {
            #impl_token #generics
        });

        if let Some((bang_opt, path, for_)) = trait_ {
            if let Some(bang_token) = bang_opt {
                tokens.push(quote! { #bang_token });
            }

            tokens.push(quote! { #path #for_ });
        }

        tokens.push(quote! { #self_ty });

        let mut filtered_items = Vec::<TokenStream>::new();

        for item in &items {
            let filtered_item = self.callback(context, CallbackType::ImplItem, item.to_token_stream())?;
            filtered_items.push(filtered_item);
        }

        tokens.push(quote! {
            {
                #(#filtered_items)*
            }
        });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemMacro>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_macro2(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemMacro2>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_mod(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemMod {
            attrs,
            vis,
            mod_token,
            ident,
            content,
            semi } = parse2::<ItemMod>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #vis #mod_token #ident
        });

        if let Some((_, items)) = content {
            let mut filtered_items = Vec::<TokenStream>::new();

            for item in &items {
                let new_item = self.callback(context, CallbackType::Item, item.to_token_stream())?;
                filtered_items.push(new_item);
            }

            tokens.push(quote! { #(#filtered_items)* });
        }

        if let Some(semi_token) = semi {
            tokens.push(quote! { #semi_token });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_static(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemStatic {
            attrs,
            vis,
            static_token,
            mutability,
            ident,
            colon_token,
            ty,
            eq_token,
            expr,
            semi_token } = parse2::<ItemStatic>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #vis #static_token
        });

        if let Some(mut_token) = mutability {
            tokens.push(quote! { #mut_token });
        }

        tokens.push(quote! { #ident #colon_token #ty #eq_token });

        let filtered_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        tokens.push(quote! { #filtered_expr #semi_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_struct(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemStruct>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_trait(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ItemTrait {
            attrs,
            vis,
            unsafety,
            auto_token,
            trait_token,
            ident,
            generics,
            colon_token,
            supertraits,
            brace_token: _,
            items } = parse2::<ItemTrait>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #vis });

        if let Some(unsafe_keyword) = unsafety {
            tokens.push(quote! { #unsafe_keyword });
        }

        if let Some(auto) = auto_token {
            tokens.push(quote! { #auto });
        }

        tokens.push(quote! { #trait_token #ident #generics });

        if let Some(colon) = colon_token {
            tokens.push(quote! { #colon });
        }

        let mut traits = Vec::<TokenStream>::new();

        for trait_ in &supertraits {
            traits.push(trait_.to_token_stream());
        }

        tokens.push(quote! { #(#traits)+* });

        let mut trait_items = Vec::<TokenStream>::new();

        for item in &items {
            let new_item = self.callback(context, CallbackType::TraitItem, item.to_token_stream())?;
            trait_items.push(new_item);
        }

        tokens.push(quote! {
            {
                #(#trait_items)*
            }
        });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn item_trait_alias(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemTraitAlias>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_type(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemType>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_union(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemUnion>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn item_use(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ItemUse>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn foreign_item(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ForeignItem>(tokens)?;

        let result = match parsed {
            ForeignItem::Fn(ref fn_) => self.callback(context, CallbackType::ForeignItemFn, fn_.to_token_stream()),
            ForeignItem::Static(ref static_) => self.callback(context, CallbackType::ForeignItemStatic, static_.to_token_stream()),
            ForeignItem::Type(ref type_) => self.callback(context, CallbackType::ForeignItemType, type_.to_token_stream()),
            ForeignItem::Macro(ref macro_) => self.callback(context, CallbackType::ForeignItemMacro, macro_.to_token_stream()),
            ForeignItem::Verbatim(ref verbatim) => self.callback(context, CallbackType::Verbatim, verbatim.clone()),
            _ => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn foreign_item_fn(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ForeignItemFn>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn foreign_item_static(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ForeignItemStatic>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn foreign_item_type(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ForeignItemType>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn foreign_item_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ForeignItemMacro>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn impl_item(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ImplItem>(tokens)?;

        let result = match parsed {
            ImplItem::Const(ref const_) => self.callback(context, CallbackType::ImplItemConst, const_.to_token_stream()),
            ImplItem::Method(ref method) => self.callback(context, CallbackType::ImplItemMethod, method.to_token_stream()),
            ImplItem::Type(ref type_) => self.callback(context, CallbackType::ImplItemType, type_.to_token_stream()),
            ImplItem::Macro(ref macro_) => self.callback(context, CallbackType::ImplItemMacro, macro_.to_token_stream()),
            ImplItem::Verbatim(ref verbatim) => self.callback(context, CallbackType::Verbatim, verbatim.clone()),
            _ => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn impl_item_const(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ImplItemConst {
            attrs,
            vis,
            defaultness,
            const_token,
            ident,
            colon_token,
            ty,
            eq_token,
            expr,
            semi_token } = parse2::<ImplItemConst>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #vis });

        if let Some(default_token) = defaultness {
            tokens.push(quote! { #default_token });
        }

        tokens.push(quote! { #const_token #ident #colon_token #ty #eq_token });

        let filtered_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        tokens.push(quote! { #filtered_expr #semi_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn impl_item_method(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ImplItemMethod {
            attrs,
            vis,
            defaultness,
            sig,
            block } = parse2::<ImplItemMethod>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #vis });

        if let Some(default_token) = defaultness {
            tokens.push(quote! { #default_token });
        }

        tokens.push(quote! { #sig });

        let filtered_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;
        tokens.push(quote! { #filtered_block });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn impl_item_type(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ImplItemType>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn impl_item_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ImplItemMacro>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn trait_item(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<TraitItem>(tokens)?;

        let result = match parsed {
            TraitItem::Const(ref const_) => self.callback(context, CallbackType::TraitItemConst, const_.to_token_stream()),
            TraitItem::Method(ref method) => self.callback(context, CallbackType::TraitItemMethod, method.to_token_stream()),
            TraitItem::Type(ref type_) => self.callback(context, CallbackType::TraitItemType, type_.to_token_stream()),
            TraitItem::Macro(ref macro_) => self.callback(context, CallbackType::TraitItemMacro, macro_.to_token_stream()),
            TraitItem::Verbatim(ref verbatim) => self.callback(context, CallbackType::Verbatim, verbatim.clone()),
            _ => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn trait_item_const(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let TraitItemConst {
            attrs,
            const_token,
            ident,
            colon_token,
            ty,
            default,
            semi_token } = parse2::<TraitItemConst>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)* #const_token #ident #colon_token #ty
        });

        if let Some((eq_token, expr)) = default {
            let filtered_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
            tokens.push(quote! { #eq_token #filtered_expr });
        }

        tokens.push(quote! { #semi_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn trait_item_method(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let TraitItemMethod {
            attrs,
            sig,
            default,
            semi_token } = parse2::<TraitItemMethod>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #sig });

        if let Some(block) = default {
            let new_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;
            tokens.push(quote! { #new_block });
        }

        if let Some(semi) = semi_token {
            tokens.push(quote! { #semi });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn trait_item_type(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<TraitItemType>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn trait_item_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<TraitItemMacro>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn block(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Block>(tokens)?;
        let stmts = parsed.stmts.clone();
        let stmt_tokens = self.callback(context, CallbackType::Stmts, quote! { #(#stmts)* })?;

        Ok(quote! {
            {
                #stmt_tokens
            }
        })
    }
    pub fn stmts(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let statements = Block::parse_within.parse2(tokens)?;
        let mut new_statements = Vec::<TokenStream>::new();

        for stmt in &statements {
            let new_stmt = self.callback(context, CallbackType::Stmt, stmt.to_token_stream())?;
            new_statements.push(new_stmt);
        }

        Ok(quote! { #(#new_statements)* })
    }
    pub fn stmt(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Stmt>(tokens)?;

        let result = match parsed {
            Stmt::Local(ref local) => self.callback(context, CallbackType::Local, local.to_token_stream()),
            Stmt::Item(ref item) => self.callback(context, CallbackType::Item, item.to_token_stream()),
            Stmt::Expr(ref expr) => self.callback(context, CallbackType::Expr, expr.to_token_stream()),
            Stmt::Semi(ref expr, _) => self.callback(context, CallbackType::Expr, expr.to_token_stream()),
        }?;

        if let Stmt::Semi(_, _) = parsed { Ok(quote! { #result ; }) }
        else { Ok(result) }
    }
    pub fn local(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        // Local doesn't implement Parse, so we need to go one level higher to get a Local struct
        let stmt = parse2::<Stmt>(tokens)?;
        let local;

        if let Stmt::Local(local_obj) = stmt {
            local = local_obj;
        }
        else { return Err(Error::new(stmt.span(), "expected Local declaration in statement")); }

        let Local {
            attrs,
            let_token,
            pat,
            init,
            semi_token } = local;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! {
            #(#attrs)*
            #let_token
        });

        let new_pat;

        // PatType is a special case that only occurs within local statements, the
        // Pat parser doesn't interpret this case as a result.
        if let Pat::Type(pat_type) = pat {
            new_pat = self.callback(context, CallbackType::PatType, pat_type.to_token_stream())?;
        }
        else {
            new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
        }
        
        tokens.push(quote! { #new_pat });

        if let Some((eq_token, expr)) = init {
            let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
            tokens.push(quote! { #eq_token #new_expr });
        }

        tokens.push(quote! { #semi_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn pat(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        let result = match parsed {
            Pat::Box(ref box_) => self.callback(context, CallbackType::PatBox, box_.to_token_stream()),
            Pat::Ident(ref ident) => self.callback(context, CallbackType::PatIdent, ident.to_token_stream()),
            Pat::Lit(ref lit) => self.callback(context, CallbackType::PatLit, lit.to_token_stream()),
            Pat::Macro(ref macro_) => self.callback(context, CallbackType::PatMacro, macro_.to_token_stream()),
            Pat::Or(ref or) => self.callback(context, CallbackType::PatOr, or.to_token_stream()),
            Pat::Path(ref path) => self.callback(context, CallbackType::PatPath, path.to_token_stream()),
            Pat::Range(ref range) => self.callback(context, CallbackType::PatRange, range.to_token_stream()),
            Pat::Reference(ref ref_) => self.callback(context, CallbackType::PatReference, ref_.to_token_stream()),
            Pat::Rest(ref rest) => self.callback(context, CallbackType::PatRest, rest.to_token_stream()),
            Pat::Slice(ref slice) => self.callback(context, CallbackType::PatSlice, slice.to_token_stream()),
            Pat::Struct(ref struct_) => self.callback(context, CallbackType::PatStruct, struct_.to_token_stream()),
            Pat::Tuple(ref tuple) => self.callback(context, CallbackType::PatTuple, tuple.to_token_stream()),
            Pat::TupleStruct(ref struct_) => self.callback(context, CallbackType::PatTupleStruct, struct_.to_token_stream()),
            Pat::Verbatim(ref verbatim) => self.callback(context, CallbackType::Verbatim, verbatim.clone()),
            Pat::Wild(ref wild) => self.callback(context, CallbackType::PatWild, wild.to_token_stream()),
            _ => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn pat_box(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        if let Pat::Box(obj) = parsed {
            Ok(obj.to_token_stream())
        }
        else { Err(Error::new(parsed.span(), "expected PatBox object in pattern")) }
    }
    pub fn pat_ident(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_ident;

        if let Pat::Ident(ident) = parsed {
            pat_ident = ident;
        }
        else { return Err(Error::new(parsed.span(), "expected PatIdent object in pattern")); }
        
        let PatIdent {
            attrs,
            by_ref,
            mutability,
            ident,
            subpat } = pat_ident;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(ref_token) = by_ref {
            tokens.push(quote! { #ref_token });
        }

        if let Some(mut_token) = mutability {
            tokens.push(quote! { #mut_token });
        }

        tokens.push(quote! { #ident });

        if let Some((at_token, pat)) = subpat {
            let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
            tokens.push(quote! { #at_token, #new_pat });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn pat_lit(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_lit;

        if let Pat::Lit(lit) = parsed {
            pat_lit = lit;
        }
        else { return Err(Error::new(parsed.span(), "expected PatLit object in pattern")); }
               
        let PatLit { attrs, expr } = pat_lit;
        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr })
    }
    pub fn pat_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        if let Pat::Macro(macro_) = parsed {
            Ok(macro_.to_token_stream())
        }
        else {
            Err(Error::new(parsed.span(), "expected PatMacro object in pattern"))
        }
    }
    pub fn pat_or(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_or;

        if let Pat::Or(or) = parsed {
            pat_or = or;
        }
        else { return Err(Error::new(parsed.span(), "expected PatOr object in pattern")); }
        
        let PatOr {
            attrs,
            leading_vert,
            cases } = pat_or;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(lead) = leading_vert {
            tokens.push(quote! { #lead });
        }

        let mut new_patterns = Vec::<TokenStream>::new();

        for case in &cases {
            let new_pat = self.callback(context, CallbackType::Pat, case.to_token_stream())?;
            new_patterns.push(new_pat);
        }

        tokens.push(quote! { #(#new_patterns)|* });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn pat_path(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        if let Pat::Path(path) = parsed {
            Ok(path.to_token_stream())
        }
        else {
            Err(Error::new(parsed.span(), "expected PatPath object in pattern"))
        }
    }
    pub fn pat_range(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_range;

        if let Pat::Range(range) = parsed {
            pat_range = range;
        }
        else { return Err(Error::new(parsed.span(), "expected PatRange object in pattern")); }
        
        let PatRange {
            attrs,
            lo,
            limits,
            hi } = pat_range;

        let new_lo = self.callback(context, CallbackType::Expr, lo.to_token_stream())?;
        let new_hi = self.callback(context, CallbackType::Expr, hi.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_lo #limits #new_hi })
    }
    pub fn pat_reference(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_ref;

        if let Pat::Reference(ref_) = parsed {
            pat_ref = ref_;
        }
        else { return Err(Error::new(parsed.span(), "expected PatReference object in pattern")); }
        
        let PatReference {
            attrs,
            and_token,
            mutability,
            pat } = pat_ref;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #and_token });

        if let Some(mut_token) = mutability {
            tokens.push(quote! { #mut_token });
        }

        let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
        tokens.push(quote! { #new_pat });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn pat_rest(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        if let Pat::Rest(rest) = parsed {
            Ok(rest.to_token_stream())
        }
        else { Err(Error::new(parsed.span(), "expected PatRest object in pattern")) }
    }
    pub fn pat_slice(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_slice;

        if let Pat::Slice(slice) = parsed {
            pat_slice = slice;
        }
        else { return Err(Error::new(parsed.span(), "expected PatSlice object in pattern")); }
        
        let PatSlice {
            attrs,
            bracket_token: _,
            elems } = pat_slice;

        let mut new_elems = Vec::<TokenStream>::new();

        for elem in &elems {
            let new_elem = self.callback(context, CallbackType::Pat, elem.to_token_stream())?;
            new_elems.push(new_elem);
        }

        Ok(quote! { #(#attrs)* [ #(#new_elems),* ] })
    }
    pub fn pat_struct(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_struct;

        if let Pat::Struct(struct_) = parsed {
            pat_struct = struct_;
        }
        else { return Err(Error::new(parsed.span(), "expected PatStruct object in pattern")); }
        
        let PatStruct {
            attrs,
            path,
            brace_token: _,
            fields,
            dot2_token } = pat_struct;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #path });

        let mut new_fields = Vec::<TokenStream>::new();

        for field in &fields {
            let FieldPat {
                attrs: field_attrs,
                member,
                colon_token,
                pat } = field;
            let mut field_result = TokenStream::new();
            let mut field_tokens = Vec::<TokenStream>::new();

            field_tokens.push(quote! { #(#field_attrs)* #member });

            if let Some(colon) = colon_token {
                field_tokens.push(quote! { #colon });
            }

            let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
            field_tokens.push(quote! { #new_pat });

            field_result.extend(field_tokens.into_iter());
            new_fields.push(field_result);
        }

        if let Some(dot2) = dot2_token {
            tokens.push(quote! { { #(#new_fields),* , #dot2 } });
        }
        else {
            tokens.push(quote! { { #(#new_fields),* } });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn pat_tuple(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_tuple;

        if let Pat::Tuple(tuple) = parsed {
            pat_tuple = tuple;
        }
        else { return Err(Error::new(parsed.span(), "expected PatTuple object in pattern")); }
        
        let PatTuple {
            attrs,
            paren_token: _,
            elems } = pat_tuple;

        let mut new_elems = Vec::<TokenStream>::new();

        for elem in &elems {
            let new_elem = self.callback(context, CallbackType::Pat, elem.to_token_stream())?;
            new_elems.push(new_elem);
        }

        Ok(quote! { #(#attrs)* ( #(#new_elems),* ) })
    }
    pub fn pat_tuple_struct(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;
        let pat_ts;

        if let Pat::TupleStruct(ts) = parsed {
            pat_ts = ts;
        }
        else { return Err(Error::new(parsed.span(), "expected PatTupleStruct object in pattern")); }
        
        let PatTupleStruct {
            attrs,
            path,
            pat } = pat_ts;

        let new_pat = self.callback(context, CallbackType::PatTuple, pat.to_token_stream())?;

        Ok(quote! { #(#attrs)* #path #new_pat })
    }
    pub fn pat_type(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        // the Pat parser doesn't actually interpret this, we need to find a janky way
        // to reinterpret this pattern cast, which is basically define a stub local
        // statement, parse that, then get the PatType object out of it.
        let PatType {
            attrs,
            pat,
            colon_token,
            ty } = get_pat_type(tokens)?;

        let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_pat #colon_token #ty })
    }
    pub fn pat_wild(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Pat>(tokens)?;

        if let Pat::Wild(wild) = parsed {
            Ok(wild.to_token_stream())
        }
        else {
            Err(Error::new(parsed.span(), "expected PatWild object in pattern"))
        }
    }
    pub fn expr(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Expr>(tokens)?;

        let result = match parsed {
            Expr::Array(ref array) => self.callback(context, CallbackType::ExprArray, array.to_token_stream()),
            Expr::Assign(ref assign) => self.callback(context, CallbackType::ExprAssign, assign.to_token_stream()),
            Expr::AssignOp(ref assign) => self.callback(context, CallbackType::ExprAssignOp, assign.to_token_stream()),
            Expr::Async(ref async_) => self.callback(context, CallbackType::ExprAsync, async_.to_token_stream()),
            Expr::Await(ref await_) => self.callback(context, CallbackType::ExprAwait, await_.to_token_stream()),
            Expr::Binary(ref binary) => self.callback(context, CallbackType::ExprBinary, binary.to_token_stream()),
            Expr::Block(ref block) => self.callback(context, CallbackType::ExprBlock, block.to_token_stream()),
            Expr::Box(ref box_) => self.callback(context, CallbackType::ExprBox, box_.to_token_stream()),
            Expr::Break(ref break_) => self.callback(context, CallbackType::ExprBreak, break_.to_token_stream()),
            Expr::Call(ref call) => self.callback(context, CallbackType::ExprCall, call.to_token_stream()),
            Expr::Cast(ref cast) => self.callback(context, CallbackType::ExprCast, cast.to_token_stream()),
            Expr::Closure(ref closure) => self.callback(context, CallbackType::ExprClosure, closure.to_token_stream()),
            Expr::Continue(ref continue_) => self.callback(context, CallbackType::ExprContinue, continue_.to_token_stream()),
            Expr::Field(ref field) => self.callback(context, CallbackType::ExprField, field.to_token_stream()),
            Expr::ForLoop(ref loop_) => self.callback(context, CallbackType::ExprForLoop, loop_.to_token_stream()),
            Expr::Group(ref group) => self.callback(context, CallbackType::ExprGroup, group.to_token_stream()),
            Expr::If(ref if_) => self.callback(context, CallbackType::ExprIf, if_.to_token_stream()),
            Expr::Index(ref index) => self.callback(context, CallbackType::ExprIndex, index.to_token_stream()),
            Expr::Let(ref let_) => self.callback(context, CallbackType::ExprLet, let_.to_token_stream()),
            Expr::Lit(ref lit) => self.callback(context, CallbackType::ExprLit, lit.to_token_stream()),
            Expr::Loop(ref loop_) => self.callback(context, CallbackType::ExprLoop, loop_.to_token_stream()),
            Expr::Macro(ref macro_) => self.callback(context, CallbackType::ExprMacro, macro_.to_token_stream()),
            Expr::Match(ref match_) => self.callback(context, CallbackType::ExprMatch, match_.to_token_stream()),
            Expr::MethodCall(ref method) => self.callback(context, CallbackType::ExprMethodCall, method.to_token_stream()),
            Expr::Paren(ref paren) => self.callback(context, CallbackType::ExprParen, paren.to_token_stream()),
            Expr::Path(ref path) => self.callback(context, CallbackType::ExprPath, path.to_token_stream()),
            Expr::Range(ref range) => self.callback(context, CallbackType::ExprRange, range.to_token_stream()),
            Expr::Reference(ref ref_) => self.callback(context, CallbackType::ExprReference, ref_.to_token_stream()),
            Expr::Repeat(ref repeat) => self.callback(context, CallbackType::ExprRepeat, repeat.to_token_stream()),
            Expr::Return(ref ret) => self.callback(context, CallbackType::ExprReturn, ret.to_token_stream()),
            Expr::Struct(ref struct_) => self.callback(context, CallbackType::ExprStruct, struct_.to_token_stream()),
            Expr::Try(ref try_) => self.callback(context, CallbackType::ExprTry, try_.to_token_stream()),
            Expr::TryBlock(ref try_) => self.callback(context, CallbackType::ExprTryBlock, try_.to_token_stream()),
            Expr::Tuple(ref tuple) => self.callback(context, CallbackType::ExprTuple, tuple.to_token_stream()),
            Expr::Type(ref type_) => self.callback(context, CallbackType::ExprType, type_.to_token_stream()),
            Expr::Unary(ref unary) => self.callback(context, CallbackType::ExprUnary, unary.to_token_stream()),
            Expr::Unsafe(ref unsafe_) => self.callback(context, CallbackType::ExprUnsafe, unsafe_.to_token_stream()),
            Expr::Verbatim(ref verbatim) => self.callback(context, CallbackType::Verbatim, verbatim.clone()),
            Expr::While(ref while_) => self.callback(context, CallbackType::ExprWhile, while_.to_token_stream()),
            Expr::Yield(ref yield_) => self.callback(context, CallbackType::ExprYield, yield_.to_token_stream()),
            _ => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn expr_array(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprArray {
            attrs,
            bracket_token: _,
            elems } = parse2::<ExprArray>(tokens)?;

        let mut new_elems = Vec::<TokenStream>::new();

        for elem in &elems {
            let new_elem = self.callback(context, CallbackType::Expr, elem.to_token_stream())?;
            new_elems.push(new_elem);
        }

        Ok(quote! { #(#attrs)* [ #(#new_elems),* ] })
    }
    pub fn expr_assign(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprAssign {
            attrs,
            left,
            eq_token,
            right } = parse2::<ExprAssign>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        let new_left = self.callback(context, CallbackType::Expr, left.to_token_stream())?;
        tokens.push(new_left);

        tokens.push(quote! { #eq_token });

        let new_right = self.callback(context, CallbackType::Expr, right.to_token_stream())?;
        tokens.push(new_right);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_assign_op(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprAssignOp {
            attrs,
            left,
            op,
            right } = parse2::<ExprAssignOp>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        let new_left = self.callback(context, CallbackType::Expr, left.to_token_stream())?;
        tokens.push(new_left);
        tokens.push(quote! { #op });

        let new_right = self.callback(context, CallbackType::Expr, right.to_token_stream())?;
        tokens.push(new_right);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_async(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprAsync {
            attrs,
            async_token,
            capture,
            block } = parse2::<ExprAsync>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #async_token });

        if let Some(move_token) = capture {
            tokens.push(quote! { #move_token });
        }

        let new_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;
        tokens.push(new_block);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_await(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprAwait {
            attrs,
            base,
            dot_token,
            await_token } = parse2::<ExprAwait>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        let new_expr = self.callback(context, CallbackType::Expr, base.to_token_stream())?;
        tokens.push(quote! { #new_expr #dot_token #await_token });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_binary(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprBinary {
            attrs,
            left,
            op,
            right } = parse2::<ExprBinary>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });
        
        let new_left = self.callback(context, CallbackType::Expr, left.to_token_stream())?;
        tokens.push(new_left);
        tokens.push(quote! { #op });

        let new_right = self.callback(context, CallbackType::Expr, right.to_token_stream())?;
        tokens.push(new_right);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_block(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprBlock {
            attrs,
            label,
            block } = parse2::<ExprBlock>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(label_token) = label {
            tokens.push(quote! { #label_token });
        }

        let new_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;
        tokens.push(new_block);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_box(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprBox {
            attrs,
            box_token,
            expr } = parse2::<ExprBox>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #box_token #new_expr })
    }
    pub fn expr_break(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprBreak {
            attrs,
            break_token,
            label,
            expr } = parse2::<ExprBreak>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #break_token });

        if let Some(label_token) = label {
            tokens.push(quote! { #label_token });
        }

        if let Some(expr_opt) = expr {
            let new_expr = self.callback(context, CallbackType::Expr, expr_opt.to_token_stream())?;
            tokens.push(new_expr);
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_call(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprCall {
            attrs,
            func,
            paren_token: _,
            args } = parse2::<ExprCall>(tokens)?;

        let new_func = self.callback(context, CallbackType::Expr, func.to_token_stream())?;
        let mut new_args = Vec::<TokenStream>::new();

        for arg in &args {
            let new_arg = self.callback(context, CallbackType::Expr, arg.to_token_stream())?;
            new_args.push(new_arg);
        }

        Ok(quote! { #(#attrs)* #new_func ( #(#new_args),* ) })
    }
    pub fn expr_cast(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprCast {
            attrs,
            expr,
            as_token,
            ty } = parse2::<ExprCast>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr #as_token #ty })
    }
    pub fn expr_closure(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprClosure {
            attrs,
            movability,
            asyncness,
            capture,
            or1_token,
            inputs,
            or2_token,
            output,
            body } = parse2::<ExprClosure>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(static_token) = movability {
            tokens.push(quote! { #static_token });
        }

        if let Some(async_token) = asyncness {
            tokens.push(quote! { #async_token });
        }

        if let Some(move_token) = capture {
            tokens.push(quote! { #move_token });
        }

        let mut new_inputs = Vec::<TokenStream>::new();

        for input in &inputs {
            let new_input = self.callback(context, CallbackType::Pat, input.to_token_stream())?;
            new_inputs.push(new_input);
        }

        tokens.push(quote! { #or1_token #(#new_inputs),* #or2_token #output });

        let new_expr = self.callback(context, CallbackType::Expr, body.to_token_stream())?;
        tokens.push(new_expr);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_continue(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ExprContinue>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn expr_field(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprField {
            attrs,
            base,
            dot_token,
            member } = parse2::<ExprField>(tokens)?;

        let new_base = self.callback(context, CallbackType::Expr, base.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_base #dot_token #member })
    }
    pub fn expr_for_loop(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprForLoop {
            attrs,
            label,
            for_token,
            pat,
            in_token,
            expr,
            body } = parse2::<ExprForLoop>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(label_token) = label {
            tokens.push(quote! { #label_token });
        }

        let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
        tokens.push(quote! { #for_token #new_pat #in_token });

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        tokens.push(new_expr);

        let new_block = self.callback(context, CallbackType::Block, body.to_token_stream())?;
        tokens.push(new_block);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_group(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Expr>(tokens)?;
        let expr_group;

        if let Expr::Group(group) = parsed {
            expr_group = group;
        }
        else { return Err(Error::new(parsed.span(), "expected ExprGroup object in expression")); }
        
        let ExprGroup {
            attrs,
            group_token: _,
            expr } = expr_group;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr })
    }
    pub fn expr_if(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprIf {
            attrs,
            if_token,
            cond,
            then_branch,
            else_branch } = parse2::<ExprIf>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #if_token });

        let new_expr = self.callback(context, CallbackType::Expr, cond.to_token_stream())?;
        tokens.push(new_expr);

        let new_block = self.callback(context, CallbackType::Block, then_branch.to_token_stream())?;
        tokens.push(new_block);

        if let Some((else_token, else_expr)) = else_branch {
            let new_expr = self.callback(context, CallbackType::Expr, else_expr.to_token_stream())?;
            tokens.push(quote! { #else_token #new_expr });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_index(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprIndex {
            attrs,
            expr,
            bracket_token: _,
            index } = parse2::<ExprIndex>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        let new_index = self.callback(context, CallbackType::Expr, index.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr [ #new_index ] })
    }
    pub fn expr_let(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprLet {
            attrs,
            let_token,
            pat,
            eq_token,
            expr } = parse2::<ExprLet>(tokens)?;

        let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #let_token #new_pat #eq_token #new_expr })
    }
    pub fn expr_lit(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprLit {
            attrs,
            lit } = parse2::<ExprLit>(tokens)?;

        let new_lit = self.callback(context, CallbackType::Lit, lit.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_lit })
    }
    pub fn expr_loop(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprLoop {
            attrs,
            label,
            loop_token,
            body } = parse2::<ExprLoop>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(label_token) = label {
            tokens.push(quote! { #label_token });
        }

        tokens.push(quote! { #loop_token });
        
        let new_block = self.callback(context, CallbackType::Block, body.to_token_stream())?;
        tokens.push(new_block);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_macro(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ExprMacro>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn expr_match(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprMatch {
            attrs,
            match_token,
            expr,
            brace_token: _,
            arms } = parse2::<ExprMatch>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #match_token });

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        tokens.push(new_expr);

        let mut new_arms = Vec::<TokenStream>::new();

        for arm in &arms {
            let new_arm = self.callback(context, CallbackType::Arm, arm.to_token_stream())?;
            new_arms.push(new_arm);
        }

        tokens.push(quote! {
            {
                #(#new_arms)*
            }
        });
        
        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_method_call(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprMethodCall {
            attrs,
            receiver,
            dot_token,
            method,
            turbofish,
            paren_token: _,
            args } = parse2::<ExprMethodCall>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        let new_receiver = self.callback(context, CallbackType::Expr, receiver.to_token_stream())?;
        tokens.push(new_receiver);

        tokens.push(quote! { #dot_token #method });

        if let Some(turbo) = turbofish {
            tokens.push(quote! { #turbo });
        }

        let mut new_args = Vec::<TokenStream>::new();

        for arg in &args {
            let new_arg = self.callback(context, CallbackType::Expr, arg.to_token_stream())?;
            new_args.push(new_arg);
        }

        tokens.push(quote! { ( #(#new_args),* ) });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_paren(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprParen {
            attrs,
            paren_token: _,
            expr } = parse2::<ExprParen>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* ( #new_expr ) })
    }
    pub fn expr_path(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<ExprPath>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn expr_range(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprRange {
            attrs,
            from,
            limits,
            to } = parse2::<ExprRange>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        if let Some(from_expr) = from {
            let new_expr = self.callback(context, CallbackType::Expr, from_expr.to_token_stream())?;
            tokens.push(new_expr);
        }

        tokens.push(quote! { #limits });

        if let Some(to_expr) = to {
            let new_expr = self.callback(context, CallbackType::Expr, to_expr.to_token_stream())?;
            tokens.push(new_expr);
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_reference(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprReference {
            attrs,
            and_token,
            raw: _,
            mutability,
            expr } = parse2::<ExprReference>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #and_token });

        if let Some(mut_token) = mutability {
            tokens.push(quote! { #mut_token });
        }

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        tokens.push(new_expr);

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_repeat(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprRepeat {
            attrs,
            bracket_token: _,
            expr,
            semi_token,
            len } = parse2::<ExprRepeat>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        let new_len = self.callback(context, CallbackType::Expr, len.to_token_stream())?;

        Ok(quote! { #(#attrs)* [ #new_expr #semi_token #new_len ] })
    }
    pub fn expr_return(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprReturn {
            attrs,
            return_token,
            expr } = parse2::<ExprReturn>(tokens)?;

        if let Some(ret_expr) = expr {
            let new_expr = self.callback(context, CallbackType::Expr, ret_expr.to_token_stream())?;
     
            Ok(quote! { #(#attrs)* #return_token #new_expr })
        }
        else {
            Ok(quote! { #(#attrs)* #return_token })
        }
    }
    pub fn expr_struct(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprStruct {
            attrs,
            path,
            brace_token: _,
            fields,
            dot2_token,
            rest } = parse2::<ExprStruct>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* #path });

        let mut new_fields = Vec::<TokenStream>::new();

        for field in &fields {
            let new_field = self.callback(context, CallbackType::FieldValue, field.to_token_stream())?;
            new_fields.push(new_field);
        }

        let mut end_data = Vec::<TokenStream>::new();

        if let Some(dot2) = dot2_token {
            end_data.push(quote! { , #dot2 });
        }

        if let Some(rest_token) = rest {
            if end_data.len() > 0 { end_data.push(quote! { #rest_token }); }
            else { end_data.push(quote! { , #rest_token }); }
        }

        tokens.push(quote! { { #(#new_fields),* #(#end_data)* } });

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn expr_try(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprTry {
            attrs,
            expr,
            question_token } = parse2::<ExprTry>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr #question_token })
    }
    pub fn expr_try_block(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprTryBlock {
            attrs,
            try_token,
            block } = parse2::<ExprTryBlock>(tokens)?;

        let new_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;

        Ok(quote! { #(#attrs)* #try_token #new_block })
    }
    pub fn expr_tuple(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprTuple {
            attrs,
            paren_token: _,
            elems } = parse2::<ExprTuple>(tokens)?;

        let mut new_elems = Vec::<TokenStream>::new();

        for elem in &elems {
            let new_elem = self.callback(context, CallbackType::Expr, elem.to_token_stream())?;
            new_elems.push(new_elem);
        }

        Ok(quote! { #(#attrs)* ( #(#new_elems),* ) })
    }
    pub fn expr_type(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprType {
            attrs,
            expr,
            colon_token,
            ty } = parse2::<ExprType>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #new_expr #colon_token #ty })
    }
    pub fn expr_unary(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprUnary {
            attrs,
            op,
            expr } = parse2::<ExprUnary>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;

        Ok(quote! { #(#attrs)* #op #new_expr })
    }
    pub fn expr_unsafe(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprUnsafe {
            attrs,
            unsafe_token,
            block } = parse2::<ExprUnsafe>(tokens)?;

        let new_block = self.callback(context, CallbackType::Block, block.to_token_stream())?;

        Ok(quote! { #(#attrs)* #unsafe_token #new_block })
    }
    pub fn expr_while(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprWhile {
            attrs,
            label,
            while_token,
            cond,
            body } = parse2::<ExprWhile>(tokens)?;

        let new_cond = self.callback(context, CallbackType::Expr, cond.to_token_stream())?;
        let new_body = self.callback(context, CallbackType::Block, body.to_token_stream())?;

        if let Some(lbl) = label {
            Ok(quote! { #(#attrs)* #lbl #while_token #new_cond #new_body })
        }
        else {
            Ok(quote! { #(#attrs)* #while_token #new_cond #new_body })
        }
    }
    pub fn expr_yield(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let ExprYield {
            attrs,
            yield_token,
            expr } = parse2::<ExprYield>(tokens)?;

        if let Some(yield_expr) = expr {
            let new_expr = self.callback(context, CallbackType::Expr, yield_expr.to_token_stream())?;

            Ok(quote! { #(#attrs)* #yield_token #new_expr })
        }
        else {
            Ok(quote! { #(#attrs)* #yield_token })
        }
    }
    pub fn arm(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let Arm {
            attrs,
            pat,
            guard,
            fat_arrow_token,
            body,
            comma } = parse2::<Arm>(tokens)?;
        let mut result = TokenStream::new();
        let mut tokens = Vec::<TokenStream>::new();

        tokens.push(quote! { #(#attrs)* });

        let new_pat = self.callback(context, CallbackType::Pat, pat.to_token_stream())?;
        tokens.push(new_pat);

        if let Some((if_token, if_expr)) = guard {
            let new_expr = self.callback(context, CallbackType::Expr, if_expr.to_token_stream())?;
            tokens.push(quote! { #if_token #new_expr });
        }

        tokens.push(quote! { #fat_arrow_token });

        let new_expr = self.callback(context, CallbackType::Expr, body.to_token_stream())?;
        tokens.push(new_expr);

        if let Some(comma_token) = comma {
            tokens.push(quote! { #comma_token });
        }

        result.extend(tokens.into_iter());
        Ok(result)
    }
    pub fn lit(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<Lit>(tokens)?;

        let result = match parsed {
            Lit::Str(ref str_) => self.callback(context, CallbackType::LitStr, str_.to_token_stream()),
            Lit::ByteStr(ref str_) => self.callback(context, CallbackType::LitByteStr, str_.to_token_stream()),
            Lit::Byte(ref byte) => self.callback(context, CallbackType::LitByte, byte.to_token_stream()),
            Lit::Char(ref char_) => self.callback(context, CallbackType::LitChar, char_.to_token_stream()),
            Lit::Int(ref int) => self.callback(context, CallbackType::LitInt, int.to_token_stream()),
            Lit::Float(ref float) => self.callback(context, CallbackType::LitFloat, float.to_token_stream()),
            Lit::Bool(ref bool_) => self.callback(context, CallbackType::LitBool, bool_.to_token_stream()),
            Lit::Verbatim(_) => Ok(parsed.to_token_stream()),
        }?;
        Ok(result)
    }
    pub fn lit_str(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitStr>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_byte_str(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitByteStr>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_byte(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitByte>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_char(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitChar>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_int(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitInt>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_float(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitFloat>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn lit_bool(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let parsed = parse2::<LitBool>(tokens)?;
        Ok(parsed.to_token_stream())
    }
    pub fn field_value(&self, context: &Context, tokens: TokenStream) -> Result<TokenStream> {
        let FieldValue {
            attrs,
            member,
            colon_token,
            expr } = parse2::<FieldValue>(tokens)?;

        let new_expr = self.callback(context, CallbackType::Expr, expr.to_token_stream())?;
        
        if let Some(colon) = colon_token {
            Ok(quote! { #(#attrs)* #member #colon #new_expr })
        }
        else {
            Ok(quote! { #(#attrs)* #member #new_expr })
        }
    }
    pub fn verbatim(&self, _: &Context, tokens: TokenStream) -> Result<TokenStream> {
        Ok(tokens)
    }
}
