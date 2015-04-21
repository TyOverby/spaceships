extern crate lux;
extern crate spaceships;
extern crate bincode;
extern crate wire;

use wire::udp;
use lux::game::*;
use lux::prelude::*;
use spaceships::*;
use spaceships::ServerToClientMessage as S2C;
use spaceships::ClientToServerMessage as C2S;
use std::collections::HashMap;
use std::net::{ToSocketAddrs, SocketAddr};

struct SpaceshipGame {
    ships: HashMap<u16, Spaceship>,
    mine: Option<u16>,
    sender: udp::Sender<C2S>,
    recvr: udp::Receiver<(SocketAddr, MessageCarrier<'static>)>,
    server_addr: SocketAddr
}

impl SpaceshipGame {
    fn new<A, B: Clone>(socket: A, server_addr: B) -> SpaceshipGame
    where A: ToSocketAddrs, B: ToSocketAddrs {
        let (sender, receiver) = udp::bind(socket).unwrap();
        sender.send(&C2S::Hello, server_addr.clone()).ok()
              .expect("expected sending HELLO to server to work.");
        SpaceshipGame {
            ships: HashMap::new(),
            mine: None,
            sender: sender,
            recvr: receiver,
            server_addr: server_addr.to_socket_addrs().unwrap().next().unwrap()
        }
    }

    fn my_ship_mut(&mut self) -> Option<&mut Spaceship> {
        self.mine.and_then(move |id| self.ships.get_mut(&id))
    }

    fn process_update(&mut self, message: S2C) {
        match message {
            S2C::AssignSpaceship(id) => {
                self.mine = Some(id);
            }
            S2C::UpdateSpaceship(ship) => {
                if Some(ship.id) != self.mine {
                    self.ships.insert(ship.id, ship);
                }
            }
            S2C::AddSpaceship(ship) => {
                self.ships.insert(ship.id, ship);
            }
            S2C::RemoveSpaceship(id) => {
                self.ships.remove(&id);
            }
            S2C::Goodbye => {
                // do nothing for now
            }
        }
    }

    fn consume_update(&mut self, carrier: MessageCarrier<'static>) {
        let general = carrier.general.take();
        let specific = carrier.specific.map(|a| a.take());
        let specific = specific.into_iter().flat_map(|a| a.into_iter());
        let all = general.into_iter().chain(specific);
        for message in all {
            self.process_update(message);
        }
    }
}

impl Game for SpaceshipGame {
    fn update(&mut self, _dt: f32, window: &mut Window, _events: &mut EventIterator) {
        let from_server = self.recvr.iter().filter_map(|(from, m)| {
            if from == self.server_addr { Some(m) } else { None }
        }).collect::<Vec<_>>();

        for message in from_server {
            self.consume_update(message);
        }

        let sender = self.sender.clone();
        let addr = self.server_addr.clone();
        if let Some(ship) = self.my_ship_mut() {
            let mut dirty = false;
            let mouse = window.mouse_pos();

            if window.mouse_pos() != ship.position {
                ship.position = mouse;
                dirty = true;
            }

            if window.is_key_pressed('a') {
                ship.rotation += 0.05;
                dirty = true;
            }

            if window.is_key_pressed('d') {
                ship.rotation -= 0.05;
                dirty = true;
            }

            if dirty {
                sender.send(&C2S::UpdateSpaceship(*ship), addr).ok()
                      .expect("Expected sending to work.");
            }
        }
    }

    fn render(&mut self, _lag: f32, _window: &mut Window, frame: &mut Frame) {
        for (_, ship) in &self.ships {
            let (x, y) = ship.position;
            frame.rect(x, y, 50.0, 50.0)
                 .rotate_around((26.0, 25.0), ship.rotation)
                 .fill_color(ship.color).fill();
        }
    }
}

fn main(){
    let client_port: u16 = std::env::args().nth(1).and_then(|a| a.parse().ok()).expect("Expected client port");
    let client_addr = ("localhost", client_port);
    let server_addr = ("localhost", 1234u16);

    let game = SpaceshipGame::new(client_addr, server_addr);
    game.run_until_end();
}
