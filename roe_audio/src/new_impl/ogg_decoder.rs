use lewton::samples::Samples;
use ogg::reading::PacketReader;

use super::{AudioFormat, Decoder};

struct OggContext {
    ident_header: lewton::header::IdentHeader,
    comment_header: lewton::header::CommentHeader,
    setup_header: lewton::header::SetupHeader,
    previous_window_right: lewton::audio::PreviousWindowRight,
    cur_absgp: Option<u64>,
    stream_serial: u32,
}

impl OggContext {
    fn new<T: std::io::Read + std::io::Seek>(
        packet_reader: &mut PacketReader<T>,
    ) -> Result<Self, DecoderError> {
        packet_reader.seek_bytes(std::io::SeekFrom::Start(0))?;
        let ((ident_header, comment_header, setup_header), stream_serial) =
            lewton::inside_ogg::read_headers(packet_reader)?;

        Ok(OggContext {
            ident_header,
            comment_header,
            setup_header,
            previous_window_right: lewton::audio::PreviousWindowRight::new(),
            cur_absgp: None,
            stream_serial,
        })
    }

    fn decode_ogg_packet_generic<S: Samples>(
        &mut self,
        packet: &ogg::Packet,
    ) -> Result<S, DecoderError> {
        Ok(lewton::audio::read_audio_packet_generic(
            &mut self.ident_header,
            &mut self.setup_header,
            &packet.data,
            &mut self.previous_window_right,
        )?)
    }

    fn decode_ogg_packet(&mut self, packet: &ogg::Packet) -> Result<Vec<Vec<i16>>, DecoderError> {
        self.decode_ogg_packet_generic(packet)
    }

    fn read_next_ogg_packet<T: std::io::Read + std::io::Seek>(
        &mut self,
        packet_reader: &mut PacketReader<T>,
    ) -> Result<Option<ogg::Packet>, DecoderError> {
        loop {
            let packet = match packet_reader.read_packet()? {
                Some(p) => p,
                None => return Ok(None),
            };

            if packet.stream_serial() == self.stream_serial {
                return Ok(Some(packet));
            }

            if packet.first_in_stream() {
                // Re-initialize context.
                let ident_header = lewton::header::read_header_ident(&packet.data)?;

                let packet = packet_reader.read_packet_expected()?;
                let comment_header = lewton::header::read_header_comment(&packet.data)?;

                let packet = packet_reader.read_packet_expected()?;
                let setup_header = lewton::header::read_header_setup(
                    &packet.data,
                    ident_header.audio_channels,
                    (ident_header.blocksize_0, ident_header.blocksize_1),
                )?;

                self.ident_header = ident_header;
                self.comment_header = comment_header;
                self.setup_header = setup_header;
                self.previous_window_right = lewton::audio::PreviousWindowRight::new();
                self.cur_absgp = None;
                self.stream_serial = packet.stream_serial();

                // Read the first data packet to initialize the previous_window_right.
                let packet = match packet_reader.read_packet()? {
                    Some(p) => p,
                    None => return Ok(None),
                };
                self.decode_ogg_packet(&packet)?;
                self.cur_absgp = Some(packet.absgp_page());

                return Ok(packet_reader.read_packet()?);
            }
        }
    }

    fn read_next_decoded_packet_generic<T: std::io::Read + std::io::Seek, S: Samples>(
        &mut self,
        packet_reader: &mut PacketReader<T>,
    ) -> Result<Option<S>, DecoderError> {
        let packet = match self.read_next_ogg_packet(packet_reader)? {
            Some(p) => p,
            None => return Ok(None),
        };
        let mut decoded_packet: S = self.decode_ogg_packet_generic(&packet)?;

        // If this is the last packet in the logical bitstream, it has to be truncated so
        // that the end matches the absgp of the current page.
        if let (Some(absgp), true) = (self.cur_absgp, packet.last_in_stream()) {
            let target_length = packet.absgp_page().saturating_sub(absgp) as usize;
            decoded_packet.truncate(target_length);
        }

        if packet.last_in_page() {
            self.cur_absgp = Some(packet.absgp_page());
        } else if let &mut Some(ref mut absgp) = &mut self.cur_absgp {
            *absgp += decoded_packet.num_samples() as u64;
        }

        Ok(Some(decoded_packet))
    }

    fn read_next_decoded_packet<T: std::io::Read + std::io::Seek>(
        &mut self,
        packet_reader: &mut PacketReader<T>,
    ) -> Result<Option<Vec<Vec<i16>>>, DecoderError> {
        self.read_next_decoded_packet_generic(packet_reader)
    }

