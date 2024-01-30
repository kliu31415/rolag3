pub struct TokenBucket {
    capacity: f64,
    regen: f64,
    last_used: f64,
    last_token_count: f64,
}

impl TokenBucket {
    pub fn new(capacity: f64, regen: f64) -> Self {
        TokenBucket {
            capacity,
            regen,
            last_used: 0.0,
            last_token_count: capacity,
        }
    }

    // Attempt to take up to some amount of tokens from the token bucket. Returns the number of tokens actually taken.
    pub fn try_take(&mut self, cur_time: f64, tokens: f64) -> f64 {
        assert!(tokens >= 0.0);
        assert!(cur_time >= self.last_used);
        let cur_token_count = f64::min(self.capacity, self.last_token_count + (cur_time - self.last_used) * self.regen);
        self.last_used = cur_time;
        if cur_token_count >= tokens {
            // there are sufficient tokens to satisfy the entire request
            self.last_token_count = cur_token_count - tokens;
            return tokens;
        } else {
            // there aren't enough tokens to satisfy the entire request. Take all the tokens.
            self.last_token_count = 0.0;
            return cur_token_count;
        }
    }

    // Attempt to take up some amount of tokens from the token bucket. fok = "Fill or Kill", i.e. either take the
    // whole requested amount or none
    pub fn try_take_fok(&mut self, cur_time: f64, tokens: f64) -> bool {
        assert!(tokens >= 0.0);
        assert!(cur_time >= self.last_used);
        let cur_token_count = f64::min(self.capacity, self.last_token_count + (cur_time - self.last_used) * self.regen);
        if cur_token_count >= tokens {
            // there are sufficient tokens to satisfy the entire request
            self.last_used = cur_time;
            self.last_token_count = cur_token_count - tokens;
            true
        } else {
            false
        }
    }

    pub fn take_all(&mut self, cur_time: f64) -> f64 {
        return self.try_take(cur_time, f64::MAX);
    }
}
