use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let modpacks = vec!["simply", "rc-plus", "2k", "rekindled"];
    let max_concurrent = 4; 
    
    let modpacks_queue = Arc::new(Mutex::new(modpacks.into_iter()));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let mut workers = vec![];

    println!("starting throttled parallel updates ({} at a time)...", max_concurrent);

    for i in 0..max_concurrent {
        let queue_clone = Arc::clone(&modpacks_queue);
        let err_clone = Arc::clone(&errors);

        let handle = thread::spawn(move || {
            loop {
                let pack = {
                    let mut queue = queue_clone.lock().unwrap();
                    queue.next()
                };

                let pack = match pack {
                    Some(p) => p,
                    None => break,
                };

                let path = format!("modpacks/{}", pack);
                println!("[Worker {}] starting: {}", i, pack);

                if !std::path::Path::new(&path).exists() {
                    let mut e = err_clone.lock().unwrap();
                    e.push(format!("directory missing: {}", path));
                    continue;
                }

                let refresh = Command::new("pw")
                    .args(["batch", "refresh", "-y"])
                    .current_dir(&path)
                    .status();

                let update = Command::new("pw")
                    .args(["batch", "update", "-a", "-y"])
                    .current_dir(&path)
                    .status();

                let failed = match (refresh, update) {
                    (Ok(s1), Ok(s2)) => !s1.success() || !s2.success(),
                    (Err(e), _) => {
                        println!("[Worker {}] refresh failed for {}: {}", i, pack, e);
                        true
                    },
                    (_, Err(e)) => {
                        println!("[Worker {}] update failed for {}: {}", i, pack, e);
                        true
                    }
                };

                if failed {
                    let mut e = err_clone.lock().unwrap();
                    e.push(format!("failed: {}", pack));
                } else {
                    println!("[Worker {}] done: {}", i, pack);
                }
            }
        });
        workers.push(handle);
    }

    for handle in workers {
        handle.join().unwrap();
    }

    let final_errors = errors.lock().unwrap();
    if !final_errors.is_empty() {
        eprintln!("\nsummary of failures");
        for err in final_errors.iter() { 
            eprintln!("{}", err); 
        }
        std::process::exit(1); 
    } else {
        println!("\nall updates finished successfully.");
    }
}