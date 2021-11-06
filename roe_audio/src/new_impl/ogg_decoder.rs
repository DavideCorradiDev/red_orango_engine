use lewton::inside_ogg::OggStreamReader;

use super::{AudioFormat, Decoder};

struct Packet {
    data: Vec<i16>,
    start_byte_stream_position: u64,
}

pub struct OggDecoder<T: std::io::Read + std::io::Seek> {
    input: OggStreamReader<T>,
    last_packet: Option<Packet>,
    format: AudioFormat,
    sample_count: usize,
    sample_stream_position: u64,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, OggDecoderCreationError> {
        let mut input = OggStreamReader::new(input)?;
        let format = AudioFormat::new(input.ident_hdr.audio_channels as u32, 2);
        if format.bytes_per_sample() != 2 {
            return Err(OggDecoderCreationError::UnsupportedFormat(format));
        }
        let sample_count = Self::compute_sample_count(&mut input)?;

        Ok(Self {
            input,
            last_packet: None,
            format,
            sample_count,
            sample_stream_position: 0,
        })
    }

    // fn compute_byte_data_offset(input: &mut OggStreamReader<T>) -> Result<usize, OggDecoderCreationError> {
    //     let mut byte_data_offset = None;
    //     while byte_data_offset == 0 {
    //         input.read_dec_packet()?;
    //     }
    // }

    fn compute_sample_count(input: &mut OggStreamReader<T>) -> Result<usize, OggDecoderCreationError> {
        let mut sample_count = 0;
        println!("Position: {}", input.get_last_absgp().unwrap_or(7887907));
        loop {
            let data = input.read_dec_packet_itl()?;
            println!("Position: {}", input.get_last_absgp().unwrap_or(7887907));
            match data {
                Some(data) => {
                    sample_count += data.len();
                    println!("Read {} data", data.len());
                }
                None => break,
            }
        }
        println!("Position: {}", input.get_last_absgp().unwrap_or(0));
        sample_count /= input.ident_hdr.audio_channels as usize;
        input.seek_absgp_pg(0)?;
        let data = input.read_dec_packet_itl().expect("FELL INTO A TRAP!");
        println!("READ PACKET AGAIN!");
        Ok(sample_count)
    }

    fn read_packet(&mut self) -> Result<Option<&Packet>, std::io::Error> {
        if self.must_read_new_packet()? {
            self.read_new_packet()?;
        }
        match &self.last_packet {
            Some(packet) => Ok(Some(&packet)),
            None => Ok(None),
        }
    }

    fn must_read_new_packet(&mut self) -> std::io::Result<bool> {
        let bps = self.audio_format().bytes_per_sample() as u64;
        let byte_stream_pos = self.byte_stream_position()?;
        match &self.last_packet {
            Some(packet) => {
                let packet_end_byte_stream_pos =
                    packet.start_byte_stream_position + packet.data.len() as u64 * bps;
                Ok(byte_stream_pos < packet.start_byte_stream_position
                    || byte_stream_pos >= packet_end_byte_stream_pos)
            }
            None => Ok(true),
        }
    }

    fn read_new_packet(&mut self) -> std::io::Result<()> {
        self.last_packet = match self.input.read_dec_packet_itl() {
            Ok(packet) => match packet {
                Some(data) => {
                    let bps = self.audio_format().bytes_per_sample() as u64;
                    let start_byte_stream_position = match self.input.get_last_absgp() {
                        Some(v) => v * bps,
                        None => {
                            panic!("Unexpected failure when reading ogg page start position")
                        }
                    };
                    Some(Packet {
                        data,
                        start_byte_stream_position,
                    })
                }
                None => None,
            },
            Err(e) => match e {
                lewton::VorbisError::OggError(e) => match e {
                    OggReadError::ReadError(e) => return Err(e),
                    _ => panic!("Unexpected error while reading from an ogg file ({})", e),
                },
                _ => panic!("Unexpected error while reading from an ogg file ({})", e),
            },
        };
        Ok(())
    }
}

