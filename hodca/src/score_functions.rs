use num_cpus;
use std::thread;
use std::sync::mpsc;
use num::Float;
use tuple_iterator::{TupleIterator,WindowedTupleIterator};
use readers::Trace;
use correlation_functions::*;

/// Type for holding the score and key value
pub type KeyScores = [(f64, usize); 256];

/// Calculates pearson scores for a trace
pub fn pearson_scores(bounds: (usize,usize), 
                      window: usize, 
                      order: usize,
                      traces: &Vec<Trace>, 
                      guesses: &Vec<Vec<u8>>) 
                      -> KeyScores {
    // We assume that all guesses as the same length. This is true if they are
    // generated using generate_guesses
    let guess_len = guesses[0].len();

    // Calculate auxilliary information about the bit guesses to speed up correlation calculations
    let mut aux_values = [(0.0,0.0);256];

    for i in 0..256 {
        aux_values[i] = get_auxilliary_values(&guesses[i]);
    }

    let num_threads = num_cpus::get();
    let (result_tx,result_rx) = mpsc::channel();

    for t in 0..num_threads {
        let (result_tx, aux_values, guesses,traces) = 
            (result_tx.clone(), aux_values,guesses.clone(), traces.clone());

        thread::spawn(move || {
            let mut correlations = [(0.0,0);256];

            // Iterate over all tuples time points, but skipping num_threads each time
            // and having an offset of the current thread index
            let range_size = bounds.1 - bounds.0;
            let time_tuples: Vec<Vec<usize>>;

            if order == 1 {
                time_tuples = TupleIterator::new(order, range_size)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            } else {
                time_tuples = WindowedTupleIterator::new(order, range_size, window)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            }

            for tuple in time_tuples {
                let mut ho_trace =  vec![0;guess_len];
                
                for i in 0..guess_len {
                    for time_point in &tuple {
                        ho_trace[i] ^= traces[i][bounds.0 + *time_point];
                    }
                }

                // Get auxilliary information about the second order trace
                let (s1,s2) = get_auxilliary_values(&ho_trace);

                // Calculate correlation of second order trace for each guess
                for i in 0..256 {
                    let (s3,s4) = (aux_values[i].0,aux_values[i].1);
                    let c = double_assisted_pearson(&ho_trace,&guesses[i],
                                                 s1,s2,s3,s4);

                    // Save guess if larger than current
                    if c.abs() > correlations[i].0.abs() {
                        correlations[i] = (c,i);
                    }
                }
            }

            result_tx.send(correlations).expect("Thread could not send result");
        });
    }

    let mut key_scores = [(0.0,0);256];

    for _ in 0..num_threads {
        let thread_result = result_rx.recv().expect("Main could not receive result");

        // Update current best correlations
        for i in 0..256 {
            if thread_result[i].0.abs() > key_scores[i].0.abs() {
                key_scores[i] = thread_result[i];
            }
        }
    }

    key_scores
}

/// Calculates equality scores for a trace
pub fn equality_scores(bounds: (usize,usize), 
                       window: usize, 
                       order: usize,
                       traces: &Vec<Trace>, 
                       guesses: &Vec<Vec<u8>>) 
                       -> KeyScores {
    // We assume that all guesses as the same length. This is true if they are
    // generated using generate_guesses
    let guess_len = guesses[0].len();

    let num_threads = num_cpus::get();
    let (result_tx,result_rx) = mpsc::channel();

    for t in 0..num_threads {
        let (result_tx, guesses, traces) = (result_tx.clone(), guesses.clone(), traces.clone());

        thread::spawn(move || {
            let mut counters = [(0.0,0);256];

            // Iterate over all tuples time points, but skipping num_threads each time
            // and having an offset of the current thread index
            let range_size = bounds.1 - bounds.0;
            let time_tuples: Vec<Vec<usize>>;

            if order == 1 {
                time_tuples = TupleIterator::new(order, range_size)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            } else {
                time_tuples = WindowedTupleIterator::new(order, range_size,window)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            }

            for tuple in time_tuples {
                let mut ho_trace =  vec![0;guess_len];
                
                for i in 0..guess_len {
                    for time_point in &tuple {
                        ho_trace[i] ^= traces[i][bounds.0 + *time_point];
                    }
                }

                // Calculate correlation of second order trace for each guess
                for i in 0..256 {
                    let c = equality_correlation(&ho_trace, &guesses[i]);

                    // Save guess if larger than current
                    if c.abs() > counters[i].0.abs() {
                        counters[i] = (c,i);
                    }
                }
            }

            result_tx.send(counters).expect("Thread could not send result");
        });
    }

    let mut key_scores = [(0.0,0);256];

    for _ in 0..num_threads {
        let thread_result = result_rx.recv().expect("Main could not receive result");

        // Update current best counter
        for i in 0..256 {
            if thread_result[i].0.abs() > key_scores[i].0.abs() {
                key_scores[i] = thread_result[i];
            }
        }
    }

    key_scores
}

/// Calculates likelihood scores for a trace.
pub fn likelihood_scores(bounds: (usize,usize), 
                         window: usize, 
                         order: usize,
                         traces: &Vec<Trace>, 
                         guesses: &Vec<Vec<u8>>) 
                         -> KeyScores {
    // We assume that all guesses as the same length. This is true if they are
    // generated using generate_guesses
    let guess_len = guesses[0].len();

    // Prepare for threading
    let num_threads = num_cpus::get();
    let (result_tx,result_rx) = mpsc::channel();

    for t in 0..num_threads {
        let (result_tx,guesses,traces) = (result_tx.clone(),guesses.clone(),traces.clone());

        thread::spawn(move || {
            let mut counters = vec![vec![0;guess_len];256];

            // Iterate over all tuples time points, but skipping num_threads each time
            // and having an offset of the current thread index
            let range_size = bounds.1 - bounds.0;
            let time_tuples: Vec<Vec<usize>>;
            
            if order == 1 {
                time_tuples = TupleIterator::new(order, range_size)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            } else {
                time_tuples = WindowedTupleIterator::new(order, range_size,window)
                             .filter(|x| x.iter().fold(0, |acc, &x| acc+x) % num_threads == t)
                             .collect();
            } 

            for tuple in time_tuples {
                let mut ho_trace = vec![0;guess_len];
                
                for i in 0..guess_len {
                    for time_point in &tuple {
                        ho_trace[i] ^= traces[i][bounds.0 + *time_point];
                    }
                }

                // Calculate correlation of second order trace for each guess
                for i in 0..256 {
                    add_loglikelihood_counters(&ho_trace,&guesses[i],&mut counters[i]);
                }
            }
            
            result_tx.send(counters).expect("Thread could not send result");
        });
    }    
    
    let mut counters = vec![vec![0;guess_len];256];
    let mut key_scores = [(0.0,0);256];

    for _ in 0..num_threads {
        let thread_result = result_rx.recv().expect("Main could not receive result");

        // Add result to counters
        for i in 0..256 {
            for j in 0..guess_len {
                counters[i][j] += thread_result[i][j];
            }
        }
    }

    // Calculate correlations
    for i in 0..256 {
        let likelihood = loglikelihood_correlation(&counters[i]);
        key_scores[i] = (likelihood,i);
    }

    key_scores
}
