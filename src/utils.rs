use rmp::Marker;
use rmp::encode::{ValueWriteError, write_map_len, write_str};
use rmp_serde::encode::VariantWriter;
use std::io::Write;


pub struct StructMapWriter;

impl VariantWriter for StructMapWriter {
    fn write_struct_len<W>(&self, wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
        where W: Write
    {
        write_map_len(wr, len)
    }

    fn write_field_name<W>(&self, wr: &mut W, _key: &str) -> Result<(), ValueWriteError>
        where W: Write
    {
        write_str(wr, _key)
    }
}
