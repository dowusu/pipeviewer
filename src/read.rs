use std::{
    fs::File,
    io::{self, BufReader, Read, Result},
};

use crossbeam::channel::Sender;

use crate::CHUNK_SIZE;

pub fn read_loop(infile: &str, stats_tx: Sender<usize>, write_tx: Sender<Vec<u8>>) -> Result<()> {
    let mut reader: Box<dyn Read> = if !infile.is_empty() {
        Box::new(BufReader::new(File::open(infile)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let mut buffer = [0; CHUNK_SIZE];

    loop {
        let num_read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_e) => break,
        };

        //todo: send this buffer to the stats thread (should be number read )
        // if stats_tx.send(num_read).is_err() {
        //     break;
        // }
        let _ = stats_tx.send(num_read);

        if write_tx.send(Vec::from(&buffer[..num_read])).is_err() {
            break;
        }
    }

    let _ = stats_tx.send(0);
    let _ = write_tx.send(Vec::new());

    Ok(())
}
