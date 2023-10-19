use std::{
    borrow::Cow,
    io::{Read, Write},
    str::FromStr,
    string::ToString, vec,
};

use crate::{
    cast::proxies,
    errors::Error,
    message_manager::{CastMessage, CastMessagePayload, MessageManager},
    Lrc,
};

const CHANNEL_NAMESPACE: &str = "urn:x-cast:com.google.cast.media";

const MESSAGE_TYPE_GET_STATUS: &str = "GET_STATUS";
const MESSAGE_TYPE_LOAD: &str = "LOAD";
const MESSAGE_TYPE_PLAY: &str = "PLAY";
const MESSAGE_TYPE_PAUSE: &str = "PAUSE";
const MESSAGE_TYPE_STOP: &str = "STOP";
const MESSAGE_TYPE_SEEK: &str = "SEEK";
const MESSAGE_TYPE_MEDIA_STATUS: &str = "MEDIA_STATUS";
const MESSAGE_TYPE_LOAD_CANCELLED: &str = "LOAD_CANCELLED";
const MESSAGE_TYPE_LOAD_FAILED: &str = "LOAD_FAILED";
const MESSAGE_TYPE_INVALID_PLAYER_STATE: &str = "INVALID_PLAYER_STATE";
const MESSAGE_TYPE_INVALID_REQUEST: &str = "INVALID_REQUEST";

/// Describes the way cast device should stream content.
#[derive(Copy, Clone, Debug)]
pub enum StreamType {
    /// This variant allows cast device to automatically choose whatever way it's most comfortable
    /// with.
    None,
    /// Cast device should buffer some portion of the content and only then start streaming.
    Buffered,
    /// Cast device should display content as soon as it gets any portion of it.
    Live,
}

impl FromStr for StreamType {
    type Err = Error;

    fn from_str(s: &str) -> Result<StreamType, Error> {
        match s {
            "BUFFERED" | "buffered" => Ok(StreamType::Buffered),
            "LIVE" | "live" => Ok(StreamType::Live),
            _ => Ok(StreamType::None),
        }
    }
}

impl ToString for StreamType {
    fn to_string(&self) -> String {
        let stream_type = match *self {
            StreamType::None => "NONE",
            StreamType::Buffered => "BUFFERED",
            StreamType::Live => "LIVE",
        };

        stream_type.to_string()
    }
}

/// Generic, movie, TV show, music track, or photo metadata.
#[derive(Clone, Debug)]
pub enum Metadata {
    Generic(GenericMediaMetadata),
    Movie(MovieMediaMetadata),
    TvShow(TvShowMediaMetadata),
    MusicTrack(MusicTrackMediaMetadata),
    Photo(PhotoMediaMetadata),
}

/// Generic media metadata.
///
/// See also the [`GenericMediaMetadata` Cast reference](https://developers.google.com/cast/docs/reference/messages#GenericMediaMetadata).
#[derive(Clone, Debug)]
pub struct GenericMediaMetadata {
    /// Descriptive title of the content.
    pub title: Option<String>,
    /// Descriptive subtitle of the content.
    pub subtitle: Option<String>,
    /// Zero or more URLs to an image associated with the content.
    pub images: Vec<Image>,
    /// Date and time the content was released, formatted as ISO 8601.
    pub release_date: Option<String>,
}

/// Movie media metadata.
///
/// See also the [`MovieMediaMetadata` Cast reference](https://developers.google.com/cast/docs/reference/messages#MovieMediaMetadata).
#[derive(Clone, Debug)]
pub struct MovieMediaMetadata {
    /// Title of the movie.
    pub title: Option<String>,
    /// Subtitle of the movie.
    pub subtitle: Option<String>,
    /// Studio which released the movie.
    pub studio: Option<String>,
    /// Zero or more URLs to an image associated with the content.
    pub images: Vec<Image>,
    /// Date and time the movie was released, formatted as ISO 8601.
    pub release_date: Option<String>,
}

