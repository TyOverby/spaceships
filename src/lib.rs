extern crate rustc_serialize;
extern crate bincode;

#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct Spaceship {
    pub id: u16,
    pub color: (u8, u8, u8),
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub rotation: f32
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerToClientMessage {
    AssignSpaceship(u16),
    UpdateSpaceship(Spaceship),
    AddSpaceship(Spaceship),
    RemoveSpaceship(u16),
    Goodbye,
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientToServerMessage {
    Hello,
    UpdateSpaceship(Spaceship),
    Goodbye,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct MessageCarrier<'a> {
    pub general: bincode::RefBox<'a, Vec<ServerToClientMessage>>,
    pub specific: Option<bincode::RefBox<'a, Vec<ServerToClientMessage>>>,
}
