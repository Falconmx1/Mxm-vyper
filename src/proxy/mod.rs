use rand::seq::SliceRandom;

pub struct ProxyRotator {
    proxies: Vec<String>,
    current: usize,
}

impl ProxyRotator {
    pub fn new(proxy_list: Vec<String>) -> Self {
        Self {
            proxies: proxy_list,
            current: 0,
        }
    }
    
    pub fn next(&mut self) -> Option<&String> {
        if self.proxies.is_empty() {
            None
        } else {
            self.current = (self.current + 1) % self.proxies.len();
            Some(&self.proxies[self.current])
        }
    }
    
    pub fn random(&mut self) -> Option<&String> {
        self.proxies.choose(&mut rand::thread_rng())
    }
}
