extern crate spaceships;
extern crate wire;
extern crate bincode;
extern crate rustc_serialize;
extern crate clock_ticks;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;
use spaceships::*;
use spaceships::ServerToClientMessage as S2C;
use spaceships::ClientToServerMessage as C2S;

struct GameState {
    addr_to_id: HashMap<SocketAddr, u16>,
    id_to_ship: HashMap<u16, Spaceship>,
    next_id: u16
}

impl GameState {
    fn new() -> GameState {
        GameState {
            addr_to_id: HashMap::new(),
            id_to_ship: HashMap::new(),
            next_id: 0
        }
    }

    fn ship_from_addr(&self, addr: &SocketAddr) -> Option<&Spaceship> {
        self.addr_to_id.get(addr).and_then(|id| self.id_to_ship.get(id))
    }

    fn ship_from_addr_mut(&mut self, addr: &SocketAddr) -> Option<&mut Spaceship> {
        match self.addr_to_id.get(addr) {
            Some(id) => self.id_to_ship.get_mut(id),
            None => None
        }
    }
}

fn main() {
    let mut state = GameState::new();
    let (sender, receiver) = wire::udp::bind::<MessageCarrier, C2S, _>(("localhost", 1234)).unwrap();

    println!("waiting on port 1234 ...");

    loop {
        let mut updates = vec![];
        let mut specific = HashMap::new();
        let mut before_updates = clock_ticks::precise_time_ms();

        // Listen for messages from our clients.
        for (from, message) in receiver.iter() {
            match message {
                C2S::Hello => {
                    println!("got hello from {}", from);
                    if state.addr_to_id.contains_key(&from){
                        continue;
                    }

                    state.next_id += 1;
                    let id = state.next_id;
                    let spaceship = Spaceship {
                        id: id,
                        color: (255, 0, 0),
                        position: (0.0, 0.0),
                        velocity: (0.0, 0.0),
                        rotation: 0.0
                    };
                    state.addr_to_id.insert(from, id);
                    state.id_to_ship.insert(id, spaceship);

                    specific.entry(from)
                            .or_insert_with(|| vec![])
                            .push(S2C::AssignSpaceship(id));
                }
                C2S::Goodbye => {
                    if let Some(id) = state.addr_to_id.remove(&from) {
                        state.id_to_ship.remove(&id);
                        updates.push(S2C::RemoveSpaceship(id));
                    }
                }
                C2S::UpdateSpaceship(ship) => {
                    if let Some(s_ship) = state.ship_from_addr_mut(&from) {
                        *s_ship = ship;
                        updates.push(S2C::UpdateSpaceship(ship));
                    }
                }
            }
        }

        // Update ship positions.
        for (_, ship) in state.id_to_ship.iter_mut() {
            ship.position.0 += ship.velocity.0;
            ship.position.1 += ship.velocity.1;
            updates.push(S2C::UpdateSpaceship(*ship));
        }

        for (to, _) in state.addr_to_id.iter() {
            let carrier = MessageCarrier {
                general: bincode::RefBox::new(&updates),
                specific: specific.get(&to).map(|s| bincode::RefBox::new(s))
            };
            sender.send(&carrier, to);
        }

        thread::sleep_ms((16 - (clock_ticks::precise_time_ms() - before_updates)) as u32);
    }
}