/// TV show media metadata.
///
/// See also the [`TvShowMediaMetadata` Cast reference](https://developers.google.com/cast/docs/reference/messages#TvShowMediaMetadata).
#[derive(Clone, Debug)]
pub struct TvShowMediaMetadata {
    /// Title of the TV series.
    pub series_title: Option<String>,
    /// Title of the episode.
    pub episode_title: Option<String>,
    /// Season number of the TV show.
    pub season: Option<u32>,
    /// Episode number (in the season) of the episode.
    pub episode: Option<u32>,
    /// Zero or more URLs to an image associated with the content.
    pub images: Vec<Image>,
    /// Date and time this episode was released, formatted as ISO 8601.
    pub original_air_date: Option<String>,
}

/// Music track media metadata.
///
/// See also the [`MusicTrackMediaMetadata` Cast reference](https://developers.google.com/cast/docs/reference/messages#MusicTrackMediaMetadata).
#[derive(Clone, Debug)]
pub struct MusicTrackMediaMetadata {
    /// Album or collection from which the track is taken.
    pub album_name: Option<String>,
    /// Name of the track (for example, song title).
    pub title: Option<String>,
    /// Name of the artist associated with the album featuring this track.
    pub album_artist: Option<String>,
    /// Name of the artist associated with the track.
    pub artist: Option<String>,
    /// Name of the composer associated with the track.
    pub composer: Option<String>,
    /// Number of the track on the album.
    pub track_number: Option<u32>,
    /// Number of the volume (for example, a disc) of the album.
    pub disc_number: Option<u32>,
    /// Zero or more URLs to an image associated with the content.
    pub images: Vec<Image>,
    /// Date and time the content was released, formatted as ISO 8601.
    pub release_date: Option<String>,
}

/// Photo media metadata.
///
/// See also the [`PhotoMediaMetadata` Cast reference](https://developers.google.com/cast/docs/reference/messages#PhotoMediaMetadata).
#[derive(Clone, Debug)]
pub struct PhotoMediaMetadata {
    /// Title of the photograph.
    pub title: Option<String>,
    /// Name of the photographer.
    pub artist: Option<String>,
    /// Verbal location where the photograph was taken, for example “Madrid, Spain”.
    pub location: Option<String>,
    /// Latitude and longitude of the location where the photograph was taken.
    pub latitude_longitude: Option<(f64, f64)>,
    /// Width and height of the photograph in pixels.
    pub dimensions: Option<(u32, u32)>,
    /// Date and time the photograph was taken, formatted as ISO 8601.
    pub creation_date_time: Option<String>,
}

/// Image URL and optionally size metadata.
///
/// This is the description of an image, including a small amount of metadata to
/// allow the sender application a choice of images, depending on how it will
/// render them. The height and width are optional on only one item in an array
/// of images.
///
/// See also the [`Image` Cast reference](https://developers.google.com/cast/docs/reference/messages#Image).
#[derive(Clone, Debug)]
pub struct Image {
    /// URL of the image.
    pub url: String,
    /// Width and height of the image.
    pub dimensions: Option<(u32, u32)>,
}

impl Image {
    pub fn new(url: String) -> Image {
        Image {
            url,
            dimensions: None,
        }
    }

    fn encode(&self) -> proxies::media::Image {
        proxies::media::Image {
            url: self.url.clone(),
            width: self.dimensions.map(|d| d.0),
            height: self.dimensions.map(|d| d.1),
        }
    }
}

/// Describes possible player states.
#[derive(Copy, Clone, Debug)]
pub enum PlayerState {
    /// Player has not been loaded yet.
    Idle,
    /// Player is actively playing content.
    Playing,
    /// Player is in PLAY mode but not actively playing content (currentTime is not changing).
    Buffering,
    /// Player is paused.
    Paused,
}

impl FromStr for PlayerState {
    type Err = Error;

    fn from_str(s: &str) -> Result<PlayerState, Error> {
        match s {
            "IDLE" => Ok(PlayerState::Idle),
            "PLAYING" => Ok(PlayerState::Playing),
            "BUFFERING" => Ok(PlayerState::Buffering),
            "PAUSED" => Ok(PlayerState::Paused),
            _ => Err(Error::Internal(format!("Unknown player state {}", s))),
        }
    }
}

impl ToString for PlayerState {
    fn to_string(&self) -> String {
        let player_state = match *self {
            PlayerState::Idle => "IDLE",
            PlayerState::Playing => "PLAYING",
            PlayerState::Buffering => "BUFFERING",
            PlayerState::Paused => "PAUSED",
        };

        player_state.to_string()
    }
}