impl<T> std::fmt::Debug for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OggDecoder {{ }}")
    }
}

impl<T> Decoder for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn sample_rate(&self) -> u32 {
        self.input.ident_hdr.audio_sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.sample_stream_position)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let byte_count = self.byte_count() as i64;
        let target_pos = match pos {
            std::io::SeekFrom::Start(v) => v as i64,
            std::io::SeekFrom::End(v) => byte_count + v,
            std::io::SeekFrom::Current(v) => self.byte_stream_position()? as i64 + v,
        };
        let target_pos = std::cmp::max(0, std::cmp::min(target_pos, byte_count)) as u64;

        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(
            target_pos % tbps == 0,
            "Invalid seek offset ({})",
            target_pos
        );

        self.input
            .seek_absgp_pg(target_pos / tbps)
            .expect("Failed to seek ogg file");
        self.last_packet = None;
        self.sample_stream_position = target_pos / tbps;
        Ok(target_pos)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        assert!(
            buf.len() % tbps == 0,
            "Invalid buffer length ({})",
            buf.len()
        );

        let mut read_byte_count = 0;
        let mut byte_stream_pos = self.byte_stream_position()?;

        while read_byte_count < buf.len() {
            match self.read_packet()? {
                None => break,
                Some(packet) => {
                    let packet_data = bytemuck::cast_slice::<_, u8>(&packet.data[..]);

                    // Note: read_packet is guaranteed to return the packet where the current stream
                    // position falls into, this is why we can make the assertions in the following
                    // lines.
                    assert!(packet.start_byte_stream_position <= byte_stream_pos);
                    let read_start = (byte_stream_pos - packet.start_byte_stream_position) as usize;
                    assert!(read_start < packet_data.len());

                    let write_start = read_byte_count;

                    let byte_to_read_count =
                        std::cmp::min(buf.len() - write_start, packet_data.len() - read_start);

                    let read_end = read_start + byte_to_read_count;
                    let write_end = write_start + byte_to_read_count;

                    buf[write_start..write_end]
                        .clone_from_slice(&packet_data[read_start..read_end]);

                    byte_stream_pos += byte_to_read_count as u64;
                    read_byte_count += byte_to_read_count;
                }
            }
        }

        self.sample_stream_position += (read_byte_count / tbps) as u64;
        Ok(read_byte_count)
    }
}

pub use lewton::{
    audio::AudioReadError as OggDataError, header::HeaderReadError as OggHeaderError, OggReadError,
};

#[derive(Debug)]
pub enum OggDecoderCreationError {
    ReadFailed(OggReadError),
    InvalidHeader(OggHeaderError),
    InvalidData(OggDataError),
    UnsupportedFormat(AudioFormat),
}

impl std::fmt::Display for OggDecoderCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFailed(e) => write!(f, "Failed to read data ({})", e),
            Self::InvalidHeader(e) => write!(f, "Invalid header ({})", e),
            Self::InvalidData(e) => write!(f, "Invalid data ({})", e),
            Self::UnsupportedFormat(format) => write!(f, "Unsupported audio format ({:?})", format),
        }
    }
}

impl std::error::Error for OggDecoderCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFailed(e) => Some(e),
            Self::InvalidHeader(e) => Some(e),
            Self::InvalidData(e) => Some(e),
            _ => None,
        }
    }
}

