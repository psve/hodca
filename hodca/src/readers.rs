use std::fs::{self, File};
use std::io::{self, Read, Write};
use options::DataType;

/// A struct representing a progress bar for progress printing on the command line.
struct ProgressBar {
    current_items: f64,
    item_size: f64,
    used: bool,
}

impl ProgressBar {
    /// Creates a new progress for tracking progress of `num_items` steps.
    pub fn new(num_items: usize) -> ProgressBar {
        let item_size = 100.0 / (num_items as f64);

        ProgressBar {
            current_items: 0.0,
            item_size,
            used: false,
        }
    }

    /// Increment the current progress of the bar. The progress bar prints if a new step was
    /// reached.
    #[inline(always)]
    pub fn increment(&mut self) {
        self.current_items += self.item_size;

        while self.current_items >= 1.0 {
            print!("=");
            io::stdout().flush().expect("Could not flush stdout");
            self.current_items -= 1.0;
        }

        self.used = true;
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.used {
            println!();
        }
    }
}

/// A trace is a vector of bytes
pub type Trace = Vec<u8>;

/// Reads a DCA trace from file.
pub fn read_traces(trace_path: &str, 
                   num_traces: usize, 
                   length: usize, 
                   data_type: DataType) 
                   -> Vec<Trace> {
    let file = File::open(trace_path).expect("Could not open file.");
    let metadata = fs::metadata(trace_path).expect("Could not get metadata.");

    if metadata.len() < (num_traces*length) as u64 {
        panic!("[ERROR] read_traces: trace file is not the correct size.");
    }

    let serialized = match data_type {
        DataType::Bits => true,
        DataType::Bytes => false,
    };
    
    let (mut current_trace, mut current_sample) = (0,0);
    
    let mut traces = if serialized {
        vec![vec![0;length];num_traces]
    } else {
        vec![vec![0;length/8];num_traces]
    };

    let total_bytes = length * num_traces;
    let mut progress_bar = ProgressBar::new(total_bytes);

    let mut current_bit = if serialized { 8 } else { 0 };

    println!("Reading trace file...");

    for byte in file.bytes() {
        if current_trace == num_traces {
            break;
        }

        if serialized {
            traces[current_trace][current_sample] = byte.expect("Could not get byte.");
        } else {
            traces[current_trace][current_sample] ^= byte.expect("Could not get byte.") << current_bit;
            current_bit += 1;
        }

        if current_bit == 8 {
            current_sample += 1;
            
            if !serialized {
                current_bit = 0;
            }

            if current_sample == traces[current_trace].len() {
                current_sample = 0;
                current_trace += 1;
            }
        }

        progress_bar.increment();
    }

    traces
}

/// Reads inputs associated with a DCA trace from file.
pub fn read_inputs(input_path: &str, num_inputs: usize) -> Vec<Vec<u8>> {
    let file = File::open(input_path).expect("Could not open file.");
    let metadata = fs::metadata(input_path).expect("Could not get metadata.");

    if metadata.len() < (num_inputs as u64)*16 {
        panic!("[ERROR] read_inputs: input file is not the correct size.");
    }

    let (mut current_input, mut current_byte) = (0,0);
    let mut inputs = vec![vec![0;16];num_inputs];

    for byte in file.bytes() {
        if current_input == num_inputs {
            break;
        }

        // First byte read is the MSB
        inputs[current_input][15-current_byte] = byte.expect("Could not get byte.");

        current_byte += 1;

        if current_byte == 16 {
            current_byte = 0;
            current_input += 1;
        }
    }

    inputs
}