/// Describes possible player idle reasons.
#[derive(Copy, Clone, Debug)]
pub enum IdleReason {
    /// A sender requested to stop playback using the STOP command.
    Cancelled,
    /// A sender requested playing a different media using the LOAD command.
    Interrupted,
    /// The media playback completed.
    Finished,
    /// The media was interrupted due to an error; For example, if the player could not download the
    /// media due to network issues.
    Error,
}

impl FromStr for IdleReason {
    type Err = Error;

    fn from_str(s: &str) -> Result<IdleReason, Error> {
        match s {
            "CANCELLED" => Ok(IdleReason::Cancelled),
            "INTERRUPTED" => Ok(IdleReason::Interrupted),
            "FINISHED" => Ok(IdleReason::Finished),
            "ERROR" => Ok(IdleReason::Error),
            _ => Err(Error::Internal(format!("Unknown idle reason {}", s))),
        }
    }
}

/// Describes the operation to perform with playback while seeking.
#[derive(Copy, Clone, Debug)]
pub enum ResumeState {
    /// Forces media to start.
    PlaybackStart,
    /// Forces media to pause.
    PlaybackPause,
}

impl FromStr for ResumeState {
    type Err = Error;

    fn from_str(s: &str) -> Result<ResumeState, Error> {
        match s {
            "PLAYBACK_START" | "start" => Ok(ResumeState::PlaybackStart),
            "PLAYBACK_PAUSE" | "pause" => Ok(ResumeState::PlaybackPause),
            _ => Err(Error::Internal(format!("Unknown resume state {}", s))),
        }
    }
}

