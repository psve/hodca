extern crate structopt;
extern crate hodca;
extern crate time;

use hodca::readers::{read_traces,read_inputs};
use hodca::options::{InputArgs, GuessType, CorrelationType, DataType};
use structopt::StructOpt;
use std::cmp;

fn main() {
    let options = InputArgs::from_args();

    // Parse options
    let path = options.path;
    let num_traces = options.traces;
    let trace_length = options.length;
    let order = options.order;
    let bounds = (options.start.unwrap_or(0), options.stop.unwrap_or(trace_length));
    let bounds = (cmp::min(bounds.0, trace_length), cmp::min(bounds.1, trace_length));

    if bounds.0 > bounds.1 {
        println!("Start index is larger than stop index.");
        return;
    }

    let window = options.window;
    let output_size = options.output_size.unwrap_or(10);
    let correlation_type = match options.correlation.as_ref() {
        "pearson"    => CorrelationType::Pearson,
        "equality"   => CorrelationType::Equality,
        "likelihood" => CorrelationType::Likelihood,
        _ => {
            println!("{:?} is not a valid correlation type.", options.correlation);
            return;
        }
    };
    let data_type = match options.data_type.as_ref() {
        "bits"  => DataType::Bits,
        "bytes" => DataType::Bytes,
        _ => {
            println!("{:?} is not a valid data type.", options.correlation);
            return;
        }
    };
    let guess_type = match options.guess.as_ref() {
        "sbox"    => GuessType::Sbox,
        "inverse" => GuessType::Inverse,
        _ => {
            println!("{:?} is not a valid guess type.", options.correlation);
            return;
        }
    };

    
    // Print attack info
    println!("#############################");
    println!("Order: {}", order);
    println!("Traces: {}", num_traces);
    println!("Analysis indices: {} -> {}", bounds.0, bounds.1);
    println!("Window size: {}", window);
    println!("Correlation: {:?}", correlation_type);
    println!("Data type: {:?}", data_type);
    println!("Target: {:?}", guess_type);
    println!("#############################\n");

    
    // Read data
    let trace_path = &(path.to_owned() + ".trace");
    let input_path = &(path.to_owned() + ".input");
    
    let start = time::precise_time_ns();
    let traces = read_traces(trace_path, num_traces, trace_length, data_type);
    let stop = time::precise_time_ns();
    println!("Read trace file in {:.4} seconds.",(stop-start) as f64 / 1000000000.0);

    let start = time::precise_time_ns();
    let inputs = read_inputs(input_path, num_traces);
    let stop = time::precise_time_ns();
    println!("Read input file in {:.4} seconds.",(stop-start) as f64 / 1000000000.0);

    
    // Start the attack
    let start = time::precise_time_ns();
    let full_key = hodca::attack_all(bounds, window, order, output_size,
                                     correlation_type, data_type, guess_type,
                                     &traces, &inputs);
    let stop = time::precise_time_ns();

    println!("\nAttacked all keys in {} seconds.",(stop-start) as f64 / 1000000000.0 );
    println!("Most likely key:");

    for k in &full_key {
      print!("{:02x}",k);
    }
    println!("");
}