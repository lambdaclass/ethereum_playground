mod raw_transaction;
mod raw_ping_packet;
use ethereum_types::{H160, H256, U256};
use raw_transaction::RawTransaction;
use raw_ping_packet::{RawPingPacket,Endpoint};
use rlp::Rlp;
use std::net::{UdpSocket, Ipv4Addr};
/// Signs an ethereum payload. This library assumes that the provided payload string
/// is the RLP encoding of the following list:
/// [nonce, gas_price, gas_limit, recipient, value, data, chain_id].
/// The returned signed payload is ready to immediately broadcast to the
/// corresponding chain.
#[rustler::nif]
pub fn sign_transaction(payload_str: String, private_key: String) -> String {
    let payload = hex::decode(payload_str).unwrap();
    let rlp = Rlp::new(&payload);
    let mut iter = rlp.iter();
    let nonce: U256 = iter.next().unwrap().as_val().unwrap();
    let price: U256 = iter.next().unwrap().as_val().unwrap();
    let gas: U256 = iter.next().unwrap().as_val().unwrap();
    let to_iter = iter.next().unwrap();
    let mut to: Option<H160> = None;
    if false == to_iter.is_empty() {
        let to_val: H160 = to_iter.as_val().unwrap();
        to = Some(to_val);
    }

    let value: U256 = iter.next().unwrap().as_val().unwrap();
    let data: Vec<u8> = iter.next().unwrap().as_val().unwrap();
    let chain_id: u64 = iter.next().unwrap().as_val().unwrap();

    let mut pkey_data: [u8; 32] = Default::default();
    pkey_data.copy_from_slice(&hex::decode(private_key).unwrap());
    let pkey = H256(pkey_data);
    let tx = RawTransaction {
        to: to,
        nonce: nonce,
        value: value,
        gas: gas,
        gas_price: price,
        data: data,
    };

    let transaction_prefix = String::from("0x").to_owned();
    let signed_payload = hex::encode(tx.sign(&pkey, &chain_id));

    return transaction_prefix + &signed_payload;
}


#[rustler::nif]
pub fn send_ping() {
    let mut pkey_data: [u8; 32] = Default::default();
    pkey_data.copy_from_slice(&hex::decode("e90d75baafee04b3d9941bd8d76abe799b391aec596515dee11a9bd55f05709c".to_string()).unwrap());
    let pkey = H256(pkey_data);

    let from_addr = Ipv4Addr::new(127, 0, 0, 1);
    let from_addr_u32: u32 = from_addr.into();

    let to_addr = Ipv4Addr::new(127, 0, 0, 1);
    let to_addr_u32: u32 = to_addr.into();

    let raw_ping = RawPingPacket {
        version: 1,
        from: Endpoint{address: from_addr_u32, udp_port: 30303, tcp_port: 0},
        to: Endpoint{address: to_addr_u32, udp_port: 30303, tcp_port: 30303},
        expiration: 1
    };

    let encoded_packet = raw_ping.encode_packet(&pkey);
    println!("{:?}", encoded_packet);
    let socket = UdpSocket::bind("127.0.0.1:30303").expect("couldn't bind to address");
    socket.send_to(&encoded_packet, "127.0.0.1:30303").expect("couldn't send data");
    let mut buf = [0; 1000];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!("{}", number_of_bytes);
    
}

rustler::init!("Elixir.EthClient", [sign_transaction, send_ping]);
