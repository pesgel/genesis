//! process

mod process;

#[cfg(test)]
mod tests {
    use vt100::Parser;

    #[test]
    fn test() {
        let xx = "ppwd\u{1b}[D\u{1b}[D\u{7f}";
        println!("{:?}", xx.as_bytes());
        let mut parser = Parser::new(24, 80, 0);
        parser.process(xx.as_bytes());
        println!("{}", parser.screen().contents())
    }
}
