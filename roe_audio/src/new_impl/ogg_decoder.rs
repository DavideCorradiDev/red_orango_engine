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

fn decode_ogg_packet_generic<S: Samples>(
    packet: &ogg::Packet,
    context: &mut OggContext,
) -> Result<S, DecoderError> {
    Ok(lewton::audio::read_audio_packet_generic(
        &mut context.ident_header,
        &mut context.setup_header,
        &packet.data,
        &mut context.previous_window_right,
    )?)
}

fn decode_ogg_packet(
    packet: &ogg::Packet,
    context: &mut OggContext,
) -> Result<Vec<Vec<i16>>, DecoderError> {
    decode_ogg_packet_generic(packet, context)
}

fn read_next_ogg_packet<T: std::io::Read + std::io::Seek>(
    packet_reader: &mut PacketReader<T>,
    context: &mut OggContext,
) -> Result<Option<ogg::Packet>, DecoderError> {
    println!("READING OGG PACKET");
    loop {
        println!("READING PACKET");
        let packet = match packet_reader.read_packet()? {
            Some(p) => p,
            None => return Ok(None),
        };

        println!("CHECKING IF SERIAL MATCHES");
        if packet.stream_serial() == context.stream_serial {
            println!("IT MATCHES");
            return Ok(Some(packet));
        }

        println!("CHECKING IF THE PACKET IS FIRST IN STREAM");
        if packet.first_in_stream() {
            println!("READING HEADER 1");
            // Re-initialize headers.
            let ident_header = lewton::header::read_header_ident(&packet.data)?;

            println!("READING HEADER 2");
            let packet = packet_reader.read_packet_expected()?;
            let comment_header = lewton::header::read_header_comment(&packet.data)?;

            println!("READING HEADER 3");
            let packet = packet_reader.read_packet_expected()?;
            let setup_header = lewton::header::read_header_setup(
                &packet.data,
                ident_header.audio_channels,
                (ident_header.blocksize_0, ident_header.blocksize_1),
            )?;

            println!("CONTEXT UPDATE");
            context.ident_header = ident_header;
            context.comment_header = comment_header;
            context.setup_header = setup_header;
            context.previous_window_right = lewton::audio::PreviousWindowRight::new();
            context.cur_absgp = None;
            context.stream_serial = packet.stream_serial();

            // Read the first data packet to initialize the previous_window_right.
            println!("READ NEW PACKET");
            let packet = match packet_reader.read_packet()? {
                Some(p) => p,
                None => return Ok(None),
            };
            println!("DECODE_PACKET");
            decode_ogg_packet(&packet, context)?;
            println!("UPDATE ABSGP");
            context.cur_absgp = Some(packet.absgp_page());

            println!("DONE!");
            return Ok(packet_reader.read_packet()?);
        }
    }
}

fn read_next_decoded_packet_generic<T: std::io::Read + std::io::Seek, S: Samples>(
    packet_reader: &mut PacketReader<T>,
    context: &mut OggContext,
) -> Result<Option<S>, DecoderError> {
    let packet = match read_next_ogg_packet(packet_reader, context)? {
        Some(p) => p,
        None => return Ok(None),
    };
    let mut decoded_packet: S = decode_ogg_packet_generic(&packet, context)?;

    // If this is the last packet in the logical bitstream, it has to be truncated so
    // that the end matches the absgp of the current page.
    if let (Some(absgp), true) = (context.cur_absgp, packet.last_in_stream()) {
        let target_length = packet.absgp_page().saturating_sub(absgp) as usize;
        decoded_packet.truncate(target_length);
    }

    if packet.last_in_page() {
        context.cur_absgp = Some(packet.absgp_page());
    } else if let &mut Some(ref mut absgp) = &mut context.cur_absgp {
        *absgp += decoded_packet.num_samples() as u64;
    }

    Ok(Some(decoded_packet))
}

fn read_next_decoded_packet<T: std::io::Read + std::io::Seek>(
    packet_reader: &mut PacketReader<T>,
    context: &mut OggContext,
) -> Result<Option<Vec<Vec<i16>>>, DecoderError> {
    read_next_decoded_packet_generic(packet_reader, context)
}

fn read_next_decoded_packet_interleaved<T: std::io::Read + std::io::Seek>(
    packet_reader: &mut PacketReader<T>,
    context: &mut OggContext,
) -> Result<Option<Vec<i16>>, DecoderError> {
    let decoded_packet = match read_next_decoded_packet_generic::<
        T,
        lewton::samples::InterleavedSamples<_>,
    >(packet_reader, context)?
    {
        Some(p) => p,
        None => return Ok(None),
    };
    Ok(Some(decoded_packet.samples))
}

fn compute_sample_count<T: std::io::Read + std::io::Seek>(
    packet_reader: &mut PacketReader<T>
) -> Result<usize, DecoderError> {
    let mut sample_count = 0;
    println!("--SEEKING TO START");
    packet_reader.seek_bytes(std::io::SeekFrom::Start(0))?;
    println!("Creating context");
    let ((ident_header, comment_header, setup_header), stream_serial) =
        lewton::inside_ogg::read_headers(packet_reader)?;

    let mut context = OggContext {
        ident_header,
        comment_header,
        setup_header,
        previous_window_right: lewton::audio::PreviousWindowRight::new(),
        cur_absgp: None,
        stream_serial,
    };
    println!("--STARTING LOOP");
    loop {
        println!("--READING PACKET");
        let packet = match read_next_decoded_packet(packet_reader, &mut context)? {
            Some(p) => p,
            None => break,
        };

        println!("--COMPUTING NUMBER OF SAMPLES IN PACKET.");
        for channel in packet {
            sample_count += channel.len();
        }
    }
    println!("--SEEKING TO START AGAIN");
    packet_reader.seek_bytes(std::io::SeekFrom::Start(0))?;
    println!("--COMPUTATION DONE!");
    Ok(sample_count)
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
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, DecoderError> {
        let mut packet_reader = PacketReader::new(input);
        let sample_count = compute_sample_count(&mut packet_reader)?;

        let ((ident_header, comment_header, setup_header), stream_serial) =
            lewton::inside_ogg::read_headers(&mut packet_reader)?;

        let mut context = OggContext {
            ident_header,
            comment_header,
            setup_header,
            previous_window_right: lewton::audio::PreviousWindowRight::new(),
            cur_absgp: None,
            stream_serial,
        };

        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(context.ident_header.audio_channels as u32, BYTES_PER_SAMPLE);

        let sample_rate = context.ident_header.audio_sample_rate;

        Ok(Self {
            packet_reader,
            context,
            format,
            sample_rate,
            sample_count,
        })
    }

    fn read_next_decoded_packet(&mut self) -> Result<Option<Vec<Vec<i16>>>, DecoderError> {
        read_next_decoded_packet(&mut self.packet_reader, &mut self.context)
    }

    fn read_next_decoded_packet_interleaved(&mut self) -> Result<Option<Vec<i16>>, DecoderError> {
        read_next_decoded_packet_interleaved(&mut self.packet_reader, &mut self.context)
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
        Ok(0)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
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
