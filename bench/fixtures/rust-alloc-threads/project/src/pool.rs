use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::job::{Job, Outcome, Receipt};
use crate::render::render;

/// How many finished jobs each worker keeps for the status page.
const RECENT: usize = 16;

/// A fixed set of worker threads that render jobs taken from one queue.
pub struct Pool {
    sender: Option<mpsc::Sender<Job>>,
    results: mpsc::Receiver<Outcome>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl Pool {
    pub fn start(threads: usize) -> Pool {
        let (sender, jobs) = mpsc::channel::<Job>();
        let (done, results) = mpsc::channel::<Outcome>();
        let jobs = Arc::new(Mutex::new(jobs));
        let workers = (0..threads)
            .map(|_| {
                let jobs = Arc::clone(&jobs);
                let done = done.clone();
                thread::spawn(move || {
                    let mut recent: VecDeque<Box<Receipt>> = VecDeque::new();
                    loop {
                        let next = jobs.lock().unwrap().recv();
                        match next {
                            Ok(job) => {
                                let receipt = render(&job);
                                let outcome = receipt.outcome;
                                recent.push_back(receipt);
                                if recent.len() > RECENT {
                                    recent.pop_front();
                                }
                                if done.send(outcome).is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                })
            })
            .collect();
        Pool { sender: Some(sender), results, workers }
    }

    pub fn submit(&self, job: Job) {
        self.sender.as_ref().expect("pool is running").send(job).expect("workers are running");
    }

    /// Waits for `count` outcomes and returns them in the order they finish.
    pub fn collect(&self, count: usize) -> Vec<Outcome> {
        (0..count).map(|_| self.results.recv().expect("workers are running")).collect()
    }

    pub fn shutdown(mut self) {
        self.sender.take();
        for worker in self.workers.drain(..) {
            worker.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_job_comes_back_once() {
        let pool = Pool::start(3);
        for id in 0..20 {
            pool.submit(Job { id, sections: vec![100 + id as usize, 2000] });
        }
        let mut ids: Vec<u64> = pool.collect(20).iter().map(|o| o.id).collect();
        ids.sort();
        assert_eq!(ids, (0..20).collect::<Vec<u64>>());
        pool.shutdown();
    }

    #[test]
    fn outcomes_match_a_direct_render() {
        let pool = Pool::start(2);
        let job = Job { id: 5, sections: vec![64, 128] };
        pool.submit(job.clone());
        assert_eq!(pool.collect(1)[0], render(&job).outcome);
        pool.shutdown();
    }
}
