#[cfg(test)]
mod test {
    use test_fuzz::*;

    #[test_fuzz]
    pub fn target() {}

    #[test]
    fn test() {
        target();
    }
}
