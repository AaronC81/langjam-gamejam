mod ast;
pub use ast::*;

mod interpreter;
pub use interpreter::*;

mod parser;
pub use parser::*;

#[cfg(test)]
mod test;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
