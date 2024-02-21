pub trait Joinable<T, E>{
    fn get_ref(&mut self) -> (&T, &mut E);
}