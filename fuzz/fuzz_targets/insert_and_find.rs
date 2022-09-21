#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (Vec<(String, i32)>, String, Option<bool>)| {
    let mut tree = path_tree::PathTree::new();

    for (path, num) in &data.0 {
        tree.insert(path, num);
    }

    match data.2 {
        None => {
            let _ = tree.find(&data.1);
        }
        Some(b) => {
            if !b {
                let _ = tree.find(&data.1);
            }
        }
    }
});
