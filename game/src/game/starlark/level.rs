use std::cell::{Ref, RefCell};
use super::room::Room;
use pythoneer_macros::class;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{UnpackValue, Value, ValueError};
use starlark::values::ValueLike;

fn get_or_init<T>(cell: &RefCell<Option<T>>, init: impl FnOnce() -> T) -> Ref<T> {
    let option = cell.borrow();
    if option.is_some() {
        Ref::map(option, |value| value.as_ref().unwrap())
    } else {
        drop(option);
        *cell.borrow_mut() = Some(init());
        Ref::map(cell.borrow(), |value| value.as_ref().unwrap())
    }
}

class! {
    pub Level {
        let rooms: Vec<Value> =;
        let start: Option<Value> =;
        let initial_pan_x: Option<i32> =;
        let initial_pan_y: Option<i32> =;
        let initial_zoom: Option<u32> =;
        let keys: Option<Value> =;

        mut start {
            value.downcast_ref_err::<Room::Mut>()?;
            *self.start.borrow_mut() = Some(value);
            Ok(())
        }

        mut initial_pan {
            if value.is_none() {
                *self.initial_pan_x.borrow_mut() = None;
                *self.initial_pan_y.borrow_mut() = None;
            } else {
                let items: Vec<i32> = UnpackTuple::unpack_value_err(value)?.items;
                if items.len() != 2 {
                    return Err(ValueError::IncorrectParameterType.into());
                }
                *self.initial_pan_x.borrow_mut() = Some(items[0]);
                *self.initial_pan_y.borrow_mut() = Some(items[1]);
            }
            Ok(())
        }

        mut initial_zoom {
            #[allow(clippy::cast_sign_loss)]
            if value.is_none() {
                *self.initial_zoom.borrow_mut() = None;
            } else {
                let val = value.unpack_i32().ok_or(ValueError::IncorrectParameterType)?;
                if !(0..=100).contains(&val) {
                    return Err(ValueError::IndexOutOfBound(val).into());
                }
                *self.initial_zoom.borrow_mut() = Some(val as u32);
            }
            Ok(())
        }

        pub keys {
            Some(*get_or_init(&self.keys, || heap.alloc_complex(Keys::new())))
        }

        fn room(pos: (i32, i32), size: (u32, u32)) -> Value<'v> {
            let room = heap.alloc(Room::new(pos, size));
            this.rooms.borrow_mut().push(room);
            Ok(room)
        }
    }
}

class! {
    pub Keys {
        let white: Option<Value> =;
        let red: Option<Value> =;
        let orange: Option<Value> =;
        let yellow: Option<Value> =;
        let green: Option<Value> =;
        let blue: Option<Value> =;
        let pink: Option<Value> =;
        let purple: Option<Value> =;

        pub white {
            Some(*get_or_init(&self.white, || heap.alloc_complex(Key::new("white".to_string()))))
        }

        pub red {
            Some(*get_or_init(&self.red, || heap.alloc_complex(Key::new("red".to_string()))))
        }

        pub orange {
            Some(*get_or_init(&self.orange, || heap.alloc_complex(Key::new("orange".to_string()))))
        }

        pub yellow {
            Some(*get_or_init(&self.yellow, || heap.alloc_complex(Key::new("yellow".to_string()))))
        }

        pub green {
            Some(*get_or_init(&self.green, || heap.alloc_complex(Key::new("green".to_string()))))
        }

        pub blue {
            Some(*get_or_init(&self.blue, || heap.alloc_complex(Key::new("blue".to_string()))))
        }

        pub pink {
            Some(*get_or_init(&self.pink, || heap.alloc_complex(Key::new("pink".to_string()))))
        }

        pub purple {
            Some(*get_or_init(&self.purple, || heap.alloc_complex(Key::new("purple".to_string()))))
        }
    }
}

class! {
    pub Key {
        let name: String;
    }
}
