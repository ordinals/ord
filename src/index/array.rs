use super::*;

pub(super) trait Array: Sized {
  type Array;

  fn from_array(array: Self::Array) -> Self;

  fn to_array(self) -> Self::Array;
}

impl Array for InscriptionId {
  type Array = InscriptionIdArray;

  fn from_array(array: Self::Array) -> Self {
    Self {
      txid: Txid::from_inner(array[0..32].try_into().unwrap()),
      vout: u32::from_be_bytes(array[32..36].try_into().unwrap()),
    }
  }

  fn to_array(self) -> Self::Array {
    let mut array = [0; 36];
    array[0..32].copy_from_slice(self.txid.as_inner());
    array[32..36].copy_from_slice(&self.vout.to_be_bytes());
    array
  }
}

impl Array for OutPoint {
  type Array = OutPointArray;

  fn from_array(array: Self::Array) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(array)).unwrap()
  }

  fn to_array(self) -> Self::Array {
    let mut array = [0; 36];
    self.consensus_encode(&mut array.as_mut_slice()).unwrap();
    array
  }
}

impl Array for SatPoint {
  type Array = SatPointArray;

  fn from_array(array: Self::Array) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(array)).unwrap()
  }

  fn to_array(self) -> Self::Array {
    let mut array = [0; 44];
    self.consensus_encode(&mut array.as_mut_slice()).unwrap();
    array
  }
}
