use cosmwasm_std::Addr;

pub trait AddressLike {}

impl AddressLike for String {}
impl AddressLike for Addr {}
