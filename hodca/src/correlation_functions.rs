use num::ToPrimitive;

/// Calculates the correlation between x and y by counting the number of positions where they are
/// equal
pub fn equality_correlation<T: ToPrimitive>(x: &Vec<T>, y: &Vec<T>) -> f64 {
    if x.len() != y.len() {
        panic!("[ERROR] equality_correlation: x and y must have same length.");
    }

    let mut eq = 0;

    for i in 0..x.len() {
        if x[i].to_u64().unwrap() == y[i].to_u64().unwrap() {
            eq +=1;
        }
    } 

    eq as f64
}

/// Calculatates auxilliary values (sum of elements and sum of squard elements) of a vector used for
/// fast calculation of Pearson correlation
pub fn get_auxilliary_values<T: ToPrimitive>(x: &Vec<T>) -> (f64,f64) {
    let (mut u, mut v) = (0.0,0.0);

    for i in 0..x.len() {
        u += x[i].to_f64().unwrap();
        v += (x[i].to_f64().unwrap())*(x[i].to_f64().unwrap());
    }

    (u,v)
}

/// Calculates the Pearson correlation between two vectors, using auxilliary information about both
/// vectors
pub fn double_assisted_pearson<T: ToPrimitive>(
        x: &Vec<T>, 
        y: &Vec<T>, 
        s1: f64, 
        s2: f64, 
        s3: f64, 
        s4: f64) 
        -> f64 {
    if x.len() != y.len() {
        panic!("[ERROR] double_assisted_pearson: x and y must have same length.");
    }

    let (mut s5, n) = (0.0,x.len() as f64);

    for i in 0..x.len() {
        s5 += (x[i].to_f64().unwrap())*(y[i].to_f64().unwrap());
    }

    let corr = (n * s5 - s1*s3) / ((n*s2 - s1*s1).sqrt() * (n*s4 - s3*s3).sqrt());

    corr
}

/// Adds the number of times x and y are equal to an existing counter. Can be used to parallelize
/// calculation of log-likelihood scores for multiple guesses
pub fn add_loglikelihood_counters<T: ToPrimitive>(
        x: &Vec<T>, 
        y: &Vec<T>, 
        counters: &mut Vec<u64>) {
    if x.len() != y.len() {
        panic!("[ERROR] add_loglikelihood_counters: x and y must have same length.");
    }

    if x.len() != counters.len() {
        panic!("[ERROR] add_loglikelihood_counters: x and counters must have same length.");
    }

    for i in 0..x.len() {
        if x[i].to_u64().unwrap() == y[i].to_u64().unwrap() {
            counters[i] += 1;
        }
    }
}

/// Calculates the log-likelihood score from a vector of counters
pub fn loglikelihood_correlation(counters: &Vec<u64>) -> f64 {
    let mut log_likelihood = 0.0;

    for &x in counters {
        log_likelihood += (x as f64).ln();
    }

    return log_likelihood
}