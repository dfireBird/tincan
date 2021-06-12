use std::convert::TryInto;

pub trait Serializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(data: Vec<u8>) -> Self;
}

#[derive(Debug)]
struct Message(String);

impl Serializable for Message {
    fn to_bytes(&self) -> Vec<u8> {
        let messsage = self.0.as_bytes().to_vec();
        let mut data = Vec::from("chat".as_bytes());
        data.append(&mut messsage.len().to_le_bytes().to_vec());
        data.append(&mut messsage.to_vec());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Self {
        Self(String::from_utf8(data[8..].to_vec()).unwrap())
    }
}

#[derive(Debug)]
struct Handshake(u32);

impl Serializable for Handshake {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::from("Hello".as_bytes());
        data.append(&mut self.0.to_le_bytes().to_vec());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Self {
        Self(u32::from_le_bytes(data[5..].try_into().unwrap()))
    }
}

#[derive(Debug)]
struct File {
    name: String,
    data: Vec<u8>,
}

impl Serializable for File {
    fn to_bytes(&self) -> Vec<u8> {
        let file_name = self.name.as_bytes();
        let mut padded_file_name = vec![0u8; 96 - file_name.len()];
        padded_file_name.append(&mut file_name.to_vec());

        let mut data = Vec::from("file".as_bytes());
        data.append(&mut (96 + self.data.len()).to_le_bytes().to_vec());
        data.append(&mut file_name.to_vec());
        data.append(&mut self.data.clone());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data: data[105..].to_vec(),
            name: String::from_utf8(data[8..104].to_vec()).unwrap(),
        }
    }
}
