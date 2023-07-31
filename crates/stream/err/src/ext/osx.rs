use super::Error;

// TODO: consider getting rid of unnecessary cast
impl From<Error> for coreaudio::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::ChannelIsEmpty(_) => coreaudio::Error::Unspecified,
            Error::ChannelIsFull(_) => coreaudio::Error::Unspecified,
            Error::ChannelIsClosed(_) => coreaudio::Error::Unspecified,
            _ => unimplemented!(),
        }
    }
}

impl From<coreaudio::Error> for Error<String> {
    fn from(value: coreaudio::Error) -> Self {
        match value {
            coreaudio::Error::Unspecified => Error::Unspecified,
            coreaudio::Error::SystemSoundClientMessageTimedOut => {
                Error::SystemSoundClientMessageTimedOut
            }
            coreaudio::Error::NoMatchingDefaultAudioUnitFound => {
                Error::NoMatchingDefaultAudioUnitFound
            }
            coreaudio::Error::RenderCallbackBufferFormatDoesNotMatchAudioUnitStreamFormat => {
                Error::RenderCallbackBufferFormatDoesNotMatchAudioUnitStreamFormat
            }
            coreaudio::Error::NoKnownSubtype => Error::NoKnownSubtype,
            coreaudio::Error::NonInterleavedInputOnlySupportsMono => {
                Error::NonInterleavedInputOnlySupportsMono
            }
            coreaudio::Error::UnsupportedSampleRate => Error::UnsupportedSampleRate,
            coreaudio::Error::UnsupportedStreamFormat => Error::UnsupportedStreamFormat,
            coreaudio::Error::Audio(e) => Error::Audio(e.to_string()),
            coreaudio::Error::AudioCodec(e) => Error::AudioCodec(e.to_string()),
            coreaudio::Error::AudioFormat(e) => Error::AudioFormat(e.to_string()),
            coreaudio::Error::AudioUnit(e) => Error::AudioUnit(e.to_string()),
            coreaudio::Error::Unknown(e) => Error::Unknown(e.to_string()),
        }
    }
}
