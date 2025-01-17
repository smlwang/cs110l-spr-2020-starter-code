use std::collections::HashMap;
use crate::inferior::Inferior;

pub struct BreakpointManager {
    breakpoint_map: HashMap<usize, Option<Breakpoint>>,
    count: usize,
}

#[derive(Clone)]
pub struct Breakpoint {
    addr: usize,
    orig_byte: u8,
}
impl Breakpoint {
    pub fn new(addr: usize, orig_byte: u8) -> Breakpoint {
        Breakpoint { addr, orig_byte }
    }
}
pub enum BreakpointArgType {
    Line(usize),
    FuncName(String),
    Addr(usize),
    Unknown,
}
impl BreakpointManager {
    pub fn new() -> BreakpointManager {
        BreakpointManager { breakpoint_map: HashMap::new(), count: 0 }
    }
    pub fn parse_breakpoint_arg(raw_addr: &str) -> BreakpointArgType {
        if raw_addr.to_lowercase().starts_with('*') {
            let raw_addr_without_0x = if raw_addr.to_lowercase().starts_with("0x") {
                &raw_addr[2..]
            } else {
                &raw_addr
            };
            return match usize::from_str_radix(raw_addr_without_0x, 16).ok() {
                Some(addr) =>  {
                    BreakpointArgType::Addr(addr)
                }
                None => BreakpointArgType::Unknown
            }
        } 
        if let Some(line) = usize::from_str_radix(raw_addr, 10).ok() {
            return BreakpointArgType::Line(line);
        }
        BreakpointArgType::FuncName(raw_addr.to_string())
    }
    
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, usize, Option<Breakpoint>>{
        self.breakpoint_map.iter_mut()
    }
    pub fn get_count(&self) -> usize {
        self.count
    }
    // set breakpoint after use unset_t
    pub fn set_t(&mut self, inferior: &mut Inferior, addr: &usize) -> Result<(), nix::Error> {
        let _ = inferior.write_byte(*addr, 0xcc)?;
        Ok(())
    }
    // temporarily unset breakpoint
    pub fn unset_t(&mut self, inferior: &mut Inferior, addr: &usize) -> Result<(), nix::Error>{
        if let Some((_, Some(breakpoint))) = self.breakpoint_map.get_key_value(&addr) {
            let _ = inferior.write_byte(breakpoint.addr, breakpoint.orig_byte)?;
        }
        Ok(())
    }
    pub fn unset(&mut self, inferior: &mut Option<Inferior>, addr: &usize) -> Result<(), nix::Error>{
        if let Some((_, Some(breakpoint))) = self.breakpoint_map.remove_entry(&addr) {
            match inferior.as_mut() {
                None => {},
                Some(i) => {
                    let _ = i.write_byte(breakpoint.addr, breakpoint.orig_byte)?;
                }
            }
        }
        Ok(())
    }
    pub fn get(&mut self, addr: &usize) -> Option<Breakpoint> {
        self.breakpoint_map.get(addr)?.clone()
    }
    pub fn set(&mut self, inferior: &mut Option<Inferior>, addr: &usize) -> Result<bool, nix::Error> {
        let breakpoint = match inferior.as_mut() {
            None => {
                None
            },
            Some(i) => {
                Some(Breakpoint{ addr: *addr, orig_byte: i.write_byte(*addr, 0xcc)? })
            }
        };
        if self.breakpoint_map.insert(*addr, breakpoint).is_none() {
            self.count += 1;
            return Ok(true);
        }
        Ok(false)
    }
}