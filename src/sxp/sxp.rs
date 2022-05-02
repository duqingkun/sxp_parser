use std::io::Read;
use std::fs;
use anyhow::Error;

#[allow(unused)]
use crate::{trace, debug, warn, error};

pub struct SxpFrame {
    frame: Vec<u8>,
}

impl SxpFrame {
    pub fn new() -> Self {
        SxpFrame {
            frame: vec![],
        }
    }

    pub fn load_data(&mut self, data: &str) -> Result<(), Error> {
        let bytes_num = data.len() / 2;

        self.frame.clear();

        for i in 0..bytes_num {
            let v = u8::from_str_radix(&data[(i * 2)..=(i * 2 + 1)], 16)?;
            self.frame.push(v);
        }

        Ok(())
    }

    pub fn load_file(&mut self, file: &str) -> Result<(), Error> {
        let mut f = fs::File::open(file)?;
        let mut buffer: [u8; 1] = [0; 1];
        let mut calu_buf: [u8; 2] = [0; 2];
        let mut index = 0;
        let white_char: Vec<u8> = vec![0x20, 0x0A, 0x0D];

        self.frame.clear();

        loop {
            match f.read(&mut buffer) {
                Ok(n) if (n == 1) => {
                    if !white_char.contains(&buffer[0]) {
                        if buffer[0] >= 0x30 && buffer[0] <= 0x39 {
                            buffer[0] -= 0x30;
                        } else if buffer[0] >= 0x61 && buffer[0] <= 0x7a {
                            buffer[0] -= 0x61 - 10;
                        } else if buffer[0] >= 0x41 && buffer[0] <= 0x5a {
                            buffer[0] -= 0x41 - 10;
                        } else {
                            continue;
                        }

                        calu_buf[index] = buffer[0];
                        index += 1;
                        if index == 2 {
                            let v = (calu_buf[0] << 4) + calu_buf[1];
                            self.frame.push(v);
                            index = 0;
                        }
                    }
                },
                _ => break,
            }
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn print_bytes(&self) {
        for b in &self.frame {
            print!("{:02x} ", b);
        }
        println!();
    }
}

impl Default for SxpFrame {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Smp {
    fn parse_smp(&mut self) -> Result<(), Error>;

    fn dword_reverse(bytes: &mut [u8]) {
        let byte_num = bytes.len();
        let dword_num = byte_num / 4;
        for i in 0..dword_num {
            let start = i * 4;
            let b3 = bytes[start];
            let b2 = bytes[start + 1];
            let b1 = bytes[start + 2];
            let b0 = bytes[start + 3];

            bytes[start] = b0;
            bytes[start + 1] = b1;
            bytes[start + 2] = b2;
            bytes[start + 3] = b3;
        }
    }

    fn parse_report_general(&self, bytes: &[u8]) -> Result<(), Error> {
        let mut index = 0;
        let mut num = 1;
        #[allow(unused)]
        let mut resp_dword_num = 0;

        debug!("SMP REPORT GENERAL:");

        while let Some(range) = bytes.get(index..=(index + num)) {
            match index {
                0 => { // smp frame type
                    if range[0] != 0x41 {
                        return Err(anyhow::format_err!("smp frame type should be 0x41"));
                    }
                    index += num;
                    num = 1;
                },
                1 => { // function
                    if range[0] != 0x00 {
                        return Err(anyhow::format_err!("smp report general function should be 0x00"));
                    }
                    index += num;
                    num = 1;
                },
                2 => { // function result
                    trace!("\tFUNCTION RESULT:                      {:02x}", range[0]);
                    index += num;
                    num = 1;
                },
                3 => { // response length
                    resp_dword_num = range[0];
                    if resp_dword_num != 0x00 && resp_dword_num != 0x11 {
                        return Err(anyhow::format_err!("smp report general response length should be 0x00 or 0x11"));
                    }
                    index += num;
                    num = 2;
                },
                4 => { // expander change count
                    let exp_change_cnt: u16 = ((range[0] as u16) << 8) + (range[1] as u16);
                    trace!("\tEXPANDER CHANGE COUNT:                {:02x}", exp_change_cnt);
                    index += num;
                    num = 2;
                },
                6 => { // expander route index
                    let exp_route_idx: u16 = ((range[0] as u16) << 8) + (range[1] as u16);
                    trace!("\tEXPANDER ROUTE INDEXES:               {:02x}", exp_route_idx);
                    index += num;
                    num = 1;
                },
                8 => { // long response
                    let long_resp = range[0] >> 7;
                    trace!("\tLONG RESPONSE:                        {}", long_resp);
                    index += num;
                    num = 1;
                },
                9 => { // number of phys
                    trace!("\tNUMBER OF PHYS:                       {}", range[0]);
                    index += num;
                    num = 1;
                },
                10 => { // 
                    let table_to_table = range[0] >> 7;
                    let zone_config = (range[0] & 0x40) >> 6;
                    let self_config = (range[0] & 0x20) >> 5;
                    let stp_continue_awt = (range[0] & 0x10) >> 4;
                    let orr_supported = (range[0] & 0x08) >> 3;
                    let config_others = (range[0] & 0x04) >> 2;
                    let configuring = (range[0] & 0x02) >> 1;
                    let ext_config_route_table = range[0] & 0x01;
                    trace!("\tTABLE TO TABLE SUPPORTED:             {}", table_to_table);
                    trace!("\tZONE CONFIGURING:                     {}", zone_config);
                    trace!("\tSELF CONFIGURING:                     {}", self_config);
                    trace!("\tSTP CONTINUE AWT:                     {}", stp_continue_awt);
                    trace!("\tOPEN REJECT RETRY SUPPORTED:          {}", orr_supported);
                    trace!("\tCONFIGURES OTHERS:                    {}", config_others);
                    trace!("\tCONFIGURING:                          {}", configuring);
                    trace!("\tEXTERNALLY CONFIGURABLE ROUTE TABLE:  {}", ext_config_route_table);
                    index += num + 1;
                    num = 8;
                },
                12 => { // ecnlosure logical identifier
                    let mut v = [0_u8; 8];
                    v.clone_from_slice(&range[0..8]);
                    let enc_logical_id = u64::from_be_bytes(v);
                    trace!("\tENCLOSURE LOGICAL IDENTIFIER:         0x{:16x}", enc_logical_id);
                    index += num + 10;
                    num = 2;
                },
                30 => { // stp bus inactivity time limit
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let bus_inactivity_time_limit = u16::from_le_bytes(v);
                    trace!("\tSTP BUS INACTIVITY TIME LIMIT:        0x{:16x}", bus_inactivity_time_limit);
                    index += num;
                    num = 2;
                },
                32 => {
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let max_connect_time_limit = u16::from_le_bytes(v);
                    trace!("\tSTP MAXIMUM CONNECT TIME LIMIT:       0x{:16x}", max_connect_time_limit);
                    index += num;
                    num = 2;
                },
                34 => {
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_le_bytes(v);
                    trace!("\tSTP SMP I_T NEXUS LOSS TIME:          0x{:16x}", value);
                    index += num;
                    num = 1;
                },
                36 => { // 
                    let zone_group_num = (range[0] & 0xc0) >> 6;
                    let zone_locked = (range[0] & 0x10) >> 4;
                    let physical_presence_supported = (range[0] & 0x08) >> 3;
                    let physical_presence_asserted = (range[0] & 0x04) >> 2;
                    let zoning_supported = (range[0] & 0x02) >> 1;
                    let zoning_enabled = range[0] & 0x01;
                    trace!("\tNUMBER OF ZONE GROUPS:                {}", zone_group_num);
                    trace!("\tZONE LOCKED:                          {}", zone_locked);
                    trace!("\tPHYSICAL PRESENCE SUPPORTED:          {}", physical_presence_supported);
                    trace!("\tPHYSICAL PRESENCE ASSERTED:           {}", physical_presence_asserted);
                    trace!("\tZONING SUPPORTED:                     {}", zoning_supported);
                    trace!("\tZONING ENABLED:                       {}", zoning_enabled);
                    index += num;
                    num = 1;
                },
                37 => { // 
                    let saving = (range[0] & 0x10) >> 4;
                    let saving_zone_manager_pwd_supported = (range[0] & 0x08) >> 3;
                    let saving_zone_phy_info_supported = (range[0] & 0x04) >> 2;
                    let saving_zone_permisssion_table_supported = (range[0] & 0x02) >> 1;
                    let saving_zoning_enabled_supported = range[0] & 0x01;
                    trace!("\tSAVING:                                   {}", saving);
                    trace!("\tSAVING ZONE MANAGER PASSWORD SUPPORTED:   {}", saving_zone_manager_pwd_supported);
                    trace!("\tSAVING ZONE PHY INFORMATION SUPPORTED:    {}", saving_zone_phy_info_supported);
                    trace!("\tSAVING ZONE PERMISSION TABLE SUPPORTED:   {}", saving_zone_permisssion_table_supported);
                    trace!("\tSAVING ZONING ENABLED SUPPORTED:          {}", saving_zoning_enabled_supported);
                    index += num;
                    num = 2;
                },
                38 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_le_bytes(v);
                    trace!("\tMAXIMUM NUMBER OF ROUTED SAS ADDRESSES:   0x{:04x}", value);
                    index += num;
                    num = 8;
                },
                40 => { // 
                    let mut v = [0_u8; 8];
                    v.clone_from_slice(&range[0..8]);
                    let value = u64::from_le_bytes(v);
                    trace!("\tACTIVE ZONE MANAGER SAS ADDRESS:      0x{:16x}", value);
                    index += num;
                    num = 2;
                },
                48 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tZONE LOCK INACTIVITY TIME LIMIT:      0x{:16x}", value);
                    index += num + 2;
                    num = 1;
                },
                52 => { // 
                    trace!("\tPOWER DONE TIMEOUT:                   {}", range[0]);
                    index += num;
                    num = 1;
                },
                53 => { // 
                    trace!("\tFIRST ENCLOSURE CONNECTOR ELEMENT INDEX:  {}", range[0]);
                    index += num;
                    num = 1;
                },
                54 => { // 
                    trace!("\tNUMBER OF ENCLOSURE CONNECTOR ELEMENT INDEXES:    {}", range[0]);
                    index += num + 1;
                    num = 1;
                },
                56 => { // 
                    trace!("\tREDUCED FUNCTIONALITY:                {}", range[0] >> 7);
                    index += num;
                    num = 1;
                },
                57 => { // 
                    trace!("\tTIME TO REDUCED FUNCTIONALITY:        {}", range[0]);
                    index += num;
                    num = 1;
                },
                58 => { // 
                    trace!("\tINITIAL TIME TO REDUCED FUNCTIONALITY:{}", range[0]);
                    index += num;
                    num = 1;
                },
                59 => { // 
                    trace!("\tMAXIMUM REDUCED FUNCTIONALITY TIME:   {}", range[0]);
                    index += num;
                    num = 2;
                },
                60 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tLAST SELF-CONFIGURATION STATUS DESCRIPTOR INDEX:  0x{:04x}", value);
                    index += num;
                    num = 2;
                },
                62 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tMAXIMUM NUMBER OF STORED SELF-CONFIGURATION STATUS DESCRIPTORS:   0x{:04x}", value);
                    index += num;
                    num = 2;
                },
                64 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tLAST PHY EVENT LIST DESCRIPTOR INDEX: 0x{:04x}", value);
                    index += num;
                    num = 2;
                },
                66 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tMAXIMUM NUMBER OF STORED PHY EVENT LIST DESCRIPTORS:  0x{:04x}", value);
                    index += num;
                    num = 2;
                },
                68 => { // 
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_be_bytes(v);
                    trace!("\tSTP REJECT TO OPEN LIMIT:     0x{:04x}", value);
                    index += num + 2;
                    num = 4;
                },
                72 => { // 
                    let mut v = [0_u8; 4];
                    v.clone_from_slice(&range[0..4]);
                    let value = u32::from_be_bytes(v);
                    trace!("\tCRC:                          0x{:08x}", value);
                    index += num;
                    num = 1;
                },
                _ => {
                    index += 1;
                    num = 1;
                },
            }
        }

        Ok(())
    }

    fn parse_report_manufacturer_information(&self, bytes: &[u8]) -> Result<(), Error> {
        let mut index = 0;
        let mut num = 1;
        #[allow(unused)]
        let mut resp_dword_num = 0;

        debug!("REPORT MANUFACTURER INFORMATION:");
        while let Some(range) = bytes.get(index..=(index + num)) {
            match index {
                0 => { // smp frame type
                    if range[0] != 0x41 {
                        return Err(anyhow::format_err!("smp frame type should be 0x41"));
                    }
                    index += num;
                    num = 1;
                },
                1 => { // function
                    if range[0] != 0x01 {
                        return Err(anyhow::format_err!("smp report general function should be 0x01"));
                    }
                    index += num;
                    num = 1;
                },
                2 => { // function result
                    trace!("\tFUNCTION RESULT:                      {:02x}", range[0]);
                    index += num;
                    num = 1;
                },
                3 => { // response length
                    resp_dword_num = range[0];
                    if resp_dword_num != 0x00 && resp_dword_num != 0x0E {
                        return Err(anyhow::format_err!("smp report manufacturer information response length should be 0x00 or 0x0E"));
                    }
                    index += num;
                    num = 2;
                },
                4 => { // expander change count
                    let exp_change_cnt: u16 = ((range[0] as u16) << 8) + (range[1] as u16);
                    trace!("\tEXPANDER CHANGE COUNT:                {:02x}", exp_change_cnt);
                    index += num + 6;
                    num = 8;
                },
                12 => { // vendor identification
                    match String::from_utf8(range[0..num].to_vec()) {
                        Ok(value) => {
                            trace!("\tVENDOR IDENTIFICATION:                {}", value);
                        },
                        Err(_) => {
                            error!("\tVENDOR IDENTIFICATION:                null");
                        },
                    }
                    
                    index += num;
                    num = 16;
                },
                20 => { // product identification
                    match String::from_utf8(range[0..num].to_vec()) {
                        Ok(value) => {
                            trace!("\tPRODUCT IDENTIFICATION:               {}", value);
                        },
                        Err(_) => {
                            error!("\tPRODUCT IDENTIFICATION:               null");
                        },
                    }
                    
                    index += num;
                    num = 4;
                },
                36 => { // product revision level
                    match String::from_utf8(range[0..num].to_vec()) {
                        Ok(value) => {
                            trace!("\tPRODUCT REVISION LEVEL:               {}", value);
                        },
                        Err(_) => {
                            error!("\tPRODUCT REVISION LEVEL:               null");
                        },
                    }
                    
                    index += num;
                    num = 8;
                },
                40 => { // component vendor identification
                    match String::from_utf8(range[0..num].to_vec()) {
                        Ok(value) => {
                            trace!("\tCOMPONENT VENDOR IDENTIFICATION:      {}", value);
                        },
                        Err(_) => {
                            error!("\tCOMPONENT VENDOR IDENTIFICATION:      null");
                        },
                    }
                    
                    index += num;
                    num = 2;
                },
                48 => { // component id
                    let mut v = [0_u8; 2];
                    v.clone_from_slice(&range[0..2]);
                    let value = u16::from_le_bytes(v);
                    trace!("\tCOMPONENT ID:                         0x{:04x}", value);
                    index += num;
                    num = 1;
                },
                50 => { // component revision level
                    trace!("\tCOMPONENT REVISION LEVEL:             0x{:02x}", range[0]);
                    index += num + 1;
                    num = 8;
                },
                52 => { // vendor specific
                    match String::from_utf8(range[0..num].to_vec()) {
                        Ok(value) => {
                            trace!("\tVendor specific:              {}", value);
                        },
                        Err(_) => {
                            error!("\tVendor specific:              null");
                        },
                    }
                    
                    index += num;
                    num = 4;
                },
                60 => { //CRC
                    let mut v = [0_u8; 4];
                    v.clone_from_slice(&range[0..4]);
                    let value = u32::from_be_bytes(v);
                    trace!("\tCRC:                          0x{:08x}", value);
                    index += num;
                    num = 1;
                },
                _ => {
                    index += 1;
                    num = 1;
                },
            }
        }

        Ok(())
    }
}

impl Smp for SxpFrame{
    fn parse_smp(&mut self) -> Result<(), Error> {
        if self.frame.len() < 4 {
            return Err(anyhow::format_err!("frame length must be greater than 4. length={}", self.frame.len()));
        }

        // 以dword为单位进行转序
        Self::dword_reverse(&mut self.frame[..]);
        match self.frame[1] {
            0x00 => {
                self.parse_report_general(&self.frame[..])?;
            },
            0x01 => {
                self.parse_report_manufacturer_information(&self.frame[..])?;
            },
            0x10 => {
                debug!("Discover");
            },
            0x20 => {
                debug!("Discover List");
            },
            other => return Err(anyhow::format_err!("invalid smp function: 0x{:02X}", other)),
        }

        //self.print_bytes();

        Ok(())
    }
}

pub trait Ssp {
    fn parse_ssp(&self) -> Result<(), Error>;
}

impl Ssp for SxpFrame {
    fn parse_ssp(&self) -> Result<(), Error> {
        for b in &self.frame {
            print!("{:02x} ", b);
        }
        println!();

        Ok(())
    }
}