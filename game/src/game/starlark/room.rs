use pythoneer_macros::class;
use starlark::values::ValueTyped;
use starlark::values::none::NoneType;
use anyhow::bail;
use crate::game::starlark::level::Key;

class! {
    pub Room {
        let pos: (i32, i32);
        let size: (u32, u32);
        let connections: Vec<Connection::ClassV> =;
        let item: Option<Value> =;

        mut item {
            if value.is_none() {
                *self.item.borrow_mut() = None;
            } else {
                Key::from_value(value)?;
                *self.item.borrow_mut() = Some(value);
            }
            Ok(())
        }

        fn connect(name: String, other: ValueTyped<'v, Mut<'v>>, #[starlark(require = named, default = false)] locked: bool, key: Option<ValueTyped<'v, Key::Mut>>) -> NoneType {
            if this.connections.borrow().iter().any(|connection| name == *connection.name.borrow()) {
                bail!("Connection {name:?} already exists");
            }
            if locked && key.is_none() {
                bail!("Key is required for locked connections");
            }
            this.connections.borrow_mut().push(Connection::new(name, other.to_value(), locked, key.map(|key| key.as_ref().name.borrow().clone())));
            Ok(NoneType)
        }
    }
}

class! {
    pub Connection {
        let name: String;
        let room: Value;
        let locked: bool;
        let key: Option<String>;
    }
}
