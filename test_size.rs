extern crate console; use console::Term; fn main() { let (h, w) = Term::stdout().size(); println!("h={}, w={}", h, w); }
