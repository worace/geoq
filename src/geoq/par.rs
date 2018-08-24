extern crate deque;
extern crate rand;

// use self::deque::{Stealer, Stolen};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver, RecvError};

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

    let mut input_channels: Vec<SyncSender<Input>> = vec![];
    let mut threads: Vec<JoinHandle<_>> = vec![];
    let mut output_channels: Vec<Receiver<Output>> = vec![];
    (0..num_workers).for_each(|i| {
        println!("Start worker {}", i);
        let (input_sender, input_receiver) = sync_channel(50);
        let (output_sender, output_receiver) = sync_channel(50);

        let t = thread::spawn(move|| {
            loop {
                let work = input_receiver.recv();
                match work {
                    Err(RecvError) => continue,
                    Ok(Input::Item(i)) => {
                        output_sender.send(Output::Item((i, rand::random()))).unwrap();
                    }
                    Ok(Input::Done) => {
                        output_sender.send(Output::Done).unwrap();
                        break;
                    }
                }
            }
        });

        input_channels.push(input_sender);
        output_channels.push(output_receiver);
        threads.push(t);
    });

    for i in 0..20 {
        println!("enqueue to worker {}", i % num_workers);
        input_channels[i % num_workers].send(Input::Item(i)).unwrap();
    }
    (0..num_workers).for_each(|i| input_channels[i].send(Input::Done).unwrap() );

    println!("{:?}", threads);
    println!("{:?}", output_channels);
    // let last_received = 0;
    // SPSC approach

    // Workers 1,2,3
    // SPSC 1 Reader ----> Worker 1
    // SPSC 2 Reader ----> Worker 2
    // SPSC 3 Reader ----> Worker 3

    // SPSC 4 Worker 1 ----> Printer
    // SPSC 5 Worker 2 ----> Printer
    // SPSC 6 Worker 3 ----> Printer

    // Reader
    // Enqueue lines in order round-robin 1,2,3,1,2,3
    // Worker
    // Push to owned output queue as done
    // Printer
    // Round-robin blocking read from worker output queues

    while !output_channels.is_empty() {
        println!("Printer loop");
        for i in 0..output_channels.len() {
            println!("Printer check channel {}", i);
            let output = output_channels[i].recv();
            match output {
                Err(RecvError) => continue,
                Ok(Output::Item((i, f))) => {
                    println!("output received ({}, {})", i, f);
                },
                Ok(Output::Done) => {
                    println!("Received done from {}", i);
                    output_channels.remove(i);
                    println!("removed channel, num outputs is {}", output_channels.len());
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
