use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::{IndexedBitmap, Palette, PaletteError, PaletteFormat, RgbaBitmap};
use crate::utils::{pack_bits, unpack_bits, PackBitsError};

#[derive(Error, Debug)]
pub enum IffError {
	#[error("Bad or unsupported IFF file: {0}")]
	BadFile(String),

	#[error("IFF palette data error")]
	BadPalette(#[from] PaletteError),

	#[error("PackBits error")]
	PackBitsError(#[from] PackBitsError),

	#[error("IFF I/O error")]
	IOError(#[from] std::io::Error),
}

pub enum IffFormat {
	Pbm,
	PbmUncompressed,
	Ilbm,
	IlbmUncompressed,
}

impl IffFormat {
	pub fn compressed(&self) -> bool {
		use IffFormat::*;
		match self {
			Pbm | Ilbm => true,
			PbmUncompressed | IlbmUncompressed => false,
		}
	}

	pub fn chunky(&self) -> bool {
		use IffFormat::*;
		match self {
			Pbm | PbmUncompressed => true,
			Ilbm | IlbmUncompressed => false,
		}
	}

	pub fn type_id(&self) -> [u8; 4] {
		use IffFormat::*;
		match self {
			Pbm | PbmUncompressed => *b"PBM ",
			Ilbm | IlbmUncompressed => *b"ILBM",
		}
	}
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct IffId {
	id: [u8; 4],
}

impl IffId {
	pub fn read<T: Read>(reader: &mut T) -> Result<Self, IffError> {
		let mut id = [0u8; 4];
		reader.read_exact(&mut id)?;
		Ok(IffId { id })
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), IffError> {
		writer.write_all(&self.id)?;
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct FormChunkHeader {
	chunk_id: IffId,
	size: u32,
	type_id: IffId,
}

impl FormChunkHeader {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, IffError> {
		let chunk_id = IffId::read(reader)?;
		let size = reader.read_u32::<BigEndian>()?;
		let type_id = IffId::read(reader)?;
		Ok(FormChunkHeader { chunk_id, size, type_id })
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), IffError> {
		self.chunk_id.write(writer)?;
		writer.write_u32::<BigEndian>(self.size)?;
		self.type_id.write(writer)?;
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct SubChunkHeader {
	chunk_id: IffId,
	size: u32,
}

impl SubChunkHeader {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, IffError> {
		let chunk_id = IffId::read(reader)?;
		let mut size = reader.read_u32::<BigEndian>()?;
		if (size & 1) == 1 {
			size += 1; // account for the padding byte
		}
		Ok(SubChunkHeader { chunk_id, size })
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), IffError> {
		self.chunk_id.write(writer)?;
		writer.write_u32::<BigEndian>(self.size)?;
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct BMHDChunk {
	width: u16,
	height: u16,
	left: u16,
	top: u16,
	bitplanes: u8,
	masking: u8,
	compress: u8,
	padding: u8,
	transparency: u16,
	x_aspect_ratio: u8,
	y_aspect_ratio: u8,
	page_width: u16,
	page_height: u16,
}

impl BMHDChunk {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, IffError> {
		Ok(BMHDChunk {
			width: reader.read_u16::<BigEndian>()?,
			height: reader.read_u16::<BigEndian>()?,
			left: reader.read_u16::<BigEndian>()?,
			top: reader.read_u16::<BigEndian>()?,
			bitplanes: reader.read_u8()?,
			masking: reader.read_u8()?,
			compress: reader.read_u8()?,
			padding: reader.read_u8()?,
			transparency: reader.read_u16::<BigEndian>()?,
			x_aspect_ratio: reader.read_u8()?,
			y_aspect_ratio: reader.read_u8()?,
			page_width: reader.read_u16::<BigEndian>()?,
			page_height: reader.read_u16::<BigEndian>()?,
		})
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), IffError> {
		writer.write_u16::<BigEndian>(self.width)?;
		writer.write_u16::<BigEndian>(self.height)?;
		writer.write_u16::<BigEndian>(self.left)?;
		writer.write_u16::<BigEndian>(self.top)?;
		writer.write_u8(self.bitplanes)?;
		writer.write_u8(self.masking)?;
		writer.write_u8(self.compress)?;
		writer.write_u8(self.padding)?;
		writer.write_u16::<BigEndian>(self.transparency)?;
		writer.write_u8(self.x_aspect_ratio)?;
		writer.write_u8(self.y_aspect_ratio)?;
		writer.write_u16::<BigEndian>(self.page_width)?;
		writer.write_u16::<BigEndian>(self.page_height)?;
		Ok(())
	}
}

fn merge_bitplane(plane: u32, src: &[u8], dest: &mut [u8], row_size: usize) {
	let bitmask = 1 << plane;
	for x in 0..row_size {
		let data = src[x];
		if (data & 128) > 0 {
			dest[x * 8] |= bitmask;
		}
		if (data & 64) > 0 {
			dest[(x * 8) + 1] |= bitmask;
		}
		if (data & 32) > 0 {
			dest[(x * 8) + 2] |= bitmask;
		}
		if (data & 16) > 0 {
			dest[(x * 8) + 3] |= bitmask;
		}
		if (data & 8) > 0 {
			dest[(x * 8) + 4] |= bitmask;
		}
		if (data & 4) > 0 {
			dest[(x * 8) + 5] |= bitmask;
		}
		if (data & 2) > 0 {
			dest[(x * 8) + 6] |= bitmask;
		}
		if (data & 1) > 0 {
			dest[(x * 8) + 7] |= bitmask;
		}
	}
}

fn extract_bitplane(plane: u32, src: &[u8], dest: &mut [u8], row_size: usize) {
	let bitmask = 1 << plane;
	let mut src_base_index = 0;
	for x in 0..row_size {
		let mut data = 0;
		if src[src_base_index] & bitmask != 0 {
			data |= 128;
		}
		if src[src_base_index + 1] & bitmask != 0 {
			data |= 64;
		}
		if src[src_base_index + 2] & bitmask != 0 {
			data |= 32;
		}
		if src[src_base_index + 3] & bitmask != 0 {
			data |= 16;
		}
		if src[src_base_index + 4] & bitmask != 0 {
			data |= 8;
		}
		if src[src_base_index + 5] & bitmask != 0 {
			data |= 4;
		}
		if src[src_base_index + 6] & bitmask != 0 {
			data |= 2;
		}
		if src[src_base_index + 7] & bitmask != 0 {
			data |= 1;
		}

		src_base_index += 8;
		dest[x] = data;
	}
}

fn load_planar_body<T: ReadBytesExt>(reader: &mut T, bmhd: &BMHDChunk) -> Result<IndexedBitmap, IffError> {
	let mut bitmap = IndexedBitmap::new(bmhd.width as u32, bmhd.height as u32).unwrap();

	let row_bytes = (((bmhd.width + 15) >> 4) << 1) as usize;
	let mut buffer = vec![0u8; row_bytes];

	for y in 0..bmhd.height {
		// planar data is stored for each bitplane in sequence for the scanline.
		// that is, ALL of bitplane1, followed by ALL of bitplane2, etc, NOT
		// alternating after each pixel. if compression is enabled, it does NOT
		// cross bitplane boundaries. each bitplane is compressed individually.
		// bitplanes also do NOT cross the scanline boundary. basically, each
		// scanline of pixel data, and within that, each of the bitplanes of
		// pixel data found in each scanline can all be treated as they are all
		// their own self-contained bit of data as far as this loading process
		// is concerned (well, except that we merge all of the scanline's
		// bitplanes together at the end of each line)

		// read all the bitplane rows per scanline
		for plane in 0..(bmhd.bitplanes as u32) {
			if bmhd.compress == 1 {
				// decompress packed line for this bitplane only
				buffer.clear();
				unpack_bits(reader, &mut buffer, row_bytes)?
			} else {
				// TODO: check this. maybe row_bytes calculation is wrong? either way, i don't
				//       think that DP2 or Grafx2 ever output uncompressed interleaved files ...
				// just read all this bitplane's line data in as-is
				reader.read_exact(&mut buffer)?;
			}

			// merge this bitplane data into the final destination. after all of
			// the bitplanes have been loaded and merged in this way for this
			// scanline, the destination pointer will contain VGA-friendly
			// "chunky pixel"-format pixel data
			merge_bitplane(
				plane, //
				&buffer,
				bitmap.pixels_at_mut(0, y as i32).unwrap(),
				row_bytes,
			);
		}
	}

	Ok(bitmap)
}

fn load_chunky_body<T: ReadBytesExt>(reader: &mut T, bmhd: &BMHDChunk) -> Result<IndexedBitmap, IffError> {
	let mut bitmap = IndexedBitmap::new(bmhd.width as u32, bmhd.height as u32).unwrap();

	for y in 0..bmhd.height {
		if bmhd.compress == 1 {
			// for compression-enabled, read row of pixels using PackBits
			let mut writer = bitmap.pixels_at_mut(0, y as i32).unwrap();
			unpack_bits(reader, &mut writer, bmhd.width as usize)?
		} else {
			// for uncompressed, read row of pixels literally
			let dest = &mut bitmap.pixels_at_mut(0, y as i32).unwrap()[0..bmhd.width as usize];
			reader.read_exact(dest)?;
		}
	}

	Ok(bitmap)
}

fn write_planar_body<T: WriteBytesExt>(
	writer: &mut T,
	bitmap: &IndexedBitmap,
	bmhd: &BMHDChunk,
) -> Result<(), IffError> {
	let row_bytes = (((bitmap.width() + 15) >> 4) << 1) as usize;
	let mut buffer = vec![0u8; row_bytes];

	for y in 0..bitmap.height() {
		for plane in 0..(bmhd.bitplanes as u32) {
			extract_bitplane(
				plane, //
				bitmap.pixels_at(0, y as i32).unwrap(),
				&mut buffer,
				row_bytes,
			);

			if bmhd.compress == 1 {
				// for compression-enabled, write this plane's pixels using PackBits
				pack_bits(&mut buffer.as_slice(), writer, row_bytes)?;
			} else {
				// TODO: check this. maybe row_bytes calculation is wrong? either way, i don't
				//       think that DP2 or Grafx2 ever output uncompressed interleaved files ...
				// for uncompressed, write this plane's pixels literally
				writer.write_all(&buffer)?;
			}
		}
	}

	Ok(())
}

fn write_chunky_body<T: WriteBytesExt>(
	writer: &mut T,
	bitmap: &IndexedBitmap,
	bmhd: &BMHDChunk,
) -> Result<(), IffError> {
	for y in 0..bitmap.height() {
		if bmhd.compress == 1 {
			// for compression-enabled, write row of pixels using PackBits
			let mut reader = bitmap.pixels_at(0, y as i32).unwrap();
			pack_bits(&mut reader, writer, bitmap.width() as usize)?;
		} else {
			// for uncompressed, write out the row of pixels literally
			let src = &bitmap.pixels_at(0, y as i32).unwrap()[0..bitmap.width() as usize];
			writer.write_all(src)?;
		}
	}

	Ok(())
}

impl IndexedBitmap {
	pub fn load_iff_bytes<T: ReadBytesExt + Seek>(reader: &mut T) -> Result<(IndexedBitmap, Palette), IffError> {
		let form_chunk = FormChunkHeader::read(reader)?;
		if form_chunk.chunk_id.id != *b"FORM" {
			return Err(IffError::BadFile(String::from("Unexpected form chunk ID, probably not an IFF file")));
		}
		if form_chunk.type_id.id != *b"ILBM" && form_chunk.type_id.id != *b"PBM " {
			return Err(IffError::BadFile(String::from("Only ILBM or PBM formats are supported")));
		}

		let mut bmhd: Option<BMHDChunk> = None;
		let mut palette: Option<Palette> = None;
		let mut bitmap: Option<IndexedBitmap> = None;

		loop {
			let header = match SubChunkHeader::read(reader) {
				Ok(header) => header,
				Err(IffError::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
					break;
				}
				Err(err) => return Err(err),
			};
			let chunk_data_position = reader.stream_position()?;

			// todo: process chunk here
			if header.chunk_id.id == *b"BMHD" {
				bmhd = Some(BMHDChunk::read(reader)?);
				if bmhd.as_ref().unwrap().bitplanes != 8 {
					return Err(IffError::BadFile(String::from("Only 8bpp files are supported")));
				}
				if bmhd.as_ref().unwrap().masking == 1 {
					return Err(IffError::BadFile(String::from("Masking is not supported")));
				}
			} else if header.chunk_id.id == *b"CMAP" {
				if header.size != 768 {
					return Err(IffError::BadFile(String::from("Only 256 color files are supported")));
				}
				palette = Some(Palette::load_from_bytes(reader, PaletteFormat::Normal)?)
			} else if header.chunk_id.id == *b"BODY" {
				if let Some(bmhd) = &bmhd {
					if form_chunk.type_id.id == *b"PBM " {
						bitmap = Some(load_chunky_body(reader, bmhd)?);
					} else {
						bitmap = Some(load_planar_body(reader, bmhd)?);
					}
				} else {
					// TODO: does this ever occur in practice? and if so, we can probably make some
					//       changes to allow for it ...
					return Err(IffError::BadFile(String::from(
						"BODY chunk occurs before BMHD chunk, or no BMHD chunk exists",
					)));
				}
			}

			reader.seek(SeekFrom::Start(chunk_data_position + header.size as u64))?;
		}

		if bitmap.is_none() {
			return Err(IffError::BadFile(String::from("No BODY chunk was found")));
		}
		// TODO: we can probably make this optional ...
		if palette.is_none() {
			return Err(IffError::BadFile(String::from("No CMAP chunk was found")));
		}

		Ok((bitmap.unwrap(), palette.unwrap()))
	}

	pub fn load_iff_file(path: &Path) -> Result<(IndexedBitmap, Palette), IffError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_iff_bytes(&mut reader)
	}

	pub fn to_iff_bytes<T: WriteBytesExt + Seek>(
		&self,
		writer: &mut T,
		palette: &Palette,
		format: IffFormat,
	) -> Result<(), IffError> {
		let form_chunk_position = writer.stream_position()?;

		let mut form_chunk = FormChunkHeader {
			chunk_id: IffId { id: *b"FORM" },
			type_id: IffId { id: format.type_id() },
			size: 0, // filled in later once we know the size
		};

		// skip over the form chunk for now. will come back here and write it out later once we
		// know what the final size is
		writer.seek(SeekFrom::Current(std::mem::size_of::<FormChunkHeader>() as i64))?;

		let bmhd_chunk_header =
			SubChunkHeader { chunk_id: IffId { id: *b"BMHD" }, size: std::mem::size_of::<BMHDChunk>() as u32 };
		let bmhd = BMHDChunk {
			width: self.width() as u16,
			height: self.height() as u16,
			left: 0,
			top: 0,
			bitplanes: 8,
			masking: 0,
			compress: if format.compressed() { 1 } else { 0 },
			padding: 0,
			transparency: 0,
			// the following values are based on what DP2 writes out in 320x200 modes. good enough.
			x_aspect_ratio: 5,
			y_aspect_ratio: 6,
			page_width: 320,
			page_height: 200,
		};
		bmhd_chunk_header.write(writer)?;
		bmhd.write(writer)?;

		let cmap_chunk_header = SubChunkHeader {
			chunk_id: IffId { id: *b"CMAP" },
			size: 768, //
		};
		cmap_chunk_header.write(writer)?;
		palette.to_bytes(writer, PaletteFormat::Normal)?;

		let body_position = writer.stream_position()?;

		let mut body_chunk_header = SubChunkHeader {
			chunk_id: IffId { id: *b"BODY" },
			size: 0, // filled in later once we know the size
		};

		// skip over the body chunk header for now. we will again come back here and write it out
		// later once we know what the final size again.
		writer.seek(SeekFrom::Current(std::mem::size_of::<SubChunkHeader>() as i64))?;

		if format.chunky() {
			write_chunky_body(writer, self, &bmhd)?;
		} else {
			write_planar_body(writer, self, &bmhd)?;
		}

		// add a padding byte (only if necessary) to the body we just finished writing
		let mut eof_pos = writer.stream_position()?;
		if (eof_pos - body_position) & 1 == 1 {
			writer.write_u8(0)?;
			eof_pos += 1;
		}

		// go back and write out the form chunk header now that we know the final file size
		form_chunk.size = (eof_pos - (std::mem::size_of::<IffId>() as u64 * 2)) as u32;
		writer.seek(SeekFrom::Start(form_chunk_position))?;
		form_chunk.write(writer)?;

		// and then write out the body chunk header since we now know the size of that too
		body_chunk_header.size = eof_pos as u32 - std::mem::size_of::<SubChunkHeader>() as u32;
		writer.seek(SeekFrom::Start(body_position))?;
		body_chunk_header.write(writer)?;

		// and then go back to eof
		writer.seek(SeekFrom::Start(eof_pos))?;

		Ok(())
	}

	pub fn to_iff_file(&self, path: &Path, palette: &Palette, format: IffFormat) -> Result<(), IffError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_iff_bytes(&mut writer, palette, format)
	}
}

// wasteful temporary measures until i feel like re-working the above loading process with some kind of
// multi-pixel-depth support.

impl RgbaBitmap {
	pub fn load_iff_bytes<T: ReadBytesExt + Seek>(reader: &mut T) -> Result<(RgbaBitmap, Palette), IffError> {
		let (temp_bitmap, palette) = IndexedBitmap::load_iff_bytes(reader)?;
		let output = temp_bitmap.to_rgba(&palette);
		Ok((output, palette))
	}

	pub fn load_iff_file(path: &Path) -> Result<(RgbaBitmap, Palette), IffError> {
		let (temp_bitmap, palette) = IndexedBitmap::load_iff_file(path)?;
		let output = temp_bitmap.to_rgba(&palette);
		Ok((output, palette))
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use tempfile::TempDir;

	use crate::tests::{load_raw_indexed, test_assets_file};

	use super::*;

	const BASE_PATH: &str = "./test-assets/iff/";

	fn test_file(file: &Path) -> PathBuf {
		PathBuf::from(BASE_PATH).join(file)
	}

	#[test]
	pub fn load_and_save() -> Result<(), IffError> {
		let tmp_dir = TempDir::new()?;

		let ref_pixels = load_raw_indexed(test_file(Path::new("small.bin")).as_path())?;
		let dp2_palette = Palette::load_from_file(
			test_assets_file(Path::new("dp2.pal")).as_path(), //
			PaletteFormat::Normal,
		)
		.unwrap();

		// ILBM format

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("small.lbm")).as_path())?;
		assert_eq!(16, bmp.width());
		assert_eq!(16, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());
		assert_eq!(palette, dp2_palette);

		let save_path = tmp_dir.path().join("test_save.lbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Ilbm)?;
		let (reloaded_bmp, reloaded_palette) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(16, reloaded_bmp.width());
		assert_eq!(16, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());
		assert_eq!(reloaded_palette, dp2_palette);

		// PBM format

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("small.pbm")).as_path())?;
		assert_eq!(16, bmp.width());
		assert_eq!(16, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());
		assert_eq!(palette, dp2_palette);

		let save_path = tmp_dir.path().join("test_save.pbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Pbm)?;
		let (reloaded_bmp, reloaded_palette) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(16, reloaded_bmp.width());
		assert_eq!(16, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());
		assert_eq!(reloaded_palette, dp2_palette);

		Ok(())
	}

	#[test]
	pub fn load_and_save_larger_image() -> Result<(), IffError> {
		let tmp_dir = TempDir::new()?;

		// first image, PBM format

		let ref_pixels = load_raw_indexed(test_file(Path::new("large_1.bin")).as_path())?;

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("large_1.pbm")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());

		let save_path = tmp_dir.path().join("test_save.pbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Pbm)?;
		let (reloaded_bmp, _) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());

		// first image, ILBM format

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("large_1.lbm")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());

		let save_path = tmp_dir.path().join("test_save.lbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Ilbm)?;
		let (reloaded_bmp, _) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());

		// second image, PBM format

		let ref_pixels = load_raw_indexed(test_file(Path::new("large_2.bin")).as_path())?;

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("large_2.lbm")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());

		let save_path = tmp_dir.path().join("test_save_2.pbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Pbm)?;
		let (reloaded_bmp, _) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());

		// second image, ILBM format

		let (bmp, palette) = IndexedBitmap::load_iff_file(test_file(Path::new("large_2.lbm")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), ref_pixels.as_ref());

		let save_path = tmp_dir.path().join("test_save_2.lbm");
		bmp.to_iff_file(&save_path, &palette, IffFormat::Ilbm)?;
		let (reloaded_bmp, _) = IndexedBitmap::load_iff_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), ref_pixels.as_ref());

		Ok(())
	}
}
