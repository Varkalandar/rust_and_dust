use std::thread;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::Receiver;

use crate::rand::Rng;

pub struct RngReceiver {
    receiver: Receiver<f32>
}

impl RngReceiver
{
    pub fn new(receiver: Receiver<f32>) -> RngReceiver
    {
        RngReceiver {
            receiver
        }
    }

    pub fn random(&self) -> f32
    {
        self.receiver.recv().unwrap()
    }

    pub fn random_irange(&self, min: i32, max: i32) -> i32
    {
        let r = self.random();
        let range = max - min;

        min + (range as f32 * r) as i32
    }

    pub fn random_urange(&self, min: usize, max: usize) -> usize
    {
        let r = self.random();
        let range = max - min;

        min + (range as f32 * r) as usize
    }

    pub fn random_frange(&self, min: f32, max: f32) -> f32
    {
        let r = self.random();
        let range = max - min;

        min + range * r
    }
}


pub struct RngSource {
    sender: SyncSender<f32>
}


impl RngSource
{
    pub fn new(sender: SyncSender<f32>) -> RngSource
    {
        RngSource {
            sender
        }
    }

    pub fn start(&self)
    {
        let local_sender = self.sender.clone();
        thread::spawn(move || 
            {
                let mut rng = rand::rng();
                
                loop {
                    let result = local_sender.send(rng.random::<f32>());
                
                    if result.is_err() {
                        let error = result.err().unwrap();
                        panic!("RngSource::start(): Error in sending number: {:?}", error);
                    }
                }
            }
        );
    }
}