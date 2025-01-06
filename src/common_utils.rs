use num_derive::FromPrimitive;
use num::FromPrimitive;
#[derive(FromPrimitive, PartialEq)]
// pub enum Commands
// {
//     None = 0x00000000,
//     //Common commands
//     //0x0001,

//     LoadDone = 0x00010001,
//     GetConfig = 0x00010002,
    
//     //Modules and services administration 
//     //0x0002
//     AdaptersStart = 0x00020010,
//     AdapterStop = 0x00020011,
//     ControlCenterStart = 0x00020020,
//     CredentialStart = 0x00020030,
//     DatabankStart = 0x00020040,
//     MessageStorageStart = 0x00020050,
//     PortalStart = 0x00020060,
//     NodeStart = 0x00020070,
//     ProcessStart = 0x00020080

// }

pub enum Commands
{
    None,
    //Common commands
    //0x0001,

    LoadDone,
    GetAdaptersConfig,
    GetNodeConfig,
    GetProcessessConfig,
    GetMessageStorageConfig,
    
    
    //Modules and services administration 
    //0x0002
    AdaptersStart,
    AdapterStop,
    ControlCenterStart,
    CredentialStart,
    DatabankStart,
    MessageStorageStart,
    PortalStart,
    NodeStart,
    ProcessStart

}

impl Commands
{
    pub fn get_command(code: i32)->Commands
    {
        let result = FromPrimitive::from_i32(code);
        match  result
        {
            Some(res)=>res,
            None => Commands::None
        }
    }
}