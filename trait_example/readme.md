## 概述

Rust 中的 trait （特性） 是一种定义共享行为的方式，即多态的实现方案，类似于 Java 语言中的接口（interface）。


## 基本定义与实现

通过 trait 关键字 特性，使用 impl 关键字实现 trait：

```rust

trait Animal {
    fn name(&self) -> String;
    fn speak(&self) {
        println!("{} cannot speak", self.name());
    }
}

struct Dog { breed: String }
struct Cat { breed: String }

impl Animal for Dog {
    fn name(&self) -> String { format!("Dog ({})", self.breed) }
    fn speak(&self) { println!("Woof!"); }
}

impl Animal for Cat {
    fn name(&self) -> String { format!("Cat ({})", self.breed) }
    fn speak(&self) { println!("Meow!"); }
}

```

## trait 作为参数

可通过三种方式使用 trait 约束函数参数：

Trait Bound 语法（最常用）：

```rust

fn introduce(a: impl Animal) {
    println!("{} says: ", a.name());
    a.speak();
}

```

泛型 + Trait Bound：

```rust

fn introduce<T: Animal>(a: T) { /* ... */ }

```


动态分发（trait 对象）：

```rust

fn introduce(a: &dyn Animal) { /* ... */ }

```

## 内置 trait

Rust 有许多预定义的 trait，例如：

* Debug：允许使用 {:?} 格式化输出
* Display：允许使用 {} 格式化输出
* Clone：支持深拷贝对象
* Copy：标记类型可按位复制（无需分配内存）
* Default：提供类型的默认值


## 高级特性

* 默认实现：trait 方法可以有默认行为（如示例中的 speak）

* Trait 继承：一个 trait 可以要求实现另一个 trait

```rust

trait Pet: Animal {
    fn is_tame(&self) -> bool { true }
}

```

关联类型：在 trait 中定义占位类型


```rust

trait Container {
    type Item;
    fn contains(&self, item: &Self::Item) -> bool;
}

```


## 使用场景

* 抽象行为：定义不同类型共享的接口
* 泛型约束：限制泛型类型必须实现某些行为
* 代码复用：通过默认实现减少重复代码


Trait 是 Rust 类型系统的核心，它提供了静态多态（泛型）和动态多态（trait 对象）两种方式，让代码既安全又灵活。