impl ToString for ResumeState {
    fn to_string(&self) -> String {
        let resume_state = match *self {
            ResumeState::PlaybackStart => "PLAYBACK_START",
            ResumeState::PlaybackPause => "PLAYBACK_PAUSE",
        };

        resume_state.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub track_id: u16,
    pub type_: String,
    pub track_content_id: String,
    pub track_content_type: String,
    pub name: String,
    pub language: String,
    pub subtype: String
} 

/// This data structure describes a media stream.
#[derive(Clone, Debug)]
pub struct Media {
    /// Service-specific identifier of the content currently loaded by the media player. This is a
    /// free form string and is specific to the application. In most cases, this will be the URL to
    /// the media, but the sender can choose to pass a string that the receiver can interpret
    /// properly. Max length: 1k.
    pub content_id: String,
    /// Describes the type of media artifact.
    pub stream_type: StreamType,
    /// MIME content type of the media being played.
    pub content_type: String,
    /// Generic, movie, TV show, music track, or photo metadata.
    pub metadata: Option<Metadata>,
    /// Duration of the currently playing stream in seconds.
    pub duration: Option<f32>,
    /// Describes the attached tracks such as subtitles
    pub tracks: Vec<proxies::media::Track>
}

/// Describes the current status of the media artifact with respect to the session.
#[derive(Clone, Debug)]
pub struct Status {
    /// Unique id of the request that requested the status.
    pub request_id: i32,
    /// Detailed status of every media status entry.
    pub entries: Vec<StatusEntry>,
}

/// Detailed status of the media artifact with respect to the session.
#[derive(Clone, Debug)]
pub struct StatusEntry {
    /// Unique ID for the playback of this specific session. This ID is set by the receiver at LOAD
    /// and can be used to identify a specific instance of a playback. For example, two playbacks of
    /// "Wish you were here" within the same session would each have a unique mediaSessionId.
    pub media_session_id: i32,
    /// Full description of the content that is being played back. Only be returned in a status
    /// messages if the Media has changed.
    pub media: Option<Media>,
    /// Indicates whether the media time is progressing, and at what rate. This is independent of
    /// the player state since the media time can stop in any state. 1.0 is regular time, 0.5 is
    /// slow motion.
    pub playback_rate: f32,
    /// Describes the state of the player.
    pub player_state: PlayerState,
    /// If the player_state is IDLE and the reason it became IDLE is known, this property is
    /// provided. If the player is IDLE because it just started, this property will not be provided.
    /// If the player is in any other state this property should not be provided.
    pub idle_reason: Option<IdleReason>,
    /// The current position of the media player since the beginning of the content, in seconds.
    /// If this a live stream content, then this field represents the time in seconds from the
    /// beginning of the event that should be known to the player.
    pub current_time: Option<f32>,
    /// Flags describing which media commands the media player supports:
    /// * `1` `Pause`;
    /// * `2` `Seek`;
    /// * `4` `Stream volume`;
    /// * `8` `Stream mute`;
    /// * `16` `Skip forward`;
    /// * `32` `Skip backward`;
    /// * `1 << 12` `Unknown`;
    /// * `1 << 13` `Unknown`;
    /// * `1 << 18` `Unknown`.
    /// Combinations are described as summations; for example, Pause+Seek+StreamVolume+Mute == 15.
    pub supported_media_commands: u32,
}

/// Describes the load cancelled error.
#[derive(Copy, Clone, Debug)]
pub struct LoadCancelled {
    /// Unique id of the request that caused this error.
    pub request_id: i32,
}

/// Describes the load failed error.
#[derive(Copy, Clone, Debug)]
pub struct LoadFailed {
    /// Unique id of the request that caused this error.
    pub request_id: i32,
}

/// Describes the invalid player state error.
#[derive(Copy, Clone, Debug)]
pub struct InvalidPlayerState {
    /// Unique id of the request that caused this error.
    pub request_id: i32,
}

/// Describes the invalid request error.
#[derive(Clone, Debug)]
pub struct InvalidRequest {
    /// Unique id of the invalid request.
    pub request_id: i32,
    /// Description of the invalid request reason if available.
    pub reason: Option<String>,
}

/// Represents all currently supported incoming messages that media channel can handle.
#[derive(Clone, Debug)]
pub enum MediaResponse {
    /// Statuses of the currently active media.
    Status(Status),
    /// Sent when the load request was cancelled (a second load request was received).
    LoadCancelled(LoadCancelled),
    /// Sent when the load request failed. The player state will be IDLE.
    LoadFailed(LoadFailed),
    /// Sent when the request by the sender can not be fulfilled because the player is not in a
    /// valid state. For example, if the application has not created a media element yet.
    InvalidPlayerState(InvalidPlayerState),
    /// Error indicating that request is not valid.
    InvalidRequest(InvalidRequest),
    /// Used every time when channel can't parse the message. Associated data contains `type` string
    /// field and raw JSON data returned from cast device.
    NotImplemented(String, serde_json::Value),
}

pub struct MediaChannel<'a, W>
where
    W: Read + Write,
{
    sender: Cow<'a, str>,
    message_manager: Lrc<MessageManager<W>>,
}

impl<'a, W> MediaChannel<'a, W>
where
    W: Read + Write,
{
    pub fn new<S>(sender: S, message_manager: Lrc<MessageManager<W>>) -> MediaChannel<'a, W>
    where
        S: Into<Cow<'a, str>>,
    {
        MediaChannel {
            sender: sender.into(),
            message_manager,
        }
    }

    /// Retrieves status of the cast device media session.
    ///
    /// # Arguments
    ///
    /// * `destination` - `protocol` identifier of specific app media session;
    /// * `media_session_id` - Media session ID of the media for which the media status should be
    /// returned. If none is provided, then the status for all media session IDs will be provided.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn get_status<S>(
        &self,
        destination: S,
        media_session_id: Option<i32>,
    ) -> Result<Status, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let payload = serde_json::to_string(&proxies::media::GetStatusRequest {
            typ: MESSAGE_TYPE_GET_STATUS.to_string(),
            request_id,
            media_session_id,
        })?;

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        self.message_manager.receive_find_map(|message| {
            if !self.can_handle(message) {
                return Ok(None);
            }

            match self.parse(message)? {
                MediaResponse::Status(status) => {
                    if status.request_id == request_id {
                        return Ok(Some(status));
                    }
                }
                MediaResponse::InvalidRequest(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(format!(
                            "Invalid request ({}).",
                            error.reason.unwrap_or_else(|| "Unknown".to_string())
                        )));
                    }
                }
                _ => {}
            }

