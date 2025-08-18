use genesis_macros::AutoDebug;

fn main() {
    let s = TestAAA {
        aaa: "hello world".to_string(),
        bb: 2,
        cc: "hello world2".to_string(),
    };
    println!("{s:?}")
}

#[derive(AutoDebug)]
struct TestAAA {
    pub bb: i32,
    pub aaa: String,
    #[debug(skip)]
    #[allow(dead_code)]
    pub cc: String,
}
