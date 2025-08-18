fn main() {
    let v1: Vec<i32> = my_vec![];
    println!("{v1:?}");
    let v1 = my_vec![1;4];
    println!("{v1:?}");
    let v1 = my_vec![1, 2, 3];
    println!("{v1:?}");
}

#[macro_export]
macro_rules! my_vec {
    ()=>{
        Vec::new()
    };
    // my_vec![1;4] 生成4个1的vec
    ($elem:expr; $n:expr)=>{std::vec::from_elem($elem, $n)};
    // 匹配多个数据, + 号表示至少一个，*号表示0到任意多个, $(,)? 表示结尾可以多有一个逗号
    ($($x:expr),+ $(,)?) => {
        {
            // let mut v = Vec::new();
            // $(
            //    v.push($x);
            // )*
            // v
            <[_]>::into_vec(Box::new([$($x),*]))
        }
    }
}
