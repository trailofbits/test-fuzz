#[cfg(test)]
mod test {
    #[test_fuzz::test_fuzz]
    pub fn target() {}

    #[test]
    fn test() {
        target();
    }
}
