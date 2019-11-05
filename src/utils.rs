use std::io::Write;

use rmp::encode::{write_map_len, write_str, ValueWriteError};
use rmp::Marker;
use rmp_serde::encode::VariantWriter;

pub struct StructMapWriter;

impl VariantWriter for StructMapWriter {
    fn write_struct_len<W>(&self, wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
    where
        W: Write,
    {
        write_map_len(wr, len)
    }

    fn write_field_name<W>(&self, wr: &mut W, _key: &str) -> Result<(), ValueWriteError>
    where
        W: Write,
    {
        write_str(wr, _key)
    }
}
