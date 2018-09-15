extern crate deque;
extern crate rand;

// use self::deque::{Stealer, Stolen};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver, RecvError};
use std::io::BufRead;
use geoq::error::Error;
use geoq::reader;
use geoq::input;
use geoq::entity::{self, Entity};
use num_cpus;

enum WorkerInput {
    Item(String),
    Done
}

enum WorkerOutput {
    Item(Result<Vec<String>, Error>),
    Done
}

enum Output {
    Item((usize, f32)),
    Done
}

enum Input {
    Item(usize),
    Done
}

pub struct LineReader<'a> {
    reader: &'a mut BufRead
}

impl<'a> LineReader<'a> {
    pub fn new(reader: &'a mut BufRead) -> LineReader<'a> {
        LineReader{reader}
    }
}

impl<'a> Iterator for LineReader<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        reader::read_line(self.reader)
    }
}

fn handle_line<F>(line: String, handler: F) -> Result<(), Error>
where F: Fn(Entity) -> Result<(), Error>
{
    let input = try!(input::read_line(line));
    let entities = try!(entity::from_input(input));
    for e in entities {
        try!(handler(e));
    }
    Ok(())
}

const WORKER_BUF_SIZE: usize = 100;
pub fn for_entity_par<'a, F: 'static>(input: &'a mut BufRead, handler: F) -> Result<(), Error>
where F: Send + Sync + Fn(Entity) -> Result<Vec<String>, Error>
{
    let num_workers = num_cpus::get();
    let mut input_channels: Vec<SyncSender<WorkerInput>> = vec![];
    let mut threads: Vec<JoinHandle<_>> = vec![];
    let mut output_channels: Vec<Receiver<WorkerOutput>> = vec![];
    let handler_arc = Arc::new(handler);

    (0..num_workers).for_each(|_| {
        let (input_sender, input_receiver) = sync_channel(WORKER_BUF_SIZE);
        let (output_sender, output_receiver) = sync_channel(WORKER_BUF_SIZE);

        let handler = handler_arc.clone();

        let t = thread::spawn(move|| {
            loop {
                match input_receiver.recv() {
                    Err(RecvError) => continue,
                    Ok(WorkerInput::Item(line)) => {
                        // TODO figure out how to make this work with arc
                        // output_sender.send(WorkerOutput::Item(handle_line(line, *handler)));

                        let mut results = Vec::new();

                        match input::read_line(line) {
                            Err(e) => eprintln!("{:?}", e),
                            Ok(input) => {
                                match entity::from_input(input) {
                                    Err(e) => eprintln!("{:?}", e),
                                    Ok(entities) => {
                                        for e in entities {
                                            match handler(e) {
                                                Err(e) => eprintln!("{:?}", e),
                                                Ok(lines) => results.extend(lines)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        output_sender.send(WorkerOutput::Item(Ok(results))).unwrap();
                    }
                    Ok(WorkerInput::Done) => {
                        output_sender.send(WorkerOutput::Done).unwrap();
                        break;
                    }
                }
            }
        });

        input_channels.push(input_sender);
        output_channels.push(output_receiver);
        threads.push(t);
    });

    let printer_thread = thread::spawn(move|| {
        while !output_channels.is_empty() {
            for i in 0..output_channels.len() {
                let output = output_channels[i].recv();
                match output {
                    Err(RecvError) => continue,
                    Ok(WorkerOutput::Item(Ok(lines))) => {
                        for l in lines {
                            println!("{}", l);
                        }
                    },
                    Ok(WorkerOutput::Item(Err(e))) => eprintln!("{:?}", e),
                    Ok(WorkerOutput::Done) => {
                        output_channels.remove(i);
                        break;
                    }
                }
            }
        }
    });

    let reader = LineReader::new(input);
    for (i, line) in reader.enumerate() {
        input_channels[i % num_workers].send(WorkerInput::Item(line)).unwrap();
    }
    (0..num_workers).for_each(|i| input_channels[i].send(WorkerInput::Done).unwrap());

    printer_thread.join().expect("Couldn't wait for printer thread to complete");

    Ok(())
}

pub fn example<F>(handler: F) -> Vec<(usize, f32)>
where F: 'static + Sync + Send + Fn(usize) -> f32
{
    let num_workers = 4;

    let mut input_channels: Vec<SyncSender<Input>> = vec![];
    let mut threads: Vec<JoinHandle<_>> = vec![];
    let mut output_channels: Vec<Receiver<Output>> = vec![];
    let handler_arc = Arc::new(handler);

    (0..num_workers).for_each(|i| {
        println!("Start worker {}", i);
        let (input_sender, input_receiver) = sync_channel(50);
        let (output_sender, output_receiver) = sync_channel(50);

        let handler = handler_arc.clone();
        let t = thread::spawn(move|| {
            loop {
                let work = input_receiver.recv();
                match work {
                    Err(RecvError) => continue,
                    Ok(Input::Item(i)) => {
                        let res = handler(i);
                        let output = (i, res);
                        let output_item = Output::Item(output);
                        output_sender.send(output_item).unwrap();
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
    extern crate rand;
    use geoq::par::{example, for_entity_par};
    use geoq::entity;
    #[test]
    #[ignore]
    fn test_example() {
        let keys: Vec<usize> = example(|i| rand::random()).iter().map(|p| p.0 ).collect();
        let exp: Vec<usize> = (1..20).collect();
        assert_eq!(exp, keys);
    }

    #[test]
    fn test_par_entities() {

        // Problem:
        // Outputs need to be processed by the single printer
        // round-robin to preserve ordering
        // But each input can potentially produce
        // many outputs
        // So outputs need to be Result(Vec<String>, Error)
        // and printer has to round-robin and then print all
        // outputs from that batch before continuing
        let mut input = r#"34.2277,-118.2623
{"type":"Polygon","coordinates":[[[-117.87231445312499,34.77997173591062],[-117.69653320312499,34.77997173591062],[-117.69653320312499,34.90170042871546],[-117.87231445312499,34.90170042871546],[-117.87231445312499,34.77997173591062]]]}
{"type":"Polygon","coordinates":[[[-118.27880859375001,34.522398580663314],[-117.89154052734375,34.522398580663314],[-117.89154052734375,34.649025753526985],[-118.27880859375001,34.649025753526985],[-118.27880859375001,34.522398580663314]]]}
"#.as_bytes();

        // let mut input = "9q5\n9q4".as_bytes();
        let res = for_entity_par(&mut input, move |entity| {
            Ok(vec![format!("handling entity {}", entity).to_owned()])
        });
        println!("***");
        println!("Res: {:?}", res);
    }
}
