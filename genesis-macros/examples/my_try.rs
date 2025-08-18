use anyhow::{Result, format_err};

fn main() -> Result<()> {
    // let x = f3(f2(f1("123")?)?)?;
    let x = my_try!(f3(my_try!(f2(my_try!(f1("123"))))));
    println!("{x}");
    Ok(())
}

fn f1(s: impl AsRef<str>) -> Result<String> {
    Ok(format!("f1 {}", s.as_ref()))
}

fn f2(s: impl AsRef<str>) -> Result<String> {
    Ok(format!("f2 {}", s.as_ref()))
}

fn f3(s: impl AsRef<str>) -> Result<String> {
    Err(format_err!("f3 {}", s.as_ref()))
}

#[macro_export]
macro_rules! my_try {
    ($ex:expr) => {
        match $ex {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        }
    };
}
