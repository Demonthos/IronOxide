use std::cell::Ref;
use core::cell::RefMut;
use std::collections::HashMap;
use core::any::TypeId;
use std::any::Any;
use core::cell::RefCell;

#[derive(Debug)]
struct TestComponent1{
    x: i32,
}

#[derive(Debug)]
struct TestComponent2{
    x: i32,
}


// can panic
#[derive(Debug)]
struct ComponentSystem{
    inner: HashMap<TypeId, RefCell<Vec<RefCell<Box<dyn Any>>>>>
}

impl ComponentSystem {
    fn add_component<T: Any>(&mut self, new_component: T){
        let t = TypeId::of::<T>();
        let new_component_cell = RefCell::new(Box::new(new_component) as Box<dyn Any>);
        if self.inner.contains_key(&t){
            self.inner.get(&t).unwrap().borrow_mut().push(new_component_cell);
        }
        else{
            let mut v = Vec::new();
            v.push(new_component_cell);
            self.inner.insert(t, RefCell::new(v));
        }
    }

    fn get_all_mut<T: Any>(&self) -> RefMut<Vec<RefCell<Box<dyn Any>>>>{
        self.inner.get(&TypeId::of::<T>()).unwrap().borrow_mut()
    }

    fn get_all_ref<T: Any>(&self) -> Ref<Vec<RefCell<Box<dyn Any>>>>{
        self.inner.get(&TypeId::of::<T>()).unwrap().borrow()
    }

    // fn get_multable_mut<'a, T: Any>(&self, component: RefMut<'a, Vec<RefCell<Box<dyn Any>>>>, indexes: Vec<usize>) -> Vec<RefMut<'a, Box<dyn Any>>>{
    //     let mut results = Vec::new();
    //     for index in indexes{
    //         results.push(component[index].borrow_mut());
    //     }
    //     results
    // }

    // fn get_multable_ref<T: Any>(&self, index: usize) -> Ref<Box<dyn Any>>{
    //     self.get_all_ref::<T>()[index].borrow()
    // }
}

fn main() {
    let mut cs = ComponentSystem{inner: HashMap::new()};
    cs.add_component::<TestComponent1>(TestComponent1{x: 0});
    cs.add_component::<TestComponent2>(TestComponent2{x: 0});
    let x = cs.get_all_mut::<TestComponent1>();
    let y = x[0].borrow_mut();
    let z = cs.get_all_mut::<TestComponent2>();
    println!("{:#?}", x);
    println!("{:#?}", y);
    println!("{:#?}", z);
    
    println!("{:#?}", cs);
}