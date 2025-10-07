use std::{any::type_name, marker::PhantomData};

use mlua::{FromLuaMulti, IntoLua, IntoLuaMulti};

use crate::MaybeSend;

#[derive(Default)]
pub struct LuaModule<M>(PhantomData<M>);
impl<M: Module> IntoLua for LuaModule<M> {
    fn into_lua(
        self,
        lua: &mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue> {
        let mut builder: ModuleBuilder = ModuleBuilder {
            table: lua.create_table()?,
            lua,
            parents: Vec::new(),
        };

        M::add_fields(&mut builder)?;
        M::add_methods(&mut builder)?;

        Ok(mlua::Value::Table(builder.table))
    }
}

/// Sepecify a lua module (table) with fields and methods
pub trait Module: Sized {
    /// Add fields to the module
    #[allow(unused_variables)]
    fn add_fields<F: ModuleFields>(fields: &mut F) -> mlua::Result<()> {
        Ok(())
    }

    /// Add methods/functions to the module
    #[allow(unused_variables)]
    fn add_methods<M: ModuleMethods>(methods: &mut M) -> mlua::Result<()> {
        Ok(())
    }

    fn module() -> LuaModule<Self> {
        LuaModule(PhantomData)
    }
}

/// Add table fields for a module
pub trait ModuleFields {
    /// Add a field to the module's table
    fn add_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: IntoLua,
        V: IntoLua;

    /// Add a field to the module's metatable
    fn add_meta_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: IntoLua,
        V: IntoLua;

    /// Add a nested module as a table in this module
    fn add_module<K, V>(&mut self, name: K) -> mlua::Result<()>
    where
        K: IntoLua,
        V: Module;
}

/// Add table functions and methods for a module
pub trait ModuleMethods {
    /// Add a function to this module's table
    fn add_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    /// Add a function to this module's metatable
    fn add_meta_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    /// Add a method to this module's table
    fn add_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    /// Add a method to this module's metatable
    fn add_meta_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;
}

/// Builder that construct a module based on the [`Module`] trait
pub struct ModuleBuilder<'a> {
    lua: &'a mlua::Lua,
    table: mlua::Table,
    parents: Vec<&'static str>,
}

impl<'a> ModuleFields for ModuleBuilder<'a> {
    fn add_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: IntoLua,
        V: IntoLua,
    {
        self.table.set(name, value)
    }

    fn add_meta_field<K, V>(&mut self, name: K, value: V) -> mlua::Result<()>
    where
        K: IntoLua,
        V: IntoLua,
    {
        let meta = match self.table.metatable() {
            Some(meta) => meta,
            None => {
                let meta = self.lua.create_table()?;
                self.table.set_metatable(Some(meta.clone()))?;
                meta
            }
        };

        meta.set(name, value)
    }

    fn add_module<K, V>(&mut self, name: K) -> mlua::Result<()>
    where
        K: IntoLua,
        V: Module,
    {
        if self.parents.contains(&type_name::<V>()) {
            return Err(mlua::Error::runtime(format!(
                "infinite nested modules using: '{}'",
                type_name::<V>()
            )));
        }

        let mut builder: ModuleBuilder = ModuleBuilder {
            table: self.lua.create_table()?,
            lua: self.lua,
            parents: self
                .parents
                .iter()
                .map(|v| *v)
                .chain([type_name::<V>()])
                .collect(),
        };

        V::add_fields(&mut builder)?;
        V::add_methods(&mut builder)?;

        self.table.set(name, builder.table)
    }
}

impl<'a> ModuleMethods for ModuleBuilder<'a> {
    fn add_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.table.set(name, self.lua.create_function(function)?)
    }

    fn add_meta_function<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let meta = match self.table.metatable() {
            Some(meta) => meta,
            None => {
                let meta = self.lua.create_table()?;
                self.table.set_metatable(Some(meta.clone()))?;
                meta
            }
        };

        meta.set(name, self.lua.create_function(function)?)
    }

    fn add_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.table.set(
            name,
            self.lua
                .create_function(move |lua, args: mlua::MultiValue| {
                    let this = mlua::Table::from_lua_multi(args.clone(), lua)?;
                    let rest = A::from_lua_multi(args, lua)?;
                    function(lua, this, rest)
                })?,
        )
    }

    fn add_meta_method<K, F, A, R>(&mut self, name: K, function: F) -> mlua::Result<()>
    where
        K: IntoLua,
        F: Fn(&mlua::Lua, mlua::Table, A) -> mlua::Result<R> + MaybeSend + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let meta = match self.table.metatable() {
            Some(meta) => meta,
            None => {
                let meta = self.lua.create_table()?;
                self.table.set_metatable(Some(meta.clone()))?;
                meta
            }
        };

        meta.set(
            name,
            self.lua
                .create_function(move |lua, args: mlua::MultiValue| {
                    let this = mlua::Table::from_lua_multi(args.clone(), lua)?;
                    let rest = A::from_lua_multi(args, lua)?;
                    function(lua, this, rest)
                })?,
        )
    }
}
