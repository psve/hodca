use std::iter::FromIterator;

/* A struct that describes m-tuples of elements from a range
 *
 * range        The lower (inclusive) and upper (exclusive) bound of the range
 * current      The current m-tuple from the range
 * done         Indicates if the last tuple was reached
 * p            Auxiliary value used for Chase's Twiddle algorithm
 * x            Auxiliary value used for Chase's Twiddle algorithm
 * y            Auxiliary value used for Chase's Twiddle algorithm
 * z            Auxiliary value used for Chase's Twiddle algorithm
 */
pub struct TupleIterator {
    pub range: (usize,usize),
    pub current: Vec<usize>,
    done: bool,
    p: Vec<i64>,
    x: usize,
    y: usize,
    z: usize,
}

impl TupleIterator {
    /* Creates a new TupleIterator
     *
     * m    The size of the tuples
     * n    The upper (exlusive) bound on the range
     */
    pub fn new(m: usize, n: usize) -> TupleIterator {
        if m > n {
            panic!("[ERROR] m cannot be greater than n.");
        }

        // Initialize values needed for Chase's Twiddle algorithm
        let mut p = vec![0;n+2];
        p[0] = (n+1) as i64;
        p[n+1] = -2;

        for i in (n-m+1)..=n {
            p[i] = (i+m-n) as i64;
        }

        // The first tuple is the last m elements of the range
        let current: Vec<usize> = Vec::from_iter((n-m)..n);

        TupleIterator {
            range: (0,n),
            current,
            done: false,
            p,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

/* Implement Iterator for the TupleIterator struct */
impl Iterator for TupleIterator {
    type Item = Vec<usize>;

    /* Generates the next tuple of distinct values in the range. 
     * This is an implementation of Chase's Twiddle algorithm (Algorithm 382)
     */
    fn next(&mut self) -> Option<Vec<usize>> {
        if self.done {
            return None;
        }

        // Save the current tuple
        let result = self.current.clone();

        // Generate next tuple
        let mut j = 1;

        while self.p[j] <= 0 {
            j += 1;
        }

        if self.p[j-1] == 0 {
            for i in 2..j {
                self.p[i] = -1;
            }

            self.p[j] = 0; 
            self.p[1] = 1;
            self.x = 0;
            self.z = 0; 
            self.y = j-1;
        } else {
            if j > 1 {
                self.p[j-1] = 0;
            }

            loop {
                j += 1;

                if self.p[j] <= 0 {
                    break;
                }
            }

            let (mut i, k) = (j,j-1);

            while self.p[i] == 0 {
                self.p[i] = -1;
                i += 1;
            }

            if self.p[i] == -1 {
                self.p[i] = self.p[k];
                self.z = (self.p[k]-1) as usize;
                self.x = i-1;
                self.y = k-1;
                self.p[k] = -1;
            } else if i == (self.p[0] as usize) {
                   self.done = true;
            } else {
                self.p[j] = self.p[i];
                self.z = (self.p[i]-1) as usize;
                self.p[i] = 0;
                self.x = j-1;
                self.y = i-1;
            }
        }

        self.current[self.z] = self.x;

        // Return the original tuple
        Some(result)
    }
}

/* A struct that describes m-tuples of elements from a range, where the distance between the
 * elements of the tuple is bounded
 *
 * range            The lower (inclusive) and upper (exclusive) bound of the range
 * current          The current m-tuple from the range
 * m                The size of the tuple
 * window_size      The bound on the distance of tuple elements
 * current_index    Current start index of the m-tuple, i.e. the value of the first element
 * tuple            Auxilliary tuple
 */
pub struct WindowedTupleIterator {
    pub range: (usize,usize),
    pub current: Vec<usize>,
    m: usize,
    window_size: usize,
    current_index: usize,
    tuple: TupleIterator,
}

impl WindowedTupleIterator {
    /* Creates a new WindowedTupleIterator
     *
     * m        The size of the tuples
     * n        The upper (exlusive) bound on the range
     * window   Bound on the distance of tuple elements
     */ 
    pub fn new(m: usize, n: usize, window: usize) -> WindowedTupleIterator {
        if m > window {
            panic!("[ERROR] m cannot be larger than the window.");
        }

        if window > n {
            panic!("[ERROR] window cannot be larger than n.");   
        }

        // Create an (m-1)-tuple over a range the size of the window minus one
        // We use this to create the bounded tuples
        let tuple = TupleIterator::new(m-1,window-1); 

        WindowedTupleIterator {
            range: (0,n),
            current: vec![0;m],
            m,
            window_size: window,
            current_index: 0,
            tuple,
        }
    }

    /* Returns the number of bounded tuples in the iterator */
    pub fn len(&self) -> usize{
        (self.window_size-1)*(self.range.1-self.window_size+1) 
           + (self.window_size-1)*(self.window_size-2)/2
    }
}

/* Implements Iterator for the WindowedTupleIterator struct */
impl Iterator for WindowedTupleIterator {
    type Item = Vec<usize>;

    /* Generates the next bounded tuple of distinct values in the range. */
    fn next(&mut self) -> Option<Vec<usize>> {
        if self.current_index + self.m > self.range.1 {
            return None;
        }

        // Try to get the next (m-1)-tuple from the range [0,window-1]
        match self.tuple.next() {
            Some(t) => {
                // Generate a new bounded tuple by offsetting tuple by current_index
                for (i,x) in t.iter().enumerate() {
                    self.current[i+1] = x+self.current_index+1;
                }
            },
            None => {
                // No more tuples starting at current_index. Move to next start value
                self.current_index += 1;
                self.current[0] = self.current_index;
                let tmp = self.range.1 - self.current_index - 1;

                // Not enough space for a full tuple, end iterator
                if tmp <= 0 {
                    return None;
                }

                // Reset the auxilliary tuple
                self.tuple = TupleIterator::new(std::cmp::min(self.m-1,tmp),
                                                std::cmp::min(self.window_size-1,tmp));
                let t = self.tuple.next().expect("Something went wrong");

                // Create first tuple with new start value
                for (i,x) in t.iter().enumerate() {
                    self.current[i+1] = x+self.current_index+1;
                }  
            }
        }

        Some(self.current.clone())
    }
}