impl From<lewton::VorbisError> for OggDecoderCreationError {
    fn from(e: lewton::VorbisError) -> Self {
        match e {
            lewton::VorbisError::BadAudio(e) => Self::InvalidData(e),
            lewton::VorbisError::BadHeader(e) => Self::InvalidHeader(e),
            lewton::VorbisError::OggError(e) => Self::ReadFailed(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn invalid_input_file() {
        let file = std::fs::File::open("data/audio/not-an-audio-file.txt").unwrap();
        let buf = std::io::BufReader::new(file);
        expect_that!(&OggDecoder::new(buf), is_variant!(Result::Err));
    }

    #[test]
    fn mono16_loading() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = OggDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono16));
        expect_that!(&decoder.byte_count(), eq(24 * 2));
        expect_that!(&decoder.sample_count(), eq(24));
        expect_that!(&decoder.byte_rate(), eq(44100 * 2));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn mono16_byte_seek() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(12)).unwrap(),
            eq(12)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(6));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(4)).unwrap(),
            eq(16)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(8));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-8)).unwrap(),
            eq(8)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-12)).unwrap(),
            eq(36)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(36));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(18));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(48)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(48));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(24));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    #[should_panic(expected = "Invalid seek offset (3)")]
    fn mono16_byte_seek_invalid_offset() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(3)).unwrap();
    }

    #[test]
    fn mono16_sample_seek() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(3)).unwrap(),
            eq(3)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(6));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(1)).unwrap(),
            eq(4)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-2)).unwrap(),
            eq(2)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(4));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-3)).unwrap(),
            eq(21)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(24)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(48));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(24));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn mono16_read() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        let mut buf = vec![0; 8];

        println!("");
        println!("1");

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![87, 49, 44, 49, 1, 49, 214, 48]));
        println!("2");

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![171, 48, 129, 48, 86, 48, 43, 48]));
        println!("3");

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42458));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21229));
        println!("4");

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![0, 0, 0, 0, 86, 48, 43, 48]));
    }

    // #[test]
    // #[should_panic(expected = "Invalid buffer length (7)")]
    // fn mono16_read_invalid_buffer_length() {
    //     let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
    //     let buf = std::io::BufReader::new(file);
    //     let mut decoder = OggDecoder::new(buf).unwrap();
    //     let mut buf = vec![0; 7];
    //     decoder.read(&mut buf).unwrap();
    // }

    // #[test]
    // fn mono16_read_to_end() {
    //     let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
    //     let buf = std::io::BufReader::new(file);
    //     let mut decoder = OggDecoder::new(buf).unwrap();
    //     decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
    //     let content = decoder.read_to_end().unwrap();
    //     expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    // }

    // #[test]
    // fn mono16_read_all() {
    //     let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
    //     let buf = std::io::BufReader::new(file);
    //     let mut decoder = OggDecoder::new(buf).unwrap();
    //     decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
    //     let content = decoder.read_all().unwrap();
    //     expect_that!(&content.len(), eq(decoder.byte_count()));
    // }

    #[test]
    fn stereo16_loading() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = OggDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Stereo16));
        expect_that!(&decoder.byte_count(), eq(21231 * 4));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100 * 4));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn stereo16_byte_seek() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(12)).unwrap(),
            eq(12)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(4)).unwrap(),
            eq(16)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-8)).unwrap(),
            eq(8)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-12)).unwrap(),
            eq(84912)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84912));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(84924)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84924));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    #[should_panic(expected = "Invalid seek offset (3)")]
    fn stereo16_byte_seek_invalid_offset() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(3)).unwrap();
    }

    #[test]
    fn stereo16_sample_seek() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(3)).unwrap(),
            eq(3)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(1)).unwrap(),
            eq(4)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-2)).unwrap(),
            eq(2)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-3)).unwrap(),
            eq(21228)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84912));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84924));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn stereo16_read() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        let mut buf = vec![0; 8];

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![227, 34, 227, 34, 197, 34, 197, 34]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![166, 34, 166, 34, 136, 34, 136, 34]));

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84920));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21230));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![0, 0, 0, 0, 136, 34, 136, 34]));
    }

    #[test]
    #[should_panic(expected = "Invalid buffer length (7)")]
    fn stereo16_read_invalid_buffer_length() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];
        decoder.read(&mut buf).unwrap();
    }

    #[test]
    fn stereo16_read_to_end() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    }

    #[test]
    fn stereo16_read_all() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }
}