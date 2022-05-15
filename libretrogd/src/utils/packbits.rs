use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackBitsError {
    #[error("PackBits I/O error")]
    IOError(#[from] std::io::Error),
}

enum PackMode {
    Dump,
    Run,
}

pub fn pack_bits<S, D>(src: &mut S, dest: &mut D, src_length: usize) -> Result<(), PackBitsError>
where
    S: ReadBytesExt,
    D: WriteBytesExt,
{
    const MIN_RUN: usize = 3;
    const MAX_RUN: usize = 128;
    const MAX_BUFFER: usize = 128;

    if src_length == 0 {
        return Ok(());
    }
    let mut bytes_left = src_length;

    let mut buffer = [0u8; (MAX_RUN * 2)];

    // read the first byte from the source, just to start things off before we get into the loop
    buffer[0] = src.read_u8()?;
    bytes_left -= 1;

    let mut mode = PackMode::Dump;
    let mut run_end = 1; // 1 because we already read the first byte into the buffer
    let mut run_start = 0;
    let mut previous_byte = buffer[0];

    while bytes_left > 0 {
        let byte = src.read_u8()?;

        buffer[run_end] = byte;
        run_end += 1;

        match mode {
            // "dump" mode. keep collecting any bytes and write them as-is until we detect the
            // start of a run of identical bytes
            PackMode::Dump => {
                if run_end > MAX_BUFFER {
                    // we need to flush the temp buffer to the destination
                    dest.write_u8((run_end - 2) as u8)?;
                    dest.write_all(&buffer[0..run_end])?;

                    buffer[0] = byte;
                    run_end = 1;
                    run_start = 0;
                } else if byte == previous_byte {
                    // detected the start of a run of identical bytes
                    if (run_end - run_start) >= MIN_RUN {
                        if run_start > 0 {
                            // we've found a run, flush the buffer we have currently so we can
                            // start tracking the length of this run
                            dest.write_u8((run_start - 1) as u8)?;
                            dest.write_all(&buffer[0..run_start])?;
                        }
                        mode = PackMode::Run;
                    } else if run_start == 0 {
                        mode = PackMode::Run;
                    }
                } else {
                    run_start = run_end - 1;
                }
            }
            // "run" mode. keep counting up bytes as long as they are identical to each other. when
            // we find the end of a run, write out the run info and switch back to dump mode
            PackMode::Run => {
                // check for the end of a run of identical bytes
                if (byte != previous_byte) || ((run_end - run_start) > MAX_RUN) {
                    // the identical byte run has ended, write it out to the destination
                    // (this is just two bytes, the count and the actual byte)
                    dest.write_i8(-((run_end - run_start - 2) as i8))?;
                    dest.write_u8(previous_byte)?;

                    // clear the temp buffer for our switch back to "dump" mode
                    buffer[0] = byte;
                    run_end = 1;
                    run_start = 0;
                    mode = PackMode::Dump;
                }
            }
        };

        previous_byte = byte;
        bytes_left -= 1;
    }

    // the source bytes have all been read, but we still might have to flush the temp buffer
    // out to the destination, or finish writing out a run of identical bytes that was at the very
    // end of the source
    match mode {
        PackMode::Dump => {
            dest.write_u8((run_end - 1) as u8)?;
            dest.write_all(&buffer[0..run_end])?;
        }
        PackMode::Run => {
            dest.write_i8(-((run_end - run_start - 1) as i8))?;
            dest.write_u8(previous_byte)?;
        }
    };

    Ok(())
}

pub fn unpack_bits<S, D>(
    src: &mut S,
    dest: &mut D,
    unpacked_length: usize,
) -> Result<(), PackBitsError>
where
    S: ReadBytesExt,
    D: WriteBytesExt,
{
    let mut buffer = [0u8; 128];
    let mut bytes_written = 0;

    while bytes_written < unpacked_length {
        // read the next "code" byte that determines how to process the subsequent byte(s)
        let byte = src.read_u8()?;

        if byte > 128 {
            // 129-255 = repeat the next byte 257-n times
            let run_length = (257 - byte as u32) as usize;

            // read the next byte from the source and repeat it the specified number of times
            let byte = src.read_u8()?;
            buffer.fill(byte);
            dest.write_all(&buffer[0..run_length])?;
            bytes_written += run_length;
        } else if byte < 128 {
            // 0-128 = copy next n-1 bytes from src to dest as-is
            let run_length = (byte + 1) as usize;

            src.read_exact(&mut buffer[0..run_length])?;
            dest.write_all(&buffer[0..run_length])?;
            bytes_written += run_length;
        }

        // note that byte == 128 is a no-op (does it even appear in any files ???)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    struct TestData<'a> {
        packed: &'a [u8],
        unpacked: &'a [u8],
    }

    static TEST_DATA: &[TestData] = &[
        TestData {
            packed: &[
                0xfe, 0xaa, 0x02, 0x80, 0x00, 0x2a, 0xfd, 0xaa, 0x03, 0x80, 0x00, 0x2a, 0x22, 0xf7,
                0xaa,
            ],
            unpacked: &[
                0xaa, 0xaa, 0xaa, 0x80, 0x00, 0x2a, 0xaa, 0xaa, 0xaa, 0xaa, 0x80, 0x00, 0x2a, 0x22,
                0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            ],
        },
        TestData {
            packed: &[0x00, 0xaa],
            unpacked: &[0xaa],
        },
        TestData {
            packed: &[0xf9, 0xaa],
            unpacked: &[0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa],
        },
        TestData {
            packed: &[0xf9, 0xaa, 0x00, 0xbb],
            unpacked: &[0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xbb],
        },
        TestData {
            packed: &[0x07, 0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8],
            unpacked: &[0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8],
        },
        TestData {
            packed: &[0x08, 0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa8],
            unpacked: &[0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa8],
        },
        TestData {
            packed: &[0x06, 0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xfe, 0xa8],
            unpacked: &[0xa0, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa8, 0xa8],
        },
    ];

    #[test]
    fn packs() -> Result<(), PackBitsError> {
        for TestData { packed, unpacked } in TEST_DATA {
            let mut src = Cursor::new(*unpacked);
            let mut dest = vec![0u8; 0];
            pack_bits(&mut src, &mut dest, unpacked.len())?;
            assert_eq!(dest, *packed);
        }

        Ok(())
    }

    #[test]
    fn unpacks() -> Result<(), PackBitsError> {
        for TestData { packed, unpacked } in TEST_DATA {
            let mut src = Cursor::new(*packed);
            let mut dest = vec![0u8; 0];
            unpack_bits(&mut src, &mut dest, unpacked.len())?;
            assert_eq!(dest, *unpacked);
        }

        Ok(())
    }
}
