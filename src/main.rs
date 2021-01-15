use std::{sync::{Arc, Mutex, mpsc::sync_channel}, thread, thread::JoinHandle, time::Duration, mem::drop, collections::VecDeque};
use rand::Rng;

type ITEM = usize;

//fn consumer(buffer: &Arc<Mutex<VecDeque<ITEM>>>, nrof_items: usize) -> JoinHandle<()> {
    //thread::spawn(move || {
        //rsleep(100);
        //println!("Consumer");
    //})
//}

fn main() {
    const NROF_PRODUCERS: usize = 2;
    const NROF_ITEMS: usize = 20;
    const BUFFER_SIZE: usize = 5;

    let counter = Arc::new(Mutex::new(0));
    let jobs = Arc::new(Mutex::new(vec![false; NROF_ITEMS + 1]));
    let job_mutex = Arc::new(Mutex::new(0));

    //let buffer: Arc<Mutex<VecDeque<ITEM>>> = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_SIZE)));
    let (sender, receiver) = sync_channel(BUFFER_SIZE);

    let mut producers: Vec<JoinHandle<()>> = Vec::with_capacity(NROF_PRODUCERS);

    for i in 0..NROF_PRODUCERS {
        println!("producer {}", i);

        let counter = Arc::clone(&counter);
        let jobs = Arc::clone(&jobs);
        let job_mutex = Arc::clone(&job_mutex);

        let sender_clone = sender.clone();

        producers.push({
            thread::spawn(move || {
                loop {
                    let mut found: ITEM;

                    let guard = job_mutex.lock().unwrap();
                    let mut count = counter.lock().unwrap();
                    let mut jobs_done = jobs.lock().unwrap();

                    *count += 1;

                    if *count > NROF_ITEMS {
                        found = NROF_ITEMS;
                    } else {
                        if *count < NROF_PRODUCERS {
                            let base: usize = 2;
                            let random = rand::thread_rng().gen_range(0..(base.pow(31) - 1));
                            found = (random % (2 * NROF_PRODUCERS)) % NROF_ITEMS;
                        }  else {
                            found = *count - NROF_PRODUCERS;

                            if jobs_done[found] {
                                let base: usize = 2;
                                let random = rand::thread_rng().gen_range(0..(base.pow(31) - 1));
                                found = (*count + (random % NROF_PRODUCERS)) % NROF_ITEMS;
                            }
                        }

                        if jobs_done[found] {
                            found = 0;
                            while jobs_done[found] {
                                found += 1;
                            }
                        }
                    }

                    jobs_done[found] = true;

                    let new_item = found;

                    if new_item == NROF_ITEMS {
                        break;
                    }

                    println!("producer {} item {}", i, new_item);
                    sender_clone.send(new_item).unwrap();

                    drop(count);
                    drop(jobs_done);
                    drop(guard);

                    rsleep(100);
                }
            })
        });
    }

    //let consumer = consumer(&buffer, NROF_ITEMS);
    let mut count = 0;

    let consumer = thread::spawn(move || {
        rsleep(100);

        while count < NROF_ITEMS {
            let item = receiver.recv().unwrap();
            println!("consumer {}", item);
            count += 1;
        }
    });

    for producer in producers {
        match producer.join() {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }

    match consumer.join() {
        Ok(_) => (),
        Err(err) => println!("{:?}", err),
    }
}

fn rsleep(max: usize) {
    let random_time = rand::thread_rng().gen_range(0..max) as u64;
    thread::sleep(Duration::from_millis(random_time));
}

//fn get_next_item(counter: &Arc<Mutex<ITEM>>, jobs: &Arc<Mutex<Vec<bool>>>, nrof_items: usize, nrof_producers: usize) -> ITEM {
////fn get_next_item(count: &usize, jobs_done: &Vec<bool>, nrof_items: usize, nrof_producers: usize) -> ITEM {
    //let mut found: ITEM;

    //let mut count = counter.lock().unwrap();
    //let mut jobs_done = jobs.lock().unwrap();

    //*count += 1;

    //if *count > nrof_items {
        //found = nrof_items;
    //} else {
        //if *count < nrof_producers {
            //let base: usize = 2;
            //let random = rand::thread_rng().gen_range(0..(base.pow(31) - 1));
            //found = (random % (2 * nrof_producers)) % nrof_items;
        //}  else {
            //found = *count - nrof_producers;

            //if jobs_done[found] {
                //let base: usize = 2;
                //let random = rand::thread_rng().gen_range(0..(base.pow(31) - 1));
                //found = (*count + (random % nrof_producers)) % nrof_items;
            //}
        //}

        //if jobs_done[found] {
            //found = 0;
            //while jobs_done[found] {
                //found += 1;
            //}
        //}
    //}

    //jobs_done[found] = true;

    //found
//}
