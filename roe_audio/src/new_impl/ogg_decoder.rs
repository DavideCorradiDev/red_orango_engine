use ogg::reading::PacketReader;
use lewton::samples::Samples;

use super::{AudioFormat, Decoder};

pub struct OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    packet_reader: PacketReader<T>,
    ident_header: lewton::header::IdentHeader,
    comment_header: lewton::header::CommentHeader,
    setup_header: lewton::header::SetupHeader,
    previous_window_right: lewton::audio::PreviousWindowRight,
    cur_absgp: Option<u64>,
    stream_serial: u32,
    format: AudioFormat,
    sample_count: usize,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, DecoderError> {
        let mut packet_reader = PacketReader::new(input);

        let ((ident_header, comment_header, setup_header), stream_serial) =
            lewton::inside_ogg::read_headers(&mut packet_reader)?;

        let channel_count = ident_header.audio_channels as u32;
        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(channel_count, BYTES_PER_SAMPLE);

        Ok(Self {
            packet_reader,
            ident_header,
            comment_header,
            setup_header,
            previous_window_right: lewton::audio::PreviousWindowRight::new(),
            cur_absgp: None,
            stream_serial,
            format,
            sample_count: 0,
        })

        // let sample_count =
        //     Self::compute_sample_count(&ident_header, &setup_header, &mut packet_reader)?;
    }

    fn decode_ogg_packet(&mut self, packet: &ogg::Packet) -> Result<Vec<Vec<i16>>, DecoderError> {
        self.decode_ogg_packet_generic(packet)
    }

    fn decode_ogg_packet_generic<S: Samples>(&mut self, packet: &ogg::Packet) -> Result<S, DecoderError> {
        Ok(lewton::audio::read_audio_packet_generic(
            &mut self.ident_header,
            &mut self.setup_header,
            &packet.data,
            &mut self.previous_window_right,
        )?)
    }

    fn read_next_ogg_packet(&mut self) -> Result<Option<ogg::Packet>, DecoderError> {
        loop {
            let packet = match self.packet_reader.read_packet()? {
                Some(p) => p,
                None => return Ok(None),
            };

            if packet.stream_serial() == self.stream_serial {
                return Ok(Some(packet));
            }

            if packet.first_in_stream() {
                // Re-initialize headers.
                let ident_header = lewton::header::read_header_ident(&packet.data)?;

                let packet = self.packet_reader.read_packet_expected()?;
                let comment_header = lewton::header::read_header_comment(&packet.data)?;

                let packet = self.packet_reader.read_packet_expected()?;
                let setup_header = lewton::header::read_header_setup(
                    &packet.data,
                    ident_header.audio_channels,
                    (ident_header.blocksize_0, ident_header.blocksize_1),
                )?;

                self.previous_window_right = lewton::audio::PreviousWindowRight::new();
                self.cur_absgp = None;
                self.ident_header = ident_header;
                self.comment_header = comment_header;
                self.setup_header = setup_header;
                self.stream_serial = packet.stream_serial();

                // Read the first data packet to initialize the previous_window_right.
                let packet = match self.packet_reader.read_packet()? {
                    Some(p) => p,
                    None => return Ok(None),
                };
                self.decode_ogg_packet(&packet)?;
                self.cur_absgp = Some(packet.absgp_page());

                return Ok(self.packet_reader.read_packet()?);
            }
        }
    }

    fn read_next_decoded_packet_generic<S: Samples>(&mut self) -> Result<Option<S>, DecoderError> {
        let packet = match self.read_next_ogg_packet()? {
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
        }
        else if let &mut Some(ref mut absgp) = &mut self.cur_absgp {
            *absgp += decoded_packet.num_samples() as u64;
        }

        Ok(Some(decoded_packet))
    }

    fn read_next_decoded_packet(&mut self) -> Result<Option<Vec<Vec<i16>>>, DecoderError>
    {
        self.read_next_decoded_packet()
    }

    fn read_next_decoded_packet_interleaved(&mut self) -> Result<Option<Vec<i16>>, DecoderError>
    {
        let decoded_packet = match self.read_next_decoded_packet_generic::<lewton::samples::InterleavedSamples<_>>()? {
            Some(p) => p,
            None => return Ok(None),
        };
        Ok(Some(decoded_packet.samples))
    }

    // fn compute_sample_count(&mut self) -> Result<usize, DecoderError> {
    //     let mut maybe_packet = packet_reader.read_packet()?;
    //     let mut sample_count = 0;
    //     let mut previous_window_right = lewton::audio::PreviousWindowRight::new();
    //     while maybe_packet.is_some() {
    //         let packet = maybe_packet.unwrap();
    //         let decoded_data = lewton::audio::read_audio_packet(
    //             ident_header,
    //             setup_header,
    //             &packet.data,
    //             &mut previous_window_right,
    //         )?;
    //         for channel in decoded_data {
    //             sample_count += channel.len()
    //         }
    //         maybe_packet = packet_reader.read_packet()?;
    //     }

    //     // Divide by the number of channels.
    //     let channel_count = ident_header.audio_channels as usize;
    //     if sample_count % channel_count != 0 {
    //         return Err(DecoderError::InvalidData(String::from(
    //             "The number of samples is not a multiple of the number of channels",
    //         )));
    //     }
    //     sample_count /= channel_count;

    //     // Reset to start of packet reader and re-read headers to make sure to be at the right offset.
    //     packet_reader.seek_bytes(std::io::SeekFrom::Start(0))?;
    //     lewton::inside_ogg::read_headers(packet_reader)?;

    //     Ok(sample_count)
    // }
}

impl<T> Decoder for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }

    fn sample_rate(&self) -> u32 {
        self.ident_header.audio_sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(0)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
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
        expect_that!(&decoder.byte_count(), eq(22103 * 2));
        expect_that!(&decoder.sample_count(), eq(22103));
        expect_that!(&decoder.byte_rate(), eq(44100 * 2));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }
}
