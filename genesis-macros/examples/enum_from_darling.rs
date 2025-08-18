use genesis_macros::EnumFromDarling;

fn main() {
    let en: Direction<i32> = DirectionUp::new(231).into();
    println!("{en:#?}");
}

#[allow(dead_code)]
#[derive(Debug, EnumFromDarling)]
enum Direction<T> {
    // 这种是Unnamed类型
    UP(DirectionUp<T>),
    Down,
    Left(u32),
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
