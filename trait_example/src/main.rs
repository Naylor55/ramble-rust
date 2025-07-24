use std::fmt;

// 1. 基本 trait 定义与实现
trait Animal {
    //抽象函数，需要子类（实现类）实现此函数
    fn name(&self) -> String;

    // 默认实现，非抽象函数，子类可以直接调用
    fn speak(&self) {
        println!("{} cannot speak", self.name());
    }
}

// 2. 实现 trait
struct Dog {
    breed: String,
}
struct Cat {
    breed: String,
}

struct Rabbit {
    breed: String,
}

impl Animal for Dog {
    fn name(&self) -> String {
        format!("Dog ({})", self.breed)
    }
    fn speak(&self) {
        println!("Woof!");
    }
}

impl Animal for Cat {
    fn name(&self) -> String {
        format!("Cat ({})", self.breed)
    }
    fn speak(&self) {
        println!("Meow!");
    }
}

impl Animal for Rabbit {
    fn name(&self) -> String {
        format!("Rabbit({})", self.breed)
    }
}

// 3. Trait 作为参数
fn introduce_static<T: Animal>(animal: T) {
    println!("静态分发: {}", animal.name());
    animal.speak();
}

fn introduce_dynamic(animal: &dyn Animal) {
    println!("动态分发: {}", animal.name());
    animal.speak();
}

// 4. 内置 trait 示例
struct Point {
    x: i32,
    y: i32,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

// 5. Trait 继承
trait Pet: Animal {
    fn is_tame(&self) -> bool {
        true
    }
}

impl Pet for Dog {}
impl Pet for Cat {}

// 6. 关联类型
trait Container {
    type Item;
    fn contains(&self, item: &Self::Item) -> bool;
}

struct VecContainer(Vec<i32>);

impl Container for VecContainer {
    type Item = i32;
    fn contains(&self, item: &i32) -> bool {
        self.0.contains(item)
    }
}

fn main() {
    println!("hello,world! 你好，世界");
    // 测试基本 trait
    let dog: Dog = Dog {
        breed: "Golden Retriever".to_string(),
    };
    println!("dog name is : {}", dog.name());
    dog.speak();

    let cat: Cat = Cat {
        breed: "Siamese".to_string(),
    };
    println!("cat name is : {}", cat.name());
    cat.speak();

    let rabbit: Rabbit = Rabbit {
        breed: "Lionhead".to_string(),
    };
    println!("rabbit name is : {}", rabbit.name());
    rabbit.speak();

    // 静态分发和动态分发
    introduce_static(dog);
    introduce_dynamic(&cat);

    // 测试 Debug trait
    let point = Point { x: 10, y: 20 };
    println!("Point: {:?}", point);

    // 测试 Pet trait
    println!("Is cat tame? {}", cat.is_tame());

    // 测试关联类型
    let vec_container = VecContainer(vec![1, 2, 3, 4, 5]);
    println!("Container contains 3? {}", vec_container.contains(&3));
}
