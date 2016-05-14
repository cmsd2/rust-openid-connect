pub trait TokenRepo where Self: Send + Sync  {
    
}

pub struct InMemoryTokenRepo;

impl InMemoryTokenRepo {
    pub fn new() -> InMemoryTokenRepo {
        InMemoryTokenRepo
    }
}

impl TokenRepo for InMemoryTokenRepo {
    
}