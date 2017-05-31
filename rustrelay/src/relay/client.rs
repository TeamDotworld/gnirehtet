use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use mio::net::TcpStream;
use mio::{Event, PollOpt, Ready};

use super::selector::{EventHandler, Selector};
use super::ipv4_packet_buffer::IPv4PacketBuffer;

const TAG: &'static str = "Client";

pub struct Client {
    id: u32,
    stream: TcpStream,
    client_to_network: IPv4PacketBuffer,
    dead: bool,
}

impl Client {
    pub fn new(id: u32, selector: &mut Selector, stream: TcpStream) -> io::Result<Rc<RefCell<Self>>> {
        let rc = Rc::new(RefCell::new(Self {
            id: id,
            stream: stream,
            client_to_network: IPv4PacketBuffer::new(),
            dead: true,
        }));
        let rc_clone = rc.clone();
        let handler = Box::new(move |selector: &mut Selector, ready| {
            let mut self_ref = rc_clone.borrow_mut();
            self_ref.on_ready(selector, ready);
        });
        // on start, we are interested only in writing (we must first send the client id)
        selector.register(&rc.borrow().stream, handler, Ready::writable(), PollOpt::level())?;
        Ok(rc)
    }

    fn kill(&mut self) {
        self.dead = true;
        // TODO unregister from Selector
    }

    fn process_send(&mut self) {
        
    }

    fn process_receive(&mut self) {
        match self.read() {
            Ok(_) => {}
            Err(_) => {
                error!(target: TAG, "Cannot read");
                self.kill();
            }
        }
    }

    fn update_interests(&mut self, selector: &mut Selector) {

    }

    fn read(&mut self) -> io::Result<()> {
        self.client_to_network.read_from(&mut self.stream)
    }
}

impl EventHandler for Client {
    fn on_ready(&mut self, selector: &mut Selector, event: Event) {
        assert!(!self.dead);
        let ready = event.readiness();
        if ready.is_writable() {
            self.process_send();
        }
        if !self.dead && ready.is_readable() {
            self.process_receive();
        }
        if !self.dead {
            self.update_interests(selector);
        }
    }
}
