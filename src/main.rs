use std::{sync::{Arc, Mutex, mpsc::sync_channel, Condvar}, thread, thread::JoinHandle, time::Duration, mem::drop};
use rand::Rng;

type ITEM = usize;

fn main() {
    const NROF_PRODUCERS: usize = 20;
    const NROF_ITEMS: usize = 2000;
    const BUFFER_SIZE: usize = 5;

    let counter = Arc::new(Mutex::new(0));
    let jobs = Arc::new(Mutex::new(vec![false; NROF_ITEMS + 1]));

    let (sender, receiver) = sync_channel(BUFFER_SIZE);

    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let mut producers: Vec<JoinHandle<()>> = Vec::with_capacity(NROF_PRODUCERS);

    for i in 0..NROF_PRODUCERS {
        let counter = Arc::clone(&counter);
        let jobs = Arc::clone(&jobs);

        let sender_clone = sender.clone();
        let pair_clone = Arc::clone(&pair);

        producers.push({
            thread::spawn(move || {
                loop {
                    let new_item: ITEM = {
                        let mut found: ITEM;

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

                        drop(count);
                        drop(jobs_done);

                        found
                    };

                    if new_item == NROF_ITEMS {
                        break;
                    }

                    let (lock, cvar) = &*pair_clone;

                    let guard = cvar.wait_while(lock.lock().unwrap(), |next_item| {
                        *next_item != new_item
                    }).unwrap();

                    drop(guard);

                    let mut next_item = lock.lock().unwrap();
                    *next_item += 1;
                    cvar.notify_all();

                    sender_clone.send(new_item).unwrap();

                    rsleep(100);
                }
            })
        });
    }

    let mut count = 0;

    let consumer = thread::spawn(move || {
        rsleep(100);

        while count < NROF_ITEMS {
            let item = receiver.recv().unwrap();
            println!("count{}: {}", count, item);
            if count != item {
                panic!("Mismatch!");
            }
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
    thread::sleep(Duration::from_micros(random_time));
}
