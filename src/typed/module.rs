use std::{any::type_name, borrow::Cow, collections::BTreeMap};

use super::{generator::FunctionBuilder, Field, Func, Typed, TypedMultiValue};
use crate::{
    extras::{Module, ModuleFields, ModuleMethods},
    MaybeSend,
};
use mlua::{FromLuaMulti, IntoLua, IntoLuaMulti};

/// Builder that constructs type and documentation information for a module using the [`TypedModule`] trait
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedModuleBuilder {
    pub doc: Option<Cow<'static, str>>,

    pub nested_modules: BTreeMap<Cow<'static, str>, TypedModuleBuilder>,

    pub fields: BTreeMap<Cow<'static, str>, Field>,
    pub meta_fields: BTreeMap<Cow<'static, str>, Field>,

    pub functions: BTreeMap<Cow<'static, str>, Func>,
    pub methods: BTreeMap<Cow<'static, str>, Func>,
    pub meta_functions: BTreeMap<Cow<'static, str>, Func>,
    pub meta_methods: BTreeMap<Cow<'static, str>, Func>,

    queued_doc: Option<String>,
    parents: Vec<&'static str>,
}

impl TypedModuleBuilder {
    pub fn new<M: TypedModule>() -> mlua::Result<Self> {
        let mut builder = TypedModuleBuilder::default();

        if let Some(doc) = M::documentation() {
            builder.doc = Some(doc.into());
        }

        M::add_fields(&mut builder)?;
        M::add_methods(&mut builder)?;

        Ok(builder)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
            && self.nested_modules.is_empty()
            && self.functions.is_empty()
            && self.methods.is_empty()
            && self.is_meta_empty()
    }

    #[inline]
    pub fn is_meta_empty(&self) -> bool {
        self.meta_fields.is_empty()
            && self.meta_functions.is_empty()
            && self.meta_methods.is_empty()
    }
}

/// Typed variant of [`ModuleFields`]
pub trait TypedModuleFields {
    /// Queue a doc comment to be used with the nest `add` call
    fn document<V: AsRef<str>>(&mut self, doc: V) -> &mut Self;

    /// Typed variant of [`add_field`][ModuleFields::add_field] only collecting the type information
    fn add_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed;

    /// Typed variant of [`add_meta_field`][ModuleFields::add_meta_field] only collecting the type information
    fn add_meta_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed;

    /// Typed variant of [`add_module`][ModuleFields::add_module] only collecting the type information
    fn add_module<V>(&mut self, name: impl AsRef<str>) -> mlua::Result<()>
    where
        V: TypedModule;
}

/// Typed variant of [`ModuleMethods`]
pub trait TypedModuleMethods {
    /// Queue a doc comment to be used with the nest `add` call
    fn document<V: AsRef<str>>(&mut self, doc: V) -> &mut Self;

    /// Typed variant of [`add_function`][ModuleMethods::add_function] only collecting the type information
    fn add_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue;

    /// Typed variant of [`add_function`][ModuleMethods::add_function] only collecting the type information
    ///
    /// Pass an additional callback that allows for param names, param doc comments, and return doc
    /// comments to be specified.
    fn add_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>);

    /// Typed variant of [`add_meta_function`][ModuleMethods::add_meta_function] only collecting the type information
    fn add_meta_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue;

    /// Typed variant of [`add_meta_function`][ModuleMethods::add_meta_function] only collecting the type information
    ///
    /// Pass an additional callback that allows for param names, param doc comments, and return doc
    /// comments to be specified.
    fn add_meta_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>);

    /// Typed variant of [`add_method`][ModuleMethods::add_method] only collecting the type information
    fn add_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue;

    /// Typed variant of [`add_method`][ModuleMethods::add_method] only collecting the type information
    ///
    /// Pass an additional callback that allows for param names, param doc comments, and return doc
    /// comments to be specified.
    fn add_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>);

    /// Typed variant of [`add_meta_method`][ModuleMethods::add_meta_method] only collecting the type information
    fn add_meta_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue;

    /// Typed variant of [`add_meta_method`][ModuleMethods::add_meta_method] only collecting the type information
    ///
    /// Pass an additional callback that allows for param names, param doc comments, and return doc
    /// comments to be specified.
    fn add_meta_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>);
}

pub struct WrappedModule<'module, M>(pub &'module mut M);
impl<'module, M: ModuleFields> TypedModuleFields for WrappedModule<'module, M> {
    fn document<V: AsRef<str>>(&mut self, _doc: V) -> &mut Self {
        self
    }

    fn add_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed,
    {
        self.0.add_field(name.as_ref(), value)
    }

    fn add_meta_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed,
    {
        self.0.add_meta_field(name.as_ref(), value)
    }

    fn add_module<V>(&mut self, name: impl AsRef<str>) -> mlua::Result<()>
    where
        V: TypedModule,
    {
        self.0.add_module::<&str, V>(name.as_ref())
    }
}

impl<'module, M: ModuleMethods> TypedModuleMethods for WrappedModule<'module, M> {
    fn document<V: AsRef<str>>(&mut self, _doc: V) -> &mut Self {
        self
    }

