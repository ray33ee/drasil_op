use futures::prelude::*;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use serde::{Serialize, Deserialize};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::net::SocketAddr;
use x25519_dalek::{EphemeralSecret, PublicKey};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug)]
enum RelayType {
    Extend{public_x: [u8; 32], ip: SocketAddr},
    Extended{public_y: [u8; 32], hash: [u8; 32]},
    Begin{ addr: String },
    Connected,
    Data,
}

#[derive(Serialize, Deserialize, Debug)]
enum CellType {
    Create{public_x: [u8; 32]},
    Created{public_y: [u8; 32], hash: [u8; 32]},
    Relay{recognised: u32, stream_id: u32, digest: u32, data: RelayType, padding: Vec<u8>},
}

#[derive(Serialize, Deserialize, Debug)]
struct Cell {
    hop_id: u32,
    data: CellType,
}

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let socket = TcpStream::connect("127.0.0.1:65432").await.unwrap();

    // Delimit frames using a length header
    let length_delimited = Framed::new(socket, LengthDelimitedCodec::new());

    // Serialize frames with JSON
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalBincode::<Cell>::default());

    //Setup the DH scheme
    let client_secret = EphemeralSecret::new(rand::rngs::OsRng);
    let client_public = PublicKey::from(&client_secret);

    // Send the create cell
    serialized
        .send(Cell { hop_id: 0, data: CellType::Create {public_x: client_public.to_bytes()} })
        .await
        .unwrap();

    // Wait for the created response
    while let Some(msg) = serialized.try_next().await.unwrap() {


        if let CellType::Created {public_y, hash} = msg.data {

            // Calculate the shared secret
            let onion_secret = client_secret.diffie_hellman(&PublicKey::from(public_y)).to_bytes();

            //Calculate thee hash of the shared secret
            let mut hasher = Sha256::new();

            hasher.update(&onion_secret);

            let ga = hasher.finalize();

            let calculated_hash = ga.as_slice();

            println!("Hash verify: {}", calculated_hash == &hash);

            println!("Shared: {:?}", onion_secret);

        } else {

        }

        return
    }
}