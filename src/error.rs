use core::convert::Infallible;

#[derive(Debug)]
pub enum Error<E = ()> {
    /// Communication error
    Comm(E),
    /// Pin setting error
    Pin(Infallible),
}
