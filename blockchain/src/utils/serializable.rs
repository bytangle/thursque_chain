pub trait Serializable <T> {
  fn serialize(&self) -> Vec<u8>;
  fn deserialize(bytes: Vec<u8>) -> T;
}