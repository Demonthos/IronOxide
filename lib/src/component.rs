use core::any::TypeId;
use core::cell::RefCell;
use core::cell::RefMut;
use std::any::Any;
use std::cell::Ref;
use std::collections::HashMap;

#[derive(Debug)]
struct TestComponent1 {
    x: i32,
}

#[derive(Debug)]
struct TestComponent2 {
    x: i32,
}

#[derive(Debug)]
struct ComponentSystem {
    inner: HashMap<TypeId, RefCell<Vec<RefCell<Box<dyn Any>>>>>,
}

impl ComponentSystem {
    fn add_component<T: Any>(&mut self, new_component: T) {
        let t = TypeId::of::<T>();
        let new_component_cell = RefCell::new(Box::new(new_component) as Box<dyn Any>);
        if self.inner.contains_key(&t) {
            self.inner
                .get(&t)
                .unwrap()
                .borrow_mut()
                .push(new_component_cell);
        } else {
            let mut v = Vec::new();
            v.push(new_component_cell);
            self.inner.insert(t, RefCell::new(v));
        }
    }

    fn get_all_mut<T: Any>(&self) -> RefMut<Vec<RefCell<Box<dyn Any>>>> {
        self.inner.get(&TypeId::of::<T>()).unwrap().borrow_mut()
    }

    fn get_all_ref<T: Any>(&self) -> Ref<Vec<RefCell<Box<dyn Any>>>> {
        self.inner.get(&TypeId::of::<T>()).unwrap().borrow()
    }
}
