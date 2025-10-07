use std::{borrow::Cow, marker::PhantomData};

use mlua::{FromLua, FromLuaMulti, Function, IntoLua, IntoLuaMulti, Lua, Value};

use crate::MaybeSend;

use super::{Type, Typed, TypedMultiValue};

/// A function parameter type representation
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Param {
    pub doc: Option<Cow<'static, str>>,
    ///If the parameter has a name (will default to Param{number} if None)
    pub name: Option<Cow<'static, str>>,
    ///The type of the parameter
    pub(crate) ty: Type,
}

impl Param {
    /// Set the parameters name
    pub fn set_name(&mut self, name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set the parameters doc comment
    pub fn set_doc(&mut self, doc: impl Into<Cow<'static, str>>) -> &mut Self {
        self.doc = Some(doc.into());
        self
    }
}

/// A function parameter type representation
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Return {
    pub doc: Option<Cow<'static, str>>,
    ///The type of the return
    pub(crate) ty: Type,
}

impl Return {
    /// Set the parameters doc comment
    pub fn set_doc(&mut self, doc: impl Into<Cow<'static, str>>) -> &mut Self {
        self.doc = Some(doc.into());
        self
    }
}

impl<I: Into<Cow<'static, str>>> From<(I, Type)> for Param {
    fn from((name, ty): (I, Type)) -> Self {
        Param {
            doc: None,
            name: Some(name.into()),
            ty,
        }
    }
}

impl From<Type> for Param {
    fn from(value: Type) -> Self {
        Param {
            doc: None,
            name: None,
            ty: value,
        }
    }
}

/// Used to purely get function type information without converting it to anything
/// else.
pub trait IntoTypedFunction<Params: TypedMultiValue, Response: TypedMultiValue> {
    fn into_typed_function(self, lua: &Lua) -> mlua::Result<TypedFunction<Params, Response>>;
}

impl<F, Params, Response> IntoTypedFunction<Params, Response> for F
where
    Params: TypedMultiValue + FromLuaMulti,
    Response: TypedMultiValue + IntoLuaMulti,
    F: Fn(&Lua, Params) -> mlua::Result<Response> + MaybeSend + 'static,
{
    fn into_typed_function(self, lua: &Lua) -> mlua::Result<TypedFunction<Params, Response>> {
        Ok(TypedFunction {
            inner: lua.create_function(self)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> IntoTypedFunction<Params, Response> for Function
where
    Params: TypedMultiValue + FromLuaMulti,
    Response: TypedMultiValue + IntoLuaMulti,
{
    fn into_typed_function(self, _lua: &Lua) -> mlua::Result<TypedFunction<Params, Response>> {
        Ok(TypedFunction {
            inner: self,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> IntoTypedFunction<Params, Response> for &TypedFunction<Params, Response>
where
    Params: TypedMultiValue + FromLuaMulti,
    Response: TypedMultiValue + IntoLuaMulti,
{
    fn into_typed_function(self, _lua: &Lua) -> mlua::Result<TypedFunction<Params, Response>> {
        Ok(TypedFunction {
            inner: self.inner.clone(),
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> IntoTypedFunction<Params, Response> for ()
where
    Params: TypedMultiValue + FromLuaMulti,
    Response: TypedMultiValue + IntoLuaMulti,
{
    fn into_typed_function(self, lua: &Lua) -> mlua::Result<TypedFunction<Params, Response>> {
        Ok(TypedFunction {
            inner: lua.create_function(|_, _: Params| Ok(()))?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

/// Helper to bake the type information for a lua [`Function`][mlua::Function]. This makes repeated
/// calls to the [`Function`][mlua::Function]'s [`call`][mlua::Function::call] all the same with
/// enforced arguments and return types.
pub struct TypedFunction<Params, Response>
where
    Params: TypedMultiValue,
    Response: TypedMultiValue,
{
    inner: Function,
    _p: PhantomData<Params>,
    _r: PhantomData<Response>,
}

impl<Params, Response> TypedFunction<Params, Response>
where
    Params: TypedMultiValue + IntoLuaMulti,
    Response: TypedMultiValue + FromLuaMulti,
{
    /// Same as [Function::call] but with the param and return
    /// types already specified
    pub fn call(&self, params: Params) -> mlua::Result<Response> {
        self.inner.call::<Response>(params)
    }

    /// Same as [Function::call] but with the param and return
    /// types already specified
    ///
    /// # Safety
    ///
    /// Panics if any lua errors occur
    pub unsafe fn call_unsafe(&self, params: Params) -> Response {
        self.inner.call::<Response>(params).unwrap()
    }

    /// Create a typed function from a rust function.
    ///
    /// This will call [`Lua::create_function`] under the hood
    pub fn from_rust<F>(&self, lua: &Lua, func: F) -> mlua::Result<Self>
    where
        Params: TypedMultiValue + FromLuaMulti,
        Response: TypedMultiValue + IntoLuaMulti,
        F: Fn(&Lua, Params) -> mlua::Result<Response> + MaybeSend + 'static,
    {
        Ok(Self {
            inner: lua.create_function(func)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> FromLua for TypedFunction<Params, Response>
where
    Params: TypedMultiValue,
    Response: TypedMultiValue,
{
    fn from_lua(value: Value, lua: &Lua) -> mlua::prelude::LuaResult<Self> {
        Ok(Self {
            inner: FromLua::from_lua(value, lua)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> IntoLua for TypedFunction<Params, Response>
where
    Params: TypedMultiValue,
    Response: TypedMultiValue,
{
    fn into_lua(self, _lua: &Lua) -> mlua::prelude::LuaResult<Value> {
        Ok(Value::Function(self.inner))
    }
}

impl<Params, Response> Typed for TypedFunction<Params, Response>
where
    Params: TypedMultiValue,
    Response: TypedMultiValue,
{
    fn ty() -> Type {
        Type::Function {
            params: Params::get_types_as_params(),
            returns: Response::get_types().into_iter().map(|ty| Return { doc: None, ty }).collect(),
        }
    }
}
