use raylib::prelude::Vector2;
use std::collections::HashMap;
use std::any::{Any, TypeId};

#[derive(Debug)]
struct TestComponent {
    num: i32
}

impl TestComponent{
    fn new(num: i32) -> TestComponent{
        TestComponent{num: num}
    }
}

#[derive(Debug)]
struct TestComponent2 {
    num: i32
}

impl TestComponent2{
    fn new(num: i32) -> TestComponent{
        TestComponent{num: num}
    }
}

#[derive(Debug)]
struct Entity {
    position: Vector2,
    componentsIndexMap: HashMap<TypeId, usize>,
    components: Vec<Box<dyn Any>>
    // components: HashMap<TypeId, Box<dyn Any>>
}

impl Entity {
    fn new(position: Vector2) -> Entity {
        Entity{position: position, componentsIndexMap: HashMap::new(), components: Vec::new()}
    }

    fn add_component(&mut self, new_component: Box<dyn Any>) {
        self.componentsIndexMap.insert((*new_component).type_id(), self.components.len());
        self.components.push(new_component);
    }

    fn get_component_ref<T: Any>(&self) -> Option<&T>{
        let result_index = self.componentsIndexMap.get(&TypeId::of::<T>());
        return match result_index {
            Some(index) => self.components[*index].downcast_ref::<T>(),
            None => None
        }
    }

    fn get_component_mut<T: Any>(&mut self) -> Option<&mut T>{
        let result_index = self.componentsIndexMap.get_mut(&TypeId::of::<T>());
        return match result_index {
            Some(index) => self.components[*index].downcast_mut::<T>(),
            None => None
        }
    }
}


fn main() {
    let mut e = Entity::new(Vector2::new(0f32, 0f32));
    let x = Box::new(TestComponent::new(0));
    e.add_component(x);
    e.add_component(Box::new(TestComponent2::new(0)));
    // println!("{:#?}", TypeId::of::<TestComponent>());
    // println!("{:#?}", TestComponent::new(0).type_id());
    let option: Option<&TestComponent> = e.get_component_mut();
    let option2: Option<&TestComponent2> = e.get_component_mut();
    if option2.is_some() {
        // option2.as_mut().unwrap().num += 1;
        println!("{:#?}", option2);
    }
    if option.is_some() {
        // option.as_mut().unwrap().num -= 1;
        println!("{:#?}", option);
    }
    println!("{:#?}", e);
}