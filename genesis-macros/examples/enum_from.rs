use genesis_macros::EnumFrom;

fn main() {
    let en: Direction<i32> = DirectionUp::new(23).into();
    println!("{en:#?}");
}

// 若要查看宏展开是啥样的
// cargo install cargo-expand
// 运行
// cargo expand --example enum_from
#[derive(Debug, EnumFrom)]
enum Direction<T> {
    // 这种是Unnamed类型
    UP(DirectionUp<T>),
    // Down,
    // Left(u32),
    // 两种相同类型会报错,因为已经生成了同类型的impl方法
    // Right(u32),
    // 这种就是Named类型了
    // Right { a: u32 },
}

#[derive(Debug)]
struct DirectionUp<T> {
    #[allow(dead_code)]
    value: T,
}
impl<T> DirectionUp<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
//
// impl From<DirectionUp> for Direction {
//     fn from(value: DirectionUp) -> Self {
//         Direction::UP(value)
//     }
// }
