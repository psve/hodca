#[macro_use] 
extern crate structopt;
extern crate tuple_iterator;
extern crate time;
extern crate num;
extern crate num_cpus;

mod tables;
mod correlation_functions;
mod score_functions;
pub mod readers;
pub mod options;

use std::io::{self,Write};
use tables::*;
use readers::Trace;
use options::*;
use score_functions::*;

/// Generate guesses for values that occur in the DCA trace based on the inputs
fn generate_guesses(position: usize, 
                    inputs: &Vec<Vec<u8>>, 
                    guess_type: GuessType) 
                    -> Vec<Vec<u8>> {
    if position > 15 {
        panic!("[ERROR] generate_guesses: position is out of bounds.");
    }

    let mut guesses = vec![vec![0;inputs.len()];256];

    // For each key guess
    for k in 0..256 {
        // For each input
        for i in 0..inputs.len() {
            guesses[k][i] = match guess_type {
                GuessType::Sbox    => S[(inputs[i][position] ^ k as u8) as usize],
                GuessType::Inverse => INV[(inputs[i][position] ^ k as u8) as usize],
            };
        }
    }

    guesses
}

/// Extracts a specific bit position of supplied guesses
fn get_bit_guesses(bit_position: u8, guesses: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    if bit_position > 8 {
        panic!("[ERROR] get_bit_guesses: invalid bit position.")
    }

    let mut bit_guesses = vec![vec![0;guesses[0].len()];256];

    for i in 0..256 {
        for j in 0..guesses[0].len() {
            bit_guesses[i][j] = (guesses[i][j] >> bit_position) & 0x1;
        }
    }

    bit_guesses
}

/// Selects the chosen correlation function and scoring method, and calculates the key scores based
/// on a set of guesses
fn calculate_key_scores(bounds: (usize,usize), 
                        window: usize, 
                        order: usize,
                        correlation_type: CorrelationType, 
                        traces: &Vec<Trace>, 
                        guesses: &Vec<Vec<u8>>) 
                        -> KeyScores {
    match correlation_type {
        CorrelationType::Pearson => {
            pearson_scores(bounds, window, order, traces, guesses)
        },
        CorrelationType::Equality => {
            equality_scores(bounds, window, order, traces, guesses)
        },
        CorrelationType::Likelihood => {
            likelihood_scores(bounds, window, order, traces, guesses)
        },
    }
}

/// Calcuates the key scores for a specific byte position, using the chosen correlation function
/// and scoring method
fn attack_position(position: usize, 
                   bounds: (usize,usize), 
                   window: usize, 
                   order: usize,
                   output_size: usize,
                   correlation_type: CorrelationType, 
                   data_type: DataType, 
                   guess_type: GuessType,
                   traces: &Vec<Trace>, 
                   inputs: &Vec<Vec<u8>>) 
                   -> KeyScores {
    let bounds = match data_type {
        DataType::Bytes => (bounds.0/8, bounds.1/8),
        _               => bounds,
    };

    // We assume that all trace as the same length. This is true if they are
    // generated with tracergrind + bin2daredevil
    if bounds.0 > traces[0].len() || bounds.1 > traces[0].len() {
        panic!("[ERROR] attack_position: start or stop position out of bounds.")
    }

    // It doesn't make sense to consider a window smaller than the order
    if order != 1 && window < order {
        panic!("[ERROR] attack_position: window cannot be smaller than order");
    }

    let guesses = generate_guesses(position, &inputs,guess_type);
    let mut key_scores = [(0.0, 0); 256];

    for i in 0..256 {
        key_scores[i].1 = i;
    }

    match data_type {
        DataType::Bits => {
            // For each bit of the target key byte
            for b in 0..8 {
                print!("\tAttacking bit {}...",b);
                io::stdout().flush().expect("Unable to flush stdout");

                let start = time::precise_time_s();

                // Extract guess values for the current bit position
                let bit_guesses = get_bit_guesses(b, &guesses);

                // Find the correlations for this bit
                let mut bit_scores = calculate_key_scores(bounds, window, order,
                                                          correlation_type,
                                                          &traces, &bit_guesses);

                // Add them to the correlations for the other bits
                for i in 0..256 {
                    key_scores[i].0 += bit_scores[i].0.abs();
                }

                let stop = time::precise_time_s();

                println!(" Done! ({:.4} seconds)", stop - start);

                // Sort bit scores
                bit_scores.sort_by(|x,y| (y.0).abs().partial_cmp(&(x.0).abs()).expect("Could not sort"));

                for i in 0..output_size {
                    println!("\t\t{:02x}, score = {:.4}", bit_scores[i].1, bit_scores[i].0);
                }
            }
        },

        DataType::Bytes => {
            // Attack whole bytes
            print!("\tAttacking all bits...");
            io::stdout().flush().expect("Unable to flush stdout");

            let start = time::precise_time_s();

            // Find the correlations for this byte
            key_scores = calculate_key_scores(bounds, window, order,
                                              correlation_type, 
                                              &traces, &guesses);

            let stop = time::precise_time_s();

            println!(" Done! ({:.4} seconds)", stop - start);
        }
    }
    
    // Sort key scores
    key_scores.sort_by(|x,y| (y.0).abs().partial_cmp(&(x.0).abs()).expect("Could not sort"));

    key_scores
}

/// Calcuates the key scores for a all byte position, using the chosen correlation function 
/// and scoring method, returns the highest scoring key bytes
pub fn attack_all(bounds: (usize,usize), 
                  window: usize, 
                  order: usize, 
                  output_size: usize,
                  correlation_type: CorrelationType, 
                  data_type: DataType, 
                  guess_type: GuessType,
                  traces: &Vec<Trace>, 
                  inputs: &Vec<Vec<u8>>) 
                  -> [usize; 16] {
    let mut full_key = [0;16];

    for k in 0..16 {
        println!("\nAttacking key byte {}...", k);

        let start = time::precise_time_s();
        let key_scores = attack_position(k, bounds, window, order, output_size,
                                         correlation_type, data_type, guess_type,
                                         &traces,&inputs);
        let stop = time::precise_time_s();

        println!("\nFinished attacking key byte {} in {:.4} seconds.", k, stop-start);

        for i in 0..output_size {
            println!("\t{:02x}, score = {:.4}", key_scores[i].1, key_scores[i].0);
        }
            
        println!("");
        println!("\tLowest score: {:.4}", key_scores[255].0);
        println!("\tHighest score: {:.4}", key_scores[0].0);

        full_key[k] = key_scores[0].1;
    }

    full_key
}