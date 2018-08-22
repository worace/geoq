extern crate deque;
extern crate rand;

use self::deque::{Stealer, Stolen};
use std::thread::{self, JoinHandle};

enum Output {
    Item((usize, f32)),
    Done
}

enum Input {
    Item(usize),
    Done
}

fn it() -> Vec<(usize, f32)> {
    let num_workers = 4;
    let (worker, stealer) = deque::new();
    (1..20).for_each(|i| worker.push(Input::Item(i)));
    (1..num_workers).for_each(|i| worker.push(Input::Done));

    let mut threads: Vec<JoinHandle<_>> = vec![];
    let mut queues: Vec<Stealer<Output>> = vec![];
    (1..num_workers).for_each(|_| {
        let stealer = stealer.clone();
        let (worker_writer, worker_output) = deque::new();
        let t = thread::spawn(move|| {
            loop {
                let work = stealer.steal();
                match work {
                    Stolen::Empty => continue,
                    Stolen::Abort => continue,
                    Stolen::Data(Input::Item(i)) => {
                        worker_writer.push(Output::Item((i, rand::random())));
                    }
                    Stolen::Data(Input::Done) => {
                        worker_writer.push(Output::Done);
                        break;
                    }
                }
            }
        });
        queues.push(worker_output);
        threads.push(t);
    });

    println!("{:?}", threads);
    println!("{:?}", queues);
    // let last_received = 0;
    while !queues.is_empty() {
        for i in (0..queues.len()) {
            let stolen = queues[i].steal();
            match stolen {
                Stolen::Empty => continue,
                Stolen::Abort => continue,
                Stolen::Data(Output::Item((i, f))) => {
                    println!("output received ({}, {})", i, f);
                }
                Stolen::Data(Output::Done) => {
                    queues.remove(i);
                    break;
                }
            }
        }
    }

    (1..20).map(|i| (i, rand::random())).collect()
}

#[cfg(test)]
mod tests {
    use geoq::par::it;
    #[test]
    fn test_it() {
        let keys: Vec<usize> = it().iter().map(|p| p.0 ).collect();
        let exp: Vec<usize> = (1..20).collect();
        assert_eq!(exp, keys);
    }
}
