use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{

    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    let mut thread_num_limit = num_threads;
    // only one threads, deal data derictly is enough.
    if num_threads < 1 || input_vec.len() <= 1 {
        thread_num_limit = 0;
    }
    if thread_num_limit > input_vec.len() {
        thread_num_limit = input_vec.len();
    }
    if thread_num_limit <= 1 {
        while let Some(raw) = input_vec.pop() {
            output_vec[input_vec.len()] = f(raw);
        }
        return output_vec;
    }
    // TODO: implement parallel map!
    let (raw_sender, raw_reciver) = crossbeam_channel::unbounded();
    let (out_sender, out_reciver) = crossbeam_channel::unbounded();
    let mut threads = vec![];
    for _ in 0..thread_num_limit {
        let in_r = raw_reciver.clone();
        let out_s = out_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok((raw, index)) = in_r.recv() {
                out_s.send((f(raw), index)).expect("error when send result");
            }
        }));
    }
    drop(out_sender);

    while let Some(raw) = input_vec.pop() {
        raw_sender.send((raw, input_vec.len())).expect("error when send raw data");
    }
    drop(raw_sender);

    let mut unordered_data: Vec<Option<U>> = Vec::with_capacity(output_vec.capacity());
    let mut index_bucket = vec![0; output_vec.capacity()];
    while let Ok((data, index)) = out_reciver.recv() {
        unordered_data.push(Some(data));
        index_bucket[index] = unordered_data.len() - 1;
    }
    
    // maybe the loop here to join all threads is not necessary,
    // because finish out_reciver while-loop means that 
    // all threads are dead due to 'drop(out_sender)'.
    for handle in threads {
        handle.join().expect("something wrong in child");
    }

    for i in 0..output_vec.capacity() {
        output_vec.push(unordered_data[index_bucket[i]].take().unwrap());
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