    fn read_next_decoded_packet_interleaved<T: std::io::Read + std::io::Seek>(
        &mut self,
        packet_reader: &mut PacketReader<T>,
    ) -> Result<Option<Vec<i16>>, DecoderError> {
        let decoded_packet = match self
            .read_next_decoded_packet_generic::<T, lewton::samples::InterleavedSamples<_>>(
                packet_reader,
            )? {
            Some(p) => p,
            None => return Ok(None),
        };
        Ok(Some(decoded_packet.samples))
    }
}

pub struct OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    packet_reader: PacketReader<T>,
    context: OggContext,
    format: AudioFormat,
    sample_rate: u32,
    sample_count: usize,
    packet: Option<Vec<i16>>,
    packet_start_byte_pos: u64,
    packet_current_byte_pos: u64,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, DecoderError> {
        let mut packet_reader = PacketReader::new(input);
        let sample_count = Self::compute_sample_count(&mut packet_reader)?;
        let context = OggContext::new(&mut packet_reader)?;
        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(context.ident_header.audio_channels as u32, BYTES_PER_SAMPLE);
        let sample_rate = context.ident_header.audio_sample_rate;
        Ok(Self {
            packet_reader,
            context,
            format,
            sample_rate,
            sample_count,
            packet: None,
            packet_start_byte_pos: 0,
            packet_current_byte_pos: 0,
        })
    }

    fn compute_sample_count(packet_reader: &mut PacketReader<T>) -> Result<usize, DecoderError> {
        let mut sample_count = 0;
        let mut context = OggContext::new(packet_reader)?;
        loop {
            let packet = match context.read_next_decoded_packet(packet_reader)? {
                Some(p) => p,
                None => break,
            };

            for channel in packet {
                sample_count += channel.len();
            }
        }
        Ok(sample_count)
    }

    fn reset_to_stream_begin(&mut self) -> Result<(), DecoderError> {
        self.context = OggContext::new(&mut self.packet_reader)?;
        self.packet = None;
        self.packet_start_byte_pos = 0;
        self.packet_current_byte_pos = 0;
        Ok(())
    }

    fn read_next_packet(&mut self) -> Result<(), DecoderError> {
        if let Some(p) = &self.packet {
            self.packet_start_byte_pos +=
                p.len() as u64 * self.audio_format().bytes_per_sample() as u64;
        }
        self.packet = self
            .context
            .read_next_decoded_packet_interleaved(&mut self.packet_reader)?;
        self.packet_current_byte_pos = 0;
        Ok(())
    }
}

impl<T> Decoder for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
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

        // TODO: replace unwrap
        self.reset_to_stream_begin().unwrap();
        while self.packet_start_byte_pos < self.byte_count() as u64 {
            match &self.packet {
                Some(p) => {
                    let packet_end_byte_pos = self.packet_start_byte_pos
                        + p.len() as u64 * self.audio_format().bytes_per_sample() as u64;
                    if target_pos < packet_end_byte_pos {
                        self.packet_current_byte_pos = target_pos - self.packet_start_byte_pos;
                        break;
                    } else {
                        self.read_next_packet().unwrap();
                    }
                }
                None => self.read_next_packet().unwrap(),
            }
        }

        Ok(target_pos)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn byte_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.packet_start_byte_pos + self.packet_current_byte_pos)
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        let byte_stream_position = self.byte_stream_position()?;
        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(byte_stream_position % tbps == 0);
        Ok(byte_stream_position / tbps)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        assert!(
            buf.len() % tbps == 0,
            "Invalid buffer length ({})",
            buf.len()
        );

        if let None = self.packet {
            self.read_next_packet().unwrap();
        }

        let mut read_byte_count = 0;
        while read_byte_count < buf.len() {
            match &self.packet {
                Some(p) => {
                    let byte_data = bytemuck::cast_slice(p.as_slice());
                    let byte_to_read_count = std::cmp::min(
                        byte_data.len() - self.packet_current_byte_pos as usize,
                        buf.len() - read_byte_count,
                    );

                    let in_range = self.packet_current_byte_pos as usize
                        ..self.packet_current_byte_pos as usize + byte_to_read_count;
                    let out_range = read_byte_count..read_byte_count + byte_to_read_count;
                    buf[out_range].clone_from_slice(&byte_data[in_range]);
                    read_byte_count += byte_to_read_count;
                    self.packet_current_byte_pos += byte_to_read_count as u64;
                    if self.packet_current_byte_pos == byte_data.len() as u64 {
                        self.read_next_packet().unwrap();
                    }
                }
                None => break,
            }
        }

        Ok(read_byte_count)
    }
}