            Ok(None)
        })
    }

    /// Loads provided media to the application.
    ///
    /// # Arguments
    /// * `destination` - `protocol` of the application to load media with (e.g. `web-1`);
    /// * `session_id` - Current session identifier of the player application;
    /// * `media` - `Media` instance that describes the media we'd like to load.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn load<S>(&self, destination: S, session_id: S, media: &Media) -> Result<Status, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let metadata = media.metadata.as_ref().map(|m| match *m {
            Metadata::Generic(ref x) => proxies::media::Metadata {
                title: x.title.clone(),
                subtitle: x.subtitle.clone(),
                images: x.images.iter().map(|i| i.encode()).collect(),
                release_date: x.release_date.clone(),
                ..proxies::media::Metadata::new(0)
            },
            Metadata::Movie(ref x) => proxies::media::Metadata {
                title: x.title.clone(),
                subtitle: x.subtitle.clone(),
                studio: x.studio.clone(),
                images: x.images.iter().map(|i| i.encode()).collect(),
                release_date: x.release_date.clone(),
                ..proxies::media::Metadata::new(1)
            },
            Metadata::TvShow(ref x) => proxies::media::Metadata {
                series_title: x.series_title.clone(),
                subtitle: x.episode_title.clone(),
                season: x.season,
                episode: x.episode,
                images: x.images.iter().map(|i| i.encode()).collect(),
                original_air_date: x.original_air_date.clone(),
                ..proxies::media::Metadata::new(2)
            },
            Metadata::MusicTrack(ref x) => proxies::media::Metadata {
                album_name: x.album_name.clone(),
                title: x.title.clone(),
                album_artist: x.album_artist.clone(),
                artist: x.artist.clone(),
                composer: x.composer.clone(),
                track_number: x.track_number,
                disc_number: x.disc_number,
                images: x.images.iter().map(|i| i.encode()).collect(),
                release_date: x.release_date.clone(),
                ..proxies::media::Metadata::new(3)
            },
            Metadata::Photo(ref x) => proxies::media::Metadata {
                title: x.title.clone(),
                artist: x.artist.clone(),
                location: x.location.clone(),
                latitude: x.latitude_longitude.map(|coord| coord.0),
                longitude: x.latitude_longitude.map(|coord| coord.1),
                width: x.dimensions.map(|dims| dims.0),
                height: x.dimensions.map(|dims| dims.1),
                creation_date_time: x.creation_date_time.clone(),
                ..proxies::media::Metadata::new(4)
            },
        });

        let active_track_ids = match media.tracks.first() {
            Some(track) => vec![track.track_id],
            None => vec![]
        };

        println!("{:?}", active_track_ids);

        let payload = serde_json::to_string(&proxies::media::MediaRequest {
            request_id,
            session_id: session_id.into().to_string(),
            typ: MESSAGE_TYPE_LOAD.to_string(),

            media: proxies::media::Media {
                content_id: media.content_id.clone(),
                stream_type: media.stream_type.to_string(),
                content_type: media.content_type.clone(),
                metadata,
                duration: media.duration,
                tracks: media.tracks.clone()
            },

            active_track_ids,

            current_time: 0_f64,
            autoplay: true,
            custom_data: proxies::media::CustomData::new(),
        })?;

        println!("payload: {}", payload);

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        // Once media is loaded cast receiver device should emit status update event, or load failed
        // event if something went wrong.
        self.message_manager.receive_find_map(|message| {
            if !self.can_handle(message) {
                return Ok(None);
            }

            match self.parse(message)? {
                MediaResponse::Status(status) => {
                    if status.request_id == request_id {
                        return Ok(Some(status));
                    }

                    // [WORKAROUND] In some cases we don't receive response (e.g. from YouTube app),
                    // so let's just wait for the response with the media we're interested in and
                    // return it.
                    let has_media = {
                        status.entries.iter().any(|entry| {
                            if let Some(ref loaded_media) = entry.media {
                                return loaded_media.content_id == media.content_id;
                            }

                            false
                        })
                    };

                    if has_media {
                        return Ok(Some(status));
                    }
                }
                MediaResponse::LoadFailed(error) => {
                    println!("{:?}", message);

                    if error.request_id == request_id {
                        return Err(Error::Internal("Failed to load media.".to_string()));
                    }
                }
                MediaResponse::LoadCancelled(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(
                            "Load cancelled by another request.".to_string(),
                        ));
                    }
                }
                MediaResponse::InvalidPlayerState(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(
                            "Load failed because of invalid player state.".to_string(),
                        ));
                    }
                }
                MediaResponse::InvalidRequest(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(format!(
                            "Load failed because of invalid media request (reason: {}).",
                            error.reason.unwrap_or_else(|| "UNKNOWN".to_string())
                        )));
                    }
                }
                _ => {}
            }

            Ok(None)
        })
    }

    /// Pauses playback of the current content. Triggers a STATUS event notification to all sender
    /// applications.
    ///
    /// # Arguments
    ///
    /// * `destination` - `protocol` of the media application (e.g. `web-1`);
    /// * `media_session_id` - ID of the media session to be paused.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn pause<S>(&self, destination: S, media_session_id: i32) -> Result<StatusEntry, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let payload = serde_json::to_string(&proxies::media::PlaybackGenericRequest {
            request_id,
            media_session_id,
            typ: MESSAGE_TYPE_PAUSE.to_string(),
            custom_data: proxies::media::CustomData::new(),
        })?;

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        self.receive_status_entry(request_id, media_session_id)
    }

    /// Begins playback of the content that was loaded with the load call, playback is continued
    /// from the current time position.
    ///
    /// # Arguments
    ///
    /// * `destination` - `protocol` of the media application (e.g. `web-1`);
    /// * `media_session_id` - ID of the media session to be played.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn play<S>(&self, destination: S, media_session_id: i32) -> Result<StatusEntry, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let payload = serde_json::to_string(&proxies::media::PlaybackGenericRequest {
            request_id,
            media_session_id,
            typ: MESSAGE_TYPE_PLAY.to_string(),
            custom_data: proxies::media::CustomData::new(),
        })?;

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        self.receive_status_entry(request_id, media_session_id)
    }

    /// Stops playback of the current content. Triggers a STATUS event notification to all sender
    /// applications. After this command the content will no longer be loaded and the
    /// media_session_id is invalidated.
    ///
    /// # Arguments
    ///
    /// * `destination` - `protocol` of the media application (e.g. `web-1`);
    /// * `media_session_id` - ID of the media session to be stopped.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn stop<S>(&self, destination: S, media_session_id: i32) -> Result<StatusEntry, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let payload = serde_json::to_string(&proxies::media::PlaybackGenericRequest {
            request_id,
            media_session_id,
            typ: MESSAGE_TYPE_STOP.to_string(),
            custom_data: proxies::media::CustomData::new(),
        })?;

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        self.receive_status_entry(request_id, media_session_id)
    }

    /// Sets the current position in the stream. Triggers a STATUS event notification to all sender
    /// applications. If the position provided is outside the range of valid positions for the
    /// current content, then the player should pick a valid position as close to the requested
    /// position as possible.
    ///
    /// # Arguments
    ///
    /// * `destination` - `protocol` of the media application (e.g. `web-1`);
    /// * `media_session_id` - ID of the media session to seek in;
    /// * `current_time` - Time in seconds to seek to.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    pub fn seek<S>(
        &self,
        destination: S,
        media_session_id: i32,
        current_time: Option<f32>,
        resume_state: Option<ResumeState>,
    ) -> Result<StatusEntry, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let request_id = self.message_manager.generate_request_id();

        let payload = serde_json::to_string(&proxies::media::PlaybackSeekRequest {
            request_id,
            media_session_id,
            typ: MESSAGE_TYPE_SEEK.to_string(),
            current_time,
            resume_state: resume_state.map(|s| s.to_string()),
            custom_data: proxies::media::CustomData::new(),
        })?;

        self.message_manager.send(CastMessage {
            namespace: CHANNEL_NAMESPACE.to_string(),
            source: self.sender.to_string(),
            destination: destination.into().to_string(),
            payload: CastMessagePayload::String(payload),
        })?;

        self.receive_status_entry(request_id, media_session_id)
    }

    pub fn can_handle(&self, message: &CastMessage) -> bool {
        message.namespace == CHANNEL_NAMESPACE
    }

    pub fn parse(&self, message: &CastMessage) -> Result<MediaResponse, Error> {
        let reply = match message.payload {
            CastMessagePayload::String(ref payload) => {
                serde_json::from_str::<serde_json::Value>(payload)?
            }
            _ => {
                return Err(Error::Internal(
                    "Binary payload is not supported!".to_string(),
                ))
            }
        };

        let message_type = reply
            .as_object()
            .and_then(|object| object.get("type"))
            .and_then(|property| property.as_str())
            .unwrap_or("")
            .to_string();

        let response = match message_type.as_ref() {
            MESSAGE_TYPE_MEDIA_STATUS => {
                let reply: proxies::media::StatusReply = serde_json::value::from_value(reply)?;

                let statuses_entries = reply.status.iter().map(|x| {
                    StatusEntry {
                        media_session_id: x.media_session_id,
                        media: x.media.as_ref().map(|m| {
                            Media {
                                content_id: m.content_id.to_string(),
                                stream_type: StreamType::from_str(m.stream_type.as_ref()).unwrap(),
                                content_type: m.content_type.to_string(),
                                metadata: None, // TODO
                                duration: m.duration,
                                tracks: vec![]
                            }
                        }),
                        playback_rate: x.playback_rate,
                        player_state: PlayerState::from_str(x.player_state.as_ref()).unwrap(),
                        idle_reason: x
                            .idle_reason
                            .as_ref()
                            .map(|reason| IdleReason::from_str(reason).unwrap()),
                        current_time: x.current_time,
                        supported_media_commands: x.supported_media_commands,
                    }
                });

                MediaResponse::Status(Status {
                    request_id: reply.request_id,
                    entries: statuses_entries.collect::<Vec<StatusEntry>>(),
                })
            }
            MESSAGE_TYPE_LOAD_CANCELLED => {
                let reply: proxies::media::LoadCancelledReply =
                    serde_json::value::from_value(reply)?;

                MediaResponse::LoadCancelled(LoadCancelled {
                    request_id: reply.request_id,
                })
            }
            MESSAGE_TYPE_LOAD_FAILED => {
                let reply: proxies::media::LoadFailedReply = serde_json::value::from_value(reply)?;

                MediaResponse::LoadFailed(LoadFailed {
                    request_id: reply.request_id,
                })
            }
            MESSAGE_TYPE_INVALID_PLAYER_STATE => {
                let reply: proxies::media::InvalidPlayerStateReply =
                    serde_json::value::from_value(reply)?;

                MediaResponse::InvalidPlayerState(InvalidPlayerState {
                    request_id: reply.request_id,
                })
            }
            MESSAGE_TYPE_INVALID_REQUEST => {
                let reply: proxies::media::InvalidRequestReply =
                    serde_json::value::from_value(reply)?;

                MediaResponse::InvalidRequest(InvalidRequest {
                    request_id: reply.request_id,
                    reason: reply.reason,
                })
            }
            _ => MediaResponse::NotImplemented(message_type.to_string(), reply),
        };

        Ok(response)
    }

    /// Waits for the status entry with specified `request_id` and `media_session_id`. This method
    /// is very handy for the media playback methods where particular `StatusEntry` is required.
    ///
    /// # Arguments
    ///
    /// * `request_id` - ID of the request that caused status entry to be broadcasted.
    /// * `media_session_id` - ID of the media session to receive.
    ///
    /// # Return value
    ///
    /// Returned `Result` should consist of either `Status` instance or an `Error`.
    fn receive_status_entry(
        &self,
        request_id: i32,
        media_session_id: i32,
    ) -> Result<StatusEntry, Error> {
        self.message_manager.receive_find_map(|message| {
            if !self.can_handle(message) {
                return Ok(None);
            }

            match self.parse(message)? {
                MediaResponse::Status(mut status) => {
                    if status.request_id == request_id {
                        let position = status
                            .entries
                            .iter()
                            .position(|e| e.media_session_id == media_session_id);

                        return Ok(position.map(|position| status.entries.remove(position)));
                    }
                }
                MediaResponse::InvalidPlayerState(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(
                            "Request failed because of invalid player state.".to_string(),
                        ));
                    }
                }
                MediaResponse::InvalidRequest(error) => {
                    if error.request_id == request_id {
                        return Err(Error::Internal(format!(
                            "Invalid request ({}).",
                            error.reason.unwrap_or_else(|| "Unknown".to_string())
                        )));
                    }
                }
                _ => {}
            }

            Ok(None)
        })
    }
}