    fn add_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.0
            .add_function::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        _generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        self.0
            .add_function::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_meta_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.0
            .add_meta_function::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_meta_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        _generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        self.0
            .add_meta_function::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.0.add_method::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        _generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        self.0.add_method::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_meta_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.0.add_meta_method::<&str, F, A, R>(name.as_ref(), function)
    }

    fn add_meta_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        function: F,
        _generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.0
            .add_meta_method::<&str, F, A, R>(name.as_ref(), function)
    }
}

impl<'lua> TypedModuleFields<'lua> for TypedModuleBuilder {
    fn document<V: AsRef<str>>(&mut self, doc: V) -> &mut Self {
        self.queued_doc = Some(doc.as_ref().into());
        self
    }

    fn add_module<V>(&mut self, name: impl AsRef<str>) -> mlua::Result<()>
    where
        V: TypedModule,
    {
        if self.parents.contains(&type_name::<V>()) {
            return Err(mlua::Error::runtime(format!(
                "infinite nested modules using: '{}'",
                type_name::<V>()
            )));
        }

        let mut nested = TypedModuleBuilder {
            parents: self
                .parents
                .iter()
                .map(|v| *v)
                .chain([type_name::<V>()])
                .collect(),
            ..Default::default()
        };

        if let Some(doc) = V::documentation() {
            nested.doc = Some(doc.into());
        }

        V::add_fields(&mut nested)?;
        V::add_methods(&mut nested)?;

        self.nested_modules.insert(name.as_ref().to_string().into(), nested);
        Ok(())
    }

    fn add_field<K, V>(&mut self, name: K, _value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed,
    {
        self.fields.insert(
            name.as_ref().to_string().into(),
            Field {
                ty: V::ty(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_meta_field<K, V>(&mut self, name: K, _value: V) -> mlua::Result<()>
    where
        K: AsRef<str>,
        V: IntoLua + Typed,
    {
        self.meta_fields.insert(
            name.as_ref().to_string().into(),
            Field {
                ty: V::ty(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }
}

impl TypedModuleMethods for TypedModuleBuilder {
    fn document<V: AsRef<str>>(&mut self, doc: V) -> &mut Self {
        self.queued_doc = Some(doc.as_ref().into());
        self
    }

    fn add_function<K, F, A, R>(&mut self, name: K, _function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.functions.insert(
            name.as_ref().to_string().into(),
            Func {
                params: A::get_types_as_params(),
                returns: R::get_types_as_returns(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        _function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        let mut builder = FunctionBuilder::<A, R>::default();
        generator(&mut builder);

        self.functions.insert(
            name.as_ref().to_string().into(),
            Func {
                params: builder.params,
                returns: builder.returns,
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_meta_function<K, F, A, R>(&mut self, name: K, _function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.meta_functions.insert(
            name.as_ref().to_string().into(),
            Func {
                params: A::get_types_as_params(),
                returns: R::get_types_as_returns(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_meta_function_with<K, F, A, R, G>(
        &mut self,
        name: K,
        _function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        let mut builder = FunctionBuilder::<A, R>::default();
        generator(&mut builder);

        self.meta_functions.insert(
            name.as_ref().to_string().into(),
            Func {
                params: builder.params,
                returns: builder.returns,
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_method<K, F, A, R>(&mut self, name: K, _function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.methods.insert(
            name.as_ref().to_string().into(),
            Func {
                params: A::get_types_as_params(),
                returns: R::get_types_as_returns(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        _function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        let mut builder = FunctionBuilder::<A, R>::default();
        generator(&mut builder);

        self.methods.insert(
            name.as_ref().to_string().into(),
            Func {
                params: builder.params,
                returns: builder.returns,
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_meta_method<K, F, A, R>(&mut self, name: K, _function: F) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
    {
        self.meta_methods.insert(
            name.as_ref().to_string().into(),
            Func {
                params: A::get_types_as_params(),
                returns: R::get_types_as_returns(),
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }

    fn add_meta_method_with<K, F, A, R, G>(
        &mut self,
        name: K,
        _function: F,
        generator: G,
    ) -> mlua::Result<()>
    where
        K: AsRef<str>,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        G: Fn(&mut FunctionBuilder<A, R>),
    {
        let mut builder = FunctionBuilder::<A, R>::default();
        generator(&mut builder);

        self.meta_methods.insert(
            name.as_ref().to_string().into(),
            Func {
                params: builder.params,
                returns: builder.returns,
                doc: self.queued_doc.take().map(|v| v.into()),
            },
        );
        Ok(())
    }
}

/// Sepecify a lua module (table) with fields and methods.
///
/// Only collects documentation and type information
pub trait TypedModule: Sized {
    /// Add module level documentation
    #[inline]
    fn documentation() -> Option<String> { None }

    /// Add fields to the module
    #[allow(unused_variables)]
    fn add_fields<F: TypedModuleFields>(fields: &mut F) -> mlua::Result<()> {
        Ok(())
    }

    /// Add methods/functions to the module
    #[allow(unused_variables)]
    fn add_methods<M: TypedModuleMethods>(methods: &mut M) -> mlua::Result<()> {
        Ok(())
    }
}

impl<T: TypedModule> Module for T {
    fn add_fields<F: ModuleFields>(fields: &mut F) -> mlua::Result<()> {
        let mut wrapped = WrappedModule(fields);
        T::add_fields(&mut wrapped)
    }

    fn add_methods<M: ModuleMethods>(methods: &mut M) -> mlua::Result<()> {
        let mut wrapped = WrappedModule(methods);
        T::add_methods(&mut wrapped)
    }
}