#[derive(Debug)]
pub enum DecoderError {
    IoError(std::io::Error),
    InvalidEncoding(String),
    InvalidHeader(String),
    InvalidData(String),
    Unimplemented,
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::InvalidEncoding(e) => write!(f, "Invalid encoding ({})", e),
            Self::InvalidHeader(e) => write!(f, "Invalid header ({})", e),
            Self::InvalidData(e) => write!(f, "Invalid data ({})", e),
            Self::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl std::error::Error for DecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DecoderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<lewton::VorbisError> for DecoderError {
    fn from(e: lewton::VorbisError) -> Self {
        match e {
            lewton::VorbisError::BadAudio(e) => DecoderError::from(e),
            lewton::VorbisError::BadHeader(e) => DecoderError::from(e),
            lewton::VorbisError::OggError(e) => DecoderError::from(e),
        }
    }
}

impl From<lewton::audio::AudioReadError> for DecoderError {
    fn from(e: lewton::audio::AudioReadError) -> Self {
        DecoderError::InvalidData(format!("{}", e))
    }
}

impl From<lewton::header::HeaderReadError> for DecoderError {
    fn from(e: lewton::header::HeaderReadError) -> Self {
        match e {
            lewton::header::HeaderReadError::NotVorbisHeader => {
                DecoderError::InvalidEncoding(format!("{}", e))
            }
            _ => DecoderError::InvalidHeader(format!("{}", e)),
        }
    }
}

impl From<ogg::reading::OggReadError> for DecoderError {
    fn from(e: ogg::reading::OggReadError) -> Self {
        match e {
            ogg::reading::OggReadError::ReadError(e) => DecoderError::IoError(e),
            ogg::reading::OggReadError::NoCapturePatternFound => {
                DecoderError::InvalidEncoding(format!("{}", e))
            }
            _ => DecoderError::InvalidData(format!("{}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    // TODO: use DecoderError for wav decoder.
    // TODO: match expectations in wav test.
    #[test]
    fn invalid_input_file() {
        let file = std::fs::File::open("data/audio/not-an-audio-file.txt").unwrap();
        let buf = std::io::BufReader::new(file);
        let result = OggDecoder::new(buf);
        expect_that!(&result, is_variant!(Result::Err));
        if let Err(e) = result {
            expect_that!(&e, is_variant!(DecoderError::InvalidEncoding));
        }
    }

    #[test]
    fn mono16_loading() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = OggDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono16));
        expect_that!(&decoder.byte_count(), eq(22208 * 2));
        expect_that!(&decoder.sample_count(), eq(22208));
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
            eq(decoder.byte_count() as u64 - 12)
        );
        expect_that!(
            &decoder.byte_stream_position().unwrap(),
            eq(decoder.byte_count() as u64 - 12)
        );
        expect_that!(
            &decoder.sample_stream_position().unwrap(),
            eq(decoder.sample_count() as u64 - 6)
        );

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(decoder.byte_count() as u64)
        );
        expect_that!(
            &decoder.byte_stream_position().unwrap(),
            eq(decoder.byte_count() as u64)
        );
        expect_that!(
            &decoder.sample_stream_position().unwrap(),
            eq(decoder.sample_count() as u64)
        );

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
            eq(decoder.sample_count() as u64 - 3)
        );
        expect_that!(
            &decoder.byte_stream_position().unwrap(),
            eq(decoder.byte_count() as u64 - 6)
        );
        expect_that!(
            &decoder.sample_stream_position().unwrap(),
            eq(decoder.sample_count() as u64 - 3)
        );

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(decoder.sample_count() as u64)
        );
        expect_that!(
            &decoder.byte_stream_position().unwrap(),
            eq(decoder.byte_count() as u64)
        );
        expect_that!(
            &decoder.sample_stream_position().unwrap(),
            eq(decoder.sample_count() as u64)
        );

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

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![190, 47, 9, 50, 24, 44, 240, 45]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![219, 43, 179, 44, 39, 44, 155, 44]));

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(44412));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(22206));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![0, 0, 0, 0, 39, 44, 155, 44]));
    }

    #[test]
    #[should_panic(expected = "Invalid buffer length (7)")]
    fn mono16_read_invalid_buffer_length() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];
        decoder.read(&mut buf).unwrap();
    }

    #[test]
    fn mono16_read_to_end() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    }

    #[test]
    fn mono16_read_all() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = OggDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }
}
