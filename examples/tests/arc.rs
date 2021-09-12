mod arc {
    use std::{
        collections::HashSet,
        thread::{current, spawn},
    };

    #[test]
    fn test() {
        let mut set = HashSet::new();
        for i in 0..32 {
            set.insert(i);
        }

        let arc = std::sync::Arc::new(set);

        for i in 0..32 {
            let arc = arc.clone();
            spawn(move || {
                target(arc, i);
            });
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(arc: std::sync::Arc<HashSet<u32>>, i: usize) {
        println!("{:?}: {:?}", current().id(), arc.iter().nth(i));
    }
}
