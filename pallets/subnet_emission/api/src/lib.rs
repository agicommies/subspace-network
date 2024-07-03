#![no_std]

pub trait SubnetEmissionApi {
    fn get_lowest_emission_netuid() -> Option<u16>;

    fn remove_subnet_emission_storage(netuid: u16);

    fn set_subnet_emission_storage(netuid: u16, emission: u64);

    fn create_yuma_subnet(netuid: u16);

    fn remove_yuma_subnet(netuid: u16);

    fn can_remove_subnet(netuid: u16) -> bool;
}
