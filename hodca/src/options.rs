use structopt::StructOpt;
use std::fmt;

#[derive(Copy,Clone)]
pub enum GuessType {
    Sbox,
    Inverse,
}

impl fmt::Debug for GuessType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuessType::Sbox    => write!(f, "S-box"),
            GuessType::Inverse => write!(f, "Inverse"),
        }
    }
}

#[derive(Copy,Clone)]
pub enum CorrelationType {
    Pearson,
    Equality,
    Likelihood,
}

impl fmt::Debug for CorrelationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CorrelationType::Pearson    => write!(f, "Pearson"),
            CorrelationType::Equality   => write!(f, "Equality"),
            CorrelationType::Likelihood => write!(f, "Likelihood"),
        }
    }
}

#[derive(Copy,Clone)]
pub enum DataType {
    Bits,
    Bytes,
}

impl fmt::Debug for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Bits  => write!(f, "Bits"),
            DataType::Bytes => write!(f, "Bytes"),
        }
    }
}

#[derive(StructOpt)]
#[structopt(name = "Higher Order DCA", about = "Apply higher order DCA to traces.")]
pub struct InputArgs {
    #[structopt(long = "path")]
    /**
    Path to trace files. The files read are <path>.trace and <path>.input.
    */
    pub path: String,

    #[structopt(long = "traces")]
    /**
    The number of traces in the input files.
    */
    pub traces: usize,

    #[structopt(long = "length")]
    /**
    The length of each trace in the input file.
    */
    pub length: usize,

    #[structopt(long = "order")]
    /**
    The order of the attack.
    */
    pub order: usize,

    #[structopt(long = "start")]
    /**
    Position in each trace to start analysis. Defaults to zero.
    */
    pub start: Option<usize>,

    #[structopt(long = "stop")]
    /**
    Position in each trace to stop analysis. Defaults to trace length.
    */
    pub stop: Option<usize>,

    #[structopt(long = "window")]
    /**
    Size of the window to use. 
    */
    pub window: usize,

    #[structopt(long = "output_size")]
    /**
    The number of output correlations to display for each position.
    */
    pub output_size: Option<usize>,

    #[structopt(long = "correlation")]
    /**
    The type of correlation to use. Valid inputs: pearson, equality, likelihood.
    */
    pub correlation: String,

    #[structopt(long = "data_type")]
    /**
    The data type to use. Valid inputs: bits, bytes.
    */
    pub data_type: String,

    #[structopt(long = "guess")]
    /**
    Type of guess to use as target. Valid inputs: sbox, inverse.
    */
    pub guess: String,
}