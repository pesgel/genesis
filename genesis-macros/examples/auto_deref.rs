use genesis_macros::AutoDeref;

fn main() {
    let s = TestAAA {
        aaa: "hello world".to_string(),
        bb: 2,
    };

    println!("{s:?}")
}

#[derive(Debug, AutoDeref)]
#[deref(mutable = true, field = "aaa")]
struct TestAAA {
    #[allow(dead_code)]
    pub bb: i32,
    pub aaa: String,
}
