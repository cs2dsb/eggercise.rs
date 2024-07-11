use super::{Builder, PeerId, RoomId};
pub trait Client<R: RoomId, P: PeerId>: Sized {
    type Builder: Builder<Self>;
    fn new() -> Self::Builder;